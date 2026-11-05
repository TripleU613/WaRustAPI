"""Execute native task cancellation with controlled dictionary/network/queue adapters."""
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
from unicorn.arm64_const import UC_ARM64_REG_X3
manager,dictionary,endpoints,provider,network,callbacks,null_object=[0x10000100+i*0x100 for i in range(7)]
rows=[]
failures=[]
canceled=[]
queued=[]
params={}
allocations={}
released=[]
cleared=False
heap_next=0x10009000
def imported(uc,address,size,user):
    global cleared,heap_next
    name=imports[address]
    x0,x1,x2,x3=[uc.reg_read(r) for r in [UC_ARM64_REG_X0,UC_ARM64_REG_X1,UC_ARM64_REG_X2,UC_ARM64_REG_X3]]
    if name=='__MCFCastToClassNotNull':uc.reg_write(UC_ARM64_REG_X0,x1)
    elif name=='MCFDictionaryGetCount':
        assert x0==dictionary;uc.reg_write(UC_ARM64_REG_X0,0 if cleared else len(rows))
    elif name=='MCFSafeMalloc':
        assert 0<x0<=8 and x1==8
        ptr=heap_next;heap_next+=x0*x1
        allocations[ptr]=x0*x1;uc.mem_write(ptr,bytes(x0*x1));uc.reg_write(UC_ARM64_REG_X0,ptr)
    elif name=='MCFDictionaryGetKeysAndValues':
        assert x0==dictionary
        uc.mem_write(x1,b''.join(struct.pack('<Q',k) for k,v in rows))
        uc.mem_write(x2,b''.join(struct.pack('<Q',v) for k,v in rows))
    elif name=='MCFGetTypeID':uc.reg_write(UC_ARM64_REG_X0,1 if x0==null_object else 2)
    elif name=='MCFNullGetTypeID':uc.reg_write(UC_ARM64_REG_X0,1)
    elif name=='MCINetworkSessionCancelTask':
        canceled.append([hex(x0),hex(x1)]);uc.reg_write(UC_ARM64_REG_X0,0)
    elif name=='MCIParamsCreate':
        assert x0==2 and x1==callbacks
        ptr=0x10007000+len(params)*32;params[ptr]=(x1,x2);uc.reg_write(UC_ARM64_REG_X0,ptr)
    elif name=='MCIGlobalGetMCIExecutionGlobalQueueForUtility':uc.reg_write(UC_ARM64_REG_X0,0x10008000)
    elif name=='MCIGlobalExecuteAsyncHelper':
        assert x0==0x10008000 and x2==0xbd8f98 and x3 in params
        queued.append({'callback_va':hex(x2),'callback_table':hex(params[x3][0]),'task_key':hex(params[x3][1])})
        uc.reg_write(UC_ARM64_REG_X0,0)
    elif name=='MCFRelease':released.append(hex(x0));uc.reg_write(UC_ARM64_REG_X0,0)
    elif name=='MCFFree':assert x0 in allocations;allocations.pop(x0);uc.reg_write(UC_ARM64_REG_X0,0)
    elif name=='MCFDictionaryRemoveAllValues':
        assert x0==dictionary;cleared=True;uc.reg_write(UC_ARM64_REG_X0,0)
    elif name=='abort':failures.append('native abort');uc.emu_stop()
    else:failures.append('unexpected import '+name);uc.emu_stop()
def interrupted(*args):
    failures.append('target interrupt');machine.emu_stop()
machine.hook_add(UC_HOOK_CODE,imported,begin=0x30000100,end=next_stub-1)
machine.hook_add(UC_HOOK_INTR,interrupted)
def type_getter(uc,address,size,user):
    uc.reg_write(UC_ARM64_REG_X0,0x1234)
    uc.reg_write(UC_ARM64_REG_PC,uc.reg_read(UC_ARM64_REG_LR))
