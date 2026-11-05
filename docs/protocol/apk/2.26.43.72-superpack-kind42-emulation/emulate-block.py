"""Bounded emulation of the pinned type-42 decoder; no Android runtime or network.
Requires unicorn==2.1.4. Arguments: pinned.apk block_ordinal ignored_output.bin
Only mapped imports are handled; unsupported imports and interrupts stop the run.
"""
import sys,struct,json,hashlib,zipfile
from pathlib import Path
from unicorn import Uc,UC_ARCH_ARM64,UC_MODE_ARM,UC_HOOK_CODE,UC_HOOK_INTR
from unicorn.arm64_const import *
apk=Path(sys.argv[1]);ordinal=int(sys.argv[2]);out=Path(sys.argv[3])
assert hashlib.sha256(apk.read_bytes()).hexdigest()=='887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e'
with zipfile.ZipFile(apk) as z:
 b=z.read('lib/arm64-v8a/libsuperpack.so');c=z.read('lib/arm64-v8a/libs.so')
assert hashlib.sha256(b).hexdigest()=='fe24d8091d7fc96e0b3dee9cda3d7ecb5ec5cbe4582b86d9c42010b04d98e45f'
assert hashlib.sha256(c).hexdigest()=='1387f4a589266b39551c5ace0bd1cf11e4b658b341519e6bb38e70bfa48f33d5'
blocks=json.loads((Path(__file__).resolve().parent.parent / '2.26.43.72-superpack-transform-blocks/blocks.json').read_text())['blocks'];block=blocks[ordinal];assert block['kind']==42
start=0x870+int(block['payload_archive_offset'],16);payload=c[start:start+block['encoded_bytes']];assert hashlib.sha256(payload).hexdigest()==block['payload_sha256']
u=Uc(UC_ARCH_ARM64,UC_MODE_ARM)
for base,size in [(0,0x40000),(0x10000000,0x10000000),(0x20000000,0x2000000),(0x30000000,0x4000000),(0x40000000,0x800000),(0x50000000,0x10000)]:u.mem_map(base,size)
phoff=struct.unpack_from('<Q',b,32)[0];phsize,phnum=struct.unpack_from('<HH',b,54)
for i in range(phnum):
 typ,flags,off,va,pa,filesz,memsz,align=struct.unpack_from('<IIQQQQQQ',b,phoff+i*phsize)
 if typ==1:u.mem_write(va,b[off:off+filesz])
shoff=struct.unpack_from('<Q',b,40)[0];shsize,shnum=struct.unpack_from('<HH',b,58)
sections=[struct.unpack_from('<IIQQQQIIQQ',b,shoff+i*shsize) for i in range(shnum)]
def symbol(section,index):
 sym=sections[section[6]];strings=sections[sym[6]];name=struct.unpack_from('<I',b,sym[4]+index*sym[9])[0];pos=strings[4]+name;return b[pos:b.index(0,pos)].decode()
u.mem_write(0x50002000,bytes.fromhex('e90300aa820000b421150038420400f1c1ffff54c0035fd6e90300aaa20000b42a1440382a150038420400f1a1ffff54c0035fd6'))
hooks={};cursor=0x50000000
for s in sections:
 if s[1]!=4:continue
 for off in range(s[4],s[4]+s[5],24):
  addr,info,addend=struct.unpack_from('<QQq',b,off);typ=info&0xffffffff
  if typ==1027:assert info>>32==0;u.mem_write(addr,struct.pack('<Q',addend))
  elif typ in [1025,1026]:
   name=symbol(s,info>>32)
   if name in ['memset','memcpy']:
    u.mem_write(addr,struct.pack('<Q',0x50002000+(24 if name=='memcpy' else 0)));continue
   hooks[cursor]=name;u.mem_write(cursor,bytes.fromhex('c0035fd6'));u.mem_write(addr,struct.pack('<Q',cursor));cursor+=4
  else:raise RuntimeError(('reloc',typ))
heap=0x10000000;allocations={};counts={};failure=None
regs=[UC_ARM64_REG_X0,UC_ARM64_REG_X1,UC_ARM64_REG_X2,UC_ARM64_REG_X3,UC_ARM64_REG_X4,UC_ARM64_REG_X5]
def alloc(n):
 global heap
 n=max(n,1);p=heap;heap=(heap+n+15)&~15
 if heap>=0x20000000:raise RuntimeError('heap cap')
 allocations[p]=n;return p
def string(p,cap=1000000):
 result=bytearray()
 for i in range(cap):
  c=u.mem_read(p+i,1)[0]
  if c==0:return bytes(result)
  result.append(c)
 raise RuntimeError('string cap')
def callback(uc,address,size,user):
 global failure
 try:
  name=hooks[address];a=[u.reg_read(r) for r in regs];counts[name]=counts.get(name,0)+1
  if name=='malloc':v=alloc(a[0])
  elif name=='calloc':v=alloc(a[0]*a[1]);u.mem_write(v,bytes(a[0]*a[1]))
  elif name=='free':v=0
  elif name=='realloc':
   v=alloc(a[1]);n=min(allocations.get(a[0],0),a[1]);u.mem_write(v,bytes(u.mem_read(a[0],n)))
  elif name in ['memcpy','memmove','__memcpy_chk','__memmove_chk']:u.mem_write(a[0],bytes(u.mem_read(a[1],a[2])));v=a[0]
  elif name in ['memset','__memset_chk']:u.mem_write(a[0],bytes([a[1]&255])*a[2]);v=a[0]
  elif name=='memcmp':x=bytes(u.mem_read(a[0],a[2]));y=bytes(u.mem_read(a[1],a[2]));v=(x>y)-(x<y)
  elif name=='strlen':v=len(string(a[0]))
  elif name=='__android_log_print':v=0
  else:raise RuntimeError('unsupported import '+name)
  u.reg_write(UC_ARM64_REG_X0,v&0xffffffffffffffff)
 except Exception as e:failure=str(e);u.emu_stop()
u.hook_add(UC_HOOK_CODE,callback,begin=0x50000000,end=cursor-1)
def interrupt(*args):
 global failure
 failure='target interrupt';u.emu_stop()
u.hook_add(UC_HOOK_INTR,interrupt)
u.mem_write(0x20000000,payload);u.reg_write(UC_ARM64_REG_SP,0x407ff000);u.reg_write(UC_ARM64_REG_LR,0x50001000);u.reg_write(UC_ARM64_REG_CPACR_EL1,3<<20)
for r,v in zip(regs,[0x20000000,0x30000000,0,block['declared_output_bytes']]):u.reg_write(r,v)
try:u.emu_start(0x218bc,0x50001000,timeout=45000000,count=10000000000)
except Exception as e:failure=str(e)
ret=u.reg_read(UC_ARM64_REG_X0);pc=u.reg_read(UC_ARM64_REG_PC)
print(json.dumps({'ordinal':ordinal,'return':ret,'pc':hex(pc),'failure':failure,'imports':counts,'heap_bytes':heap-0x10000000}),flush=True)
if failure or pc!=0x50001000 or ret!=block['declared_output_bytes']:raise SystemExit(1)
data=bytes(u.mem_read(0x30000000,ret));out.write_bytes(data);print(json.dumps({'bytes':len(data),'sha256':hashlib.sha256(data).hexdigest(),'prefix':data[:16].hex()}))
