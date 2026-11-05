"""Execute selected deletion worker with synthetic objects and intercepted remove()."""
import contextlib
import io
import json
from pathlib import Path
import runpy
import struct
from unicorn import Uc, UC_ARCH_ARM64, UC_MODE_ARM, UC_HOOK_CODE, UC_HOOK_INTR
from unicorn.arm64_const import UC_ARM64_REG_SP, UC_ARM64_REG_LR, UC_ARM64_REG_X0, UC_ARM64_REG_X1, UC_ARM64_REG_X2, UC_ARM64_REG_PC, UC_ARM64_REG_TPIDR_EL0
source = Path(__file__).resolve().parent.parent / '2.26.43.72-metrics-native-event-routing/resolve-jni.py'
with contextlib.redirect_stdout(io.StringIO()):
    evidence = runpy.run_path(str(source))
data, sections = evidence['data'], evidence['sections']
machine = Uc(UC_ARCH_ARM64, UC_MODE_ARM)
machine.mem_map(0, 0xc50000)
machine.mem_map(0x10000000, 0x10000)
machine.mem_map(0x20000000, 0x100000)
machine.mem_map(0x30000000, 0x10000)
phoff = struct.unpack_from('<Q',data,32)[0]
phsize, phnum = struct.unpack_from('<HH',data,54)
for index in range(phnum):
    kind,flags,offset,va,pa,filesz,memsz,align = struct.unpack_from('<IIQQQQQQ',data,phoff+index*phsize)
    if kind == 1: machine.mem_write(va,data[offset:offset+filesz])
imports = {}
next_stub = 0x30000100
for section in sections:
    if section[1] != 4: continue
    for offset in range(section[4],section[4]+section[5],24):
        address,info,addend = struct.unpack_from('<QQq',data,offset)
        kind = info & 0xffffffff
        if kind == 1027:
            assert info >> 32 == 0
            machine.mem_write(address,struct.pack('<Q',addend))
        elif kind in [257,1025,1026]:
            symbols = sections[section[6]]
            symbol = struct.unpack_from('<IBBHQQ',data,symbols[4]+(info>>32)*symbols[9])
            if kind == 257 and symbol[3] != 0:
                machine.mem_write(address,struct.pack('<Q',symbol[4]+addend))
                continue
            names = sections[symbols[6]]
            name_offset = struct.unpack_from('<I',data,symbols[4]+(info>>32)*symbols[9])[0]
            position = names[4]+name_offset
            name = data[position:data.index(0,position)].decode('ascii')
            imports[next_stub] = name
            machine.mem_write(next_stub,bytes.fromhex('c0035fd6'))
            machine.mem_write(address,struct.pack('<Q',next_stub+addend))
            next_stub += 4
        else:
            raise RuntimeError(('unsupported relocation',kind))
from unicorn.arm64_const import UC_ARM64_REG_X3, UC_ARM64_REG_X4
params,owner,store,descriptor,completion,path,cpath,error,errno_ptr,queue,root=[0x10000100+i*0x200 for i in range(11)]
events=[]
failures=[]
status=0
errnum=0
on_context=True
def ret(uc,value):uc.reg_write(UC_ARM64_REG_X0,value)
def local(uc,address,size,user):
    x0,x1,x2=[uc.reg_read(r) for r in [UC_ARM64_REG_X0,UC_ARM64_REG_X1,UC_ARM64_REG_X2]]
    if address in [0x725ee0,0x71f5b0]:ret(uc,101 if address==0x725ee0 else 102)
    elif address==0x6fe0d0:
        assert (x0,x1,x2)==(root,descriptor,1)
        events.append({'action':'path-request','temporary_argument':x2});ret(uc,path)
    elif address==0x71f248:
        assert (x0,x1,x2)==(completion,descriptor,error if status else 0)
        events.append({'action':'completion','error_present':bool(x2)})
    else:raise AssertionError(hex(address))
    uc.reg_write(UC_ARM64_REG_PC,uc.reg_read(UC_ARM64_REG_LR))
def imported(uc,address,size,user):
    x0,x1,x2,x3,x4=[uc.reg_read(r) for r in [UC_ARM64_REG_X0,UC_ARM64_REG_X1,UC_ARM64_REG_X2,UC_ARM64_REG_X3,UC_ARM64_REG_X4]]
    name=imports[address]
    if name=='MCIParamsGetTypeID':ret(uc,100)
    elif name=='__MCFCastToClassNotNull':assert x0==100 and x1==params;ret(uc,params)
    elif name=='MCIParamsUnpackWithTypeIDs':
        assert x0==2
        uc.mem_write(x2,struct.pack('<Q',owner));uc.mem_write(x4,struct.pack('<Q',descriptor))
    elif name=='MCIExecutionGlobalQueueCheckIsOnContext':assert x0==queue;ret(uc,int(on_context))
    elif name=='MCFASCIIStringGetCString':assert x0==path;ret(uc,cpath)
    elif name=='remove':
        assert x0==cpath
        events.append({'action':'remove','return_value':status});ret(uc,status & 0xffffffffffffffff)
    elif name=='__errno':ret(uc,errno_ptr)
    elif name=='MCFErrorCreate':
        assert x0==0xc30098 and x1==errnum and x2==0
        events.append({'action':'error-create','errno':x1,'domain_va':hex(x0)});ret(uc,error)
    elif name=='MCFRelease':events.append({'action':'release','object':hex(x0)})
    elif name=='abort':events.append({'action':'abort'});uc.emu_stop()
    else:failures.append('unexpected import '+name);uc.emu_stop()
def interrupted(*args):failures.append('target interrupt');machine.emu_stop()
machine.hook_add(UC_HOOK_CODE,imported,begin=0x30000100,end=next_stub-1)
for address in [0x725ee0,0x71f5b0,0x6fe0d0,0x71f248]:machine.hook_add(UC_HOOK_CODE,local,begin=address,end=address)
machine.hook_add(UC_HOOK_INTR,interrupted)
machine.reg_write(UC_ARM64_REG_TPIDR_EL0,0x1000f000)
machine.mem_write(owner+0x18,struct.pack('<Q',store));machine.mem_write(owner+0x28,struct.pack('<Q',completion))
machine.mem_write(store+0x18,struct.pack('<Q',queue));machine.mem_write(store+0x28,struct.pack('<Q',root))
results=[]
for name,status,errnum,on_context in [('removed',0,0,True),('missing-file',-1,2,True),('permission-denied',-1,13,True),('wrong-context',0,0,False)]:
    events.clear();failures.clear();machine.mem_write(errno_ptr,struct.pack('<I',errnum))
    machine.reg_write(UC_ARM64_REG_SP,0x200ff000);machine.reg_write(UC_ARM64_REG_LR,0x30000000);machine.reg_write(UC_ARM64_REG_X0,params)
    machine.emu_start(0x726580,0x30000000,timeout=1000000,count=100000)
    assert not failures,failures
    if not on_context:assert events==[{'action':'abort'}]
    else:
        assert machine.reg_read(UC_ARM64_REG_PC)==0x30000000
        expected=['path-request','remove']+(['error-create'] if status else [])+['completion']+(['release'] if status else [])+['release']
        assert [x['action'] for x in events]==expected
        assert events[-1]['object']==hex(path)
        if status:assert events[-2]['object']==hex(error)
    results.append({'case':name,'events':events[:]})
print(json.dumps({'scope':'complete selected worker with typed parameter, path, remove, errno and callback adapters; no real file deletion or durable storage',
 'unicorn_version':'2.1.4','cases':results},indent=2))
