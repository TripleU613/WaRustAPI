"""Reconstruct pinned ARM64 members from digest-verified decoded streams.
Run from repository root with unicorn==2.1.4:
  python reconstruct-members.py PINNED.apk 0 IGNORED_OUTPUT_DIRECTORY
Argument 0 is a retained bootstrap block selector, not a member/range selector.
All 83 members are reconstructed in metadata order. No target OS/network APIs.
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
blocks=json.loads(Path('docs/protocol/apk/2.26.43.72-superpack-transform-blocks/blocks.json').read_text())['blocks'];block=blocks[ordinal];assert block['kind']==42
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
u.mem_write(0x50003000,bytes.fromhex('fd7bb9a9f35301a9f55b02a9f76303a9f96b04a9fb7305a9f30300aaf40301aaf50302aaf60303aa9f0a00f1e306005497fe41d3d70000b4f70600d1f90317aaf80314aa0b000094fbffff17980600d1d80500b4fb0313aa1c4f159b21000094190080d203000094180700d1f9ffff17fe3300f93afb7fd35a0700915f0318ebc2020054490700913f0118eb220100545b4f159b3c4d159be0031baae1031caac0023fd61f0000714a0000545a0700913b4f159b5c4f159be0031baae1031caac0023fd61f0000718a00005405000094f9031aaae8ffff17fe3340f9c0035fd6090080d23f0115ebe20000546a6b69388b6b69386b6b29388a6b293829050091f9ffff17c0035fd6f35341a9f55b42a9f76343a9f96b44a9fb7345a9fd7bc7a8c0035fd6'))
hooks={};cursor=0x50000000
for s in sections:
 if s[1]!=4:continue
 for off in range(s[4],s[4]+s[5],24):
  addr,info,addend=struct.unpack_from('<QQq',b,off);typ=info&0xffffffff
  if typ==1027:assert info>>32==0;u.mem_write(addr,struct.pack('<Q',addend))
  elif typ in [1025,1026]:
   name=symbol(s,info>>32)
   if name=='qsort':
    u.mem_write(addr,struct.pack('<Q',0x50003000));continue
   if name in ['memset','memcpy']:
    u.mem_write(addr,struct.pack('<Q',0x50002000+(24 if name=='memcpy' else 0)));continue
   hooks[cursor]=name;u.mem_write(cursor,bytes.fromhex('c0035fd6'));u.mem_write(addr,struct.pack('<Q',cursor));cursor+=4
  else:raise RuntimeError(('reloc',typ))
heap=0x10000000;allocations={};free_pool=[];counts={};failure=None
regs=[UC_ARM64_REG_X0,UC_ARM64_REG_X1,UC_ARM64_REG_X2,UC_ARM64_REG_X3,UC_ARM64_REG_X4,UC_ARM64_REG_X5]
def alloc(n):
 global heap
 n=(max(n,1)+15)&~15
 for i,(p,space) in enumerate(free_pool):
  if space>=n:
   free_pool.pop(i);allocations[p]=space;return p
 p=heap;heap=(heap+n+15)&~15
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
  elif name=='free':
   if a[0]:free_pool.append((a[0],allocations.pop(a[0])))
   v=0
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

u.reg_write(UC_ARM64_REG_CPACR_EL1,3<<20)
def call(entry,args):
 global failure
 failure=None
 u.reg_write(UC_ARM64_REG_SP,0x407ff000);u.reg_write(UC_ARM64_REG_LR,0x50001000)
 for r,v in zip(regs,args):u.reg_write(r,v)
 try:u.emu_start(entry,0x50001000,timeout=45000000,count=10000000000)
 except Exception as e:failure=str(e)
 pc=u.reg_read(UC_ARM64_REG_PC);ret=u.reg_read(UC_ARM64_REG_X0)
 if failure or pc!=0x50001000:raise RuntimeError((hex(entry),hex(pc),failure))
 return ret
def q(p):return struct.unpack('<Q',u.mem_read(p,8))[0]
def wq(p,v):u.mem_write(p,struct.pack('<Q',v))
vector=call(0x2ded0,[0x903]);assert vector>0x17
pointers=q(vector+8)
manifest=json.loads(Path('docs/protocol/apk/2.26.43.72-superpack-all-blocks/decoded-blocks.json').read_text())
for block,row in zip(blocks,manifest):
 assert block['ordinal']==row['ordinal']
 raw=Path(f'../android-lab/output/metrics-native/framed-decoded/{row["ordinal"]:03d}.bin').read_bytes()
 assert hashlib.sha256(raw).hexdigest()==row['sha256']
 stream=q(pointers+block['block_id']*8)
 assert call(0x2e33c,[stream,len(raw)])==0
 buffer=q(stream);u.mem_write(q(buffer+8),raw);wq(stream+0x10,len(raw))
config=alloc(64);u.mem_write(config,c[0x870+28:0x870+56]);wq(config+0x1d,0)
metadata=json.loads(Path('docs/protocol/apk/2.26.43.72-superpack-metadata/metadata.json').read_text())

call(0xb488,[])
output_dir=out;output_dir.mkdir(exist_ok=True)
results=[]
for m in metadata['members']:
 size=m['declared_size'];offset=0;u.mem_write(0x30000000,bytes(size))
 while offset<size:
  ret=call(0x1adb8,[config,vector,0x30000000+offset,size,0x20000000]);consumed=q(0x20000000)
  if ret or not 0<consumed<=size-offset:raise RuntimeError((m['ordinal'],ret,consumed,size-offset))
  offset+=consumed
 raw=bytes(u.mem_read(0x30000000,size))
 assert raw[:6]==bytes.fromhex('7f454c460201'),m['ordinal']
 (output_dir/m['name']).write_bytes(raw)
 results.append({'ordinal':m['ordinal'],'name':m['name'],'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest(),'machine':struct.unpack_from('<H',raw,18)[0]})
 (output_dir/'results.json').write_text(json.dumps(results,indent=2)+'\n')
 print(json.dumps(results[-1]),flush=True)
print(json.dumps({'completed_members':len(results),'heap_bytes':heap-0x10000000,'imports':counts}),flush=True)
