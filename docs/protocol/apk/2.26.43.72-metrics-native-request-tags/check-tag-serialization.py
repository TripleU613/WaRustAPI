"""Execute complete native tag serialization using bounded libc adapters."""
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
import ctypes
from unicorn.arm64_const import UC_ARM64_REG_X3
libc=ctypes.CDLL(None)
libc.snprintf.restype=ctypes.c_int
heap_base,heap_end=0x11000000,0x11100000
machine.mem_map(heap_base,heap_end-heap_base)
heap_next=heap_base
allocations={}
failures=[]
called={}
formats=[]
root=0x10000100
machine.reg_write(UC_ARM64_REG_TPIDR_EL0,0x10008000)
machine.mem_write(0x10008028,struct.pack('<Q',0x123456789abcdef))
def cstring(uc,address):
    result=bytearray()
    for i in range(8192):
        b=uc.mem_read(address+i,1)[0]
        if not b:return bytes(result)
        result.append(b)
    raise AssertionError('unterminated synthetic string')
def allocate(uc,count):
    global heap_next
    assert 0<count<=8192
    ptr=heap_next;heap_next+=(count+15)&~15
    assert heap_next<=heap_end
    uc.mem_write(ptr,b'\xa5'*count)
    allocations[ptr]=count
    return ptr
def imported(uc,address,size,user):
    name=imports[address];called[name]=called.get(name,0)+1
    x0,x1,x2,x3=[uc.reg_read(r) for r in [UC_ARM64_REG_X0,UC_ARM64_REG_X1,UC_ARM64_REG_X2,UC_ARM64_REG_X3]]
    if name=='malloc':uc.reg_write(UC_ARM64_REG_X0,allocate(uc,x0))
    elif name=='realloc':
        assert x0 in allocations
        previous=allocations.pop(x0)
        ptr=allocate(uc,x1)
        uc.mem_write(ptr,bytes(uc.mem_read(x0,min(previous,x1))))
        uc.reg_write(UC_ARM64_REG_X0,ptr)
    elif name=='free':
        if x0:assert x0 in allocations;allocations.pop(x0)
        uc.reg_write(UC_ARM64_REG_X0,0)
    elif name=='strlen':uc.reg_write(UC_ARM64_REG_X0,len(cstring(uc,x0)))
    elif name=='vsnprintf':
        fmt=cstring(uc,x2)
        kinds={b'%d':['int'],b'"%s":"%s",':['str','str'],b'"%s":%s,':['str','str']}[fmt]
        stack,gr_top,vr_top,gr_offs,vr_offs=struct.unpack('<QQQii',uc.mem_read(x3,32))
        args=[]
        for kind in kinds:
            if gr_offs<0:
                word=struct.unpack('<Q',uc.mem_read(gr_top+gr_offs,8))[0];gr_offs+=8
            else:
                word=struct.unpack('<Q',uc.mem_read(stack,8))[0];stack+=8
            args.append(ctypes.c_int(word&0xffffffff) if kind=='int' else ctypes.c_char_p(cstring(uc,word)))
        assert 0<x1<=8192
        if heap_base<=x0<heap_end:
            assert any(ptr<=x0 and x0+x1<=ptr+cap for ptr,cap in allocations.items())
        else:assert 0x20000000<=x0 and x0+x1<=0x20100000
        out=ctypes.create_string_buffer(x1)
        count=libc.snprintf(out,ctypes.c_size_t(x1),ctypes.c_char_p(fmt),*args)
        assert count>=0
        written=min(count,x1-1)
        uc.mem_write(x0,out.raw[:written+1])
        formats.append({'format':fmt.decode(),'capacity':x1,'would_write':count,'written':written})
        uc.reg_write(UC_ARM64_REG_X0,count)
    else:
        failures.append('unexpected import '+name);uc.emu_stop()
def interrupted(*args):
    failures.append('target interrupt');machine.emu_stop()
machine.hook_add(UC_HOOK_CODE,imported,begin=0x30000100,end=next_stub-1)
machine.hook_add(UC_HOOK_INTR,interrupted)
# Expected strings use independent JSON serialization for the supported safe inputs.
cases=[]
for retry in [0,1,9,99,999,9999,10000,2147483647,-1,-99,-999,-1000,-2147483648]:
    cases.append((f'retry-{retry}','synthetic-product','','',1,4,0,retry))
cases.extend([
 ('all-optional','synthetic-product','synthetic-surface','{"bucket":"synthetic"}',2,3,1,0),
 ('string-application',None,None,'synthetic-app',0,0,2,0),
 ('empty-fields','','','',0,0,0,0),
 ('long-product','x'*512,None,None,3,7,0,0),
 ('raw-object-product','{"synthetic":1}',None,None,1,4,0,0),
 ('unescaped-quote','synthetic"quote',None,None,1,4,0,0),
])
results=[]
for name,product,surface,application,purpose,category,is_ad,retry in cases:
    heap_next=heap_base;allocations.clear();failures.clear();called.clear();formats.clear()
    machine.mem_write(root,bytes(64))
    for slot,value,ptr in [(0,product,0x10001000),(8,surface,0x10003000),(24,application,0x10005000)]:
        if value is not None:
            machine.mem_write(ptr,value.encode()+b'\0');machine.mem_write(root+slot,struct.pack('<Q',ptr))
    for slot,value in [(16,purpose),(32,category),(36,is_ad),(40,retry)]:machine.mem_write(root+slot,struct.pack('<I',value & 0xffffffff))
    machine.reg_write(UC_ARM64_REG_SP,0x200ff000)
    machine.reg_write(UC_ARM64_REG_LR,0x30000000)
    machine.reg_write(UC_ARM64_REG_X0,root)
    machine.emu_start(0x72ca94,0x30000000,timeout=1000000,count=100000)
    assert not failures and machine.reg_read(UC_ARM64_REG_PC)==0x30000000,(name,failures)
    result_ptr=machine.reg_read(UC_ARM64_REG_X0)
    assert list(allocations)==[result_ptr],(name,allocations)
    actual=cstring(machine,result_ptr).decode('ascii')
    selected={}
    for key,value in [('product',product),('purpose',{1:'prefetch',2:'fetch',3:'refresh'}.get(purpose)),('surface',surface),('is_ad',{1:'0',2:'1'}.get(is_ad)),('request_category',{1:'image',2:'video',3:'api',4:'analytics',5:'upload',6:'audio',7:'document'}.get(category)),('retry_attempt',str(retry)[:4])]:
        if value:selected[key]=json.loads(value) if value.startswith('{') else value
    expected={'network_tags':selected}
    if application:expected['application_tags']=json.loads(application) if application.startswith('{') else application
    if name=='unescaped-quote':
        expected_text='{"network_tags":{"product":"synthetic"quote","purpose":"prefetch","request_category":"analytics","retry_attempt":"0"}}'
        try:json.loads(actual)
        except json.JSONDecodeError:pass
        else:raise AssertionError('quote fixture unexpectedly valid JSON')
    else:expected_text=json.dumps(expected,separators=(',',':'))
    assert actual==expected_text,(name,actual,expected_text)
    results.append({'case':name,'retry_input':retry,'native_bytes':actual,
                    'matches_independent_expected_bytes':True,'live_allocation_count':len(allocations),
                    'imports':dict(called),'format_calls':formats[:]})
print(json.dumps({'scope':'complete native tag builder with synthetic allocation and host libc snprintf adapters; no Android/network',
 'unicorn_version':'2.1.4','instruction_cap_per_case':100000,'timeout_microseconds_per_case':1000000,
 'cases':results},indent=2))