machine.hook_add(UC_HOOK_CODE,type_getter,begin=0x727aec,end=0x727aec)
results=[]
cases=[
 ('absent-empty',False,False,[],True,False),
 ('active-empty',True,True,[],True,False),
 ('placeholder',True,True,['placeholder'],True,False),
 ('active-task',True,True,['active'],True,False),
 ('mixed',True,True,['placeholder','active'],True,False),
 ('no-callback-table',True,True,['active'],False,False),
 ('missing-provider',True,False,[],True,True),
 ('orphan-provider',False,True,[],True,True),
 ('orphan-tasks',False,False,['placeholder'],True,True),
 ('missing-backend',True,True,['missing-backend'],True,True),
 ('null-value',True,True,['null'],True,True),
 ('destructor-active',True,True,['active'],True,False),
]
for name,has_network,has_provider,kinds,has_callbacks,expect_abort in cases:
    rows=[];failures.clear();canceled.clear();queued.clear();params.clear();released.clear();allocations.clear()
    cleared=False;heap_next=0x10009000
    machine.mem_write(manager,bytes(0x100))
    for slot,value in [(0x48,provider if has_provider else 0),(0x58,network if has_network else 0),(0x60,dictionary),(0x68,endpoints),(0x70,callbacks if has_callbacks else 0)]:machine.mem_write(manager+slot,struct.pack('<Q',value))
    machine.mem_write(manager+0xc0,b'\x01')
    for i,kind in enumerate(kinds):
        key=0x10001000+i*0x100
        value=null_object if kind=='placeholder' else 0 if kind=='null' else 0x10003000+i*0x100
        rows.append((key,value))
        if kind not in ['placeholder','null']:
            backend=0 if kind=='missing-backend' else 0x10005000+i*0x100
            machine.mem_write(value+0x58,struct.pack('<Q',backend))
            if backend:
                machine.mem_write(backend+0x30,struct.pack('<Q',0x10006000+i*0x100))
                machine.mem_write(backend+0x58,struct.pack('<Q',0x10006400+i*0x100))
    machine.reg_write(UC_ARM64_REG_SP,0x200ff000)
    machine.reg_write(UC_ARM64_REG_LR,0x30000000)
    machine.reg_write(UC_ARM64_REG_X0,manager)
    machine.emu_start(0x727b28 if name=='destructor-active' else 0x727bd0,0x30000000,timeout=1000000,count=100000)
    assert failures==(['native abort'] if expect_abort else []),(name,failures)
    if not expect_abort:
        assert machine.reg_read(UC_ARM64_REG_PC)==0x30000000 and not allocations
        assert canceled==[[hex(0x10006000+i*0x100),hex(0x10006400+i*0x100)] for i,k in enumerate(kinds) if k=='active']
        assert [q['task_key'] for q in queued]==([hex(k) for k,v in rows] if has_callbacks and name!='destructor-active' else [])
        if name=='destructor-active':
            assert released[0]==hex(callbacks)
            assert struct.unpack('<Q',machine.mem_read(manager+0x70,8))[0]==0
        assert cleared==has_network
        assert struct.unpack('<Q',machine.mem_read(manager+0x48,8))[0]==0
        assert struct.unpack('<Q',machine.mem_read(manager+0x58,8))[0]==0
        assert machine.mem_read(manager+0xc0,1)[0]==(0 if has_network else 1)
    assert struct.unpack('<Q',machine.mem_read(manager+0x68,8))[0]==endpoints
    results.append({'case':name,'aborted':bool(failures),'cancel_requests':canceled[:],
                    'queued_callbacks':queued[:],'active_map_cleared':cleared,
                    'endpoint_pointer_unchanged':True,'busy_byte':machine.mem_read(manager+0xc0,1)[0],
                    'released_objects':released[:]})
print(json.dumps({'scope':'synthetic cancellation/destructor only; queue/network cancellation recorded, never executed',
 'unicorn_version':'2.1.4','instruction_cap_per_case':100000,'timeout_microseconds_per_case':1000000,'cases':results},indent=2))
