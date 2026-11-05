"""Execute native tag-selection branches with controlled append/zero-format adapters."""
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
from unicorn.arm64_const import UC_ARM64_REG_X4,UC_ARM64_REG_X19,UC_ARM64_REG_X29
root=0x10000100
calls=[]
failures=[]
def text(uc,address):
    value=bytearray()
    for i in range(2048):
        b=uc.mem_read(address+i,1)[0]
        if not b:return value.decode('ascii')
        value.append(b)
    raise AssertionError('unterminated string')
def ret(uc):uc.reg_write(UC_ARM64_REG_PC,uc.reg_read(UC_ARM64_REG_LR))
def controlled(uc,address,size,user):
    if address==0x72cd78:
        key=text(uc,uc.reg_read(UC_ARM64_REG_X1));value=text(uc,uc.reg_read(UC_ARM64_REG_X2))
        calls.append((key,value));ret(uc)
    elif address==0x72cd24:
        # All fixtures supply zero; no general integer formatting is emulated.
        assert uc.reg_read(UC_ARM64_REG_X4)==0
        uc.mem_write(uc.reg_read(UC_ARM64_REG_X0),b'0\0');ret(uc)
    elif address in imports:
        failures.append('unexpected import '+imports[address]);uc.emu_stop()
def interrupted(*args):
    failures.append('target interrupt');machine.emu_stop()
machine.hook_add(UC_HOOK_CODE,controlled)
machine.hook_add(UC_HOOK_INTR,interrupted)
purpose_names={1:'prefetch',2:'fetch',3:'refresh'}
category_names={1:'image',2:'video',3:'api',4:'analytics',5:'upload',6:'audio',7:'document'}
cases=[('selected-sender',1,4,0,'synthetic-product','',''),
       ('optional-strings',2,3,1,'synthetic-product','synthetic-surface','synthetic-application'),
       ('empty-strings',0,0,0,'','','')]
for value in [-1,0,1,2,3,4]:cases.append((f'purpose-{value}',value,0,0,None,None,None))
for value in [-1,0,1,2,3,4,5,6,7,8]:cases.append((f'category-{value}',0,value,0,None,None,None))
for value in [-1,0,1,2,3]:cases.append((f'is-ad-{value}',0,0,value,None,None,None))
results=[]
for name,purpose,category,is_ad,product,surface,application in cases:
    machine.mem_write(root,bytes(64))
    for slot,value,ptr in [(0,product,0x10001000),(8,surface,0x10002000),(24,application,0x10003000)]:
        if value is not None:
            machine.mem_write(ptr,value.encode()+b'\0')
            machine.mem_write(root+slot,struct.pack('<Q',ptr))
    for slot,value in [(16,purpose),(32,category),(36,is_ad)]:machine.mem_write(root+slot,struct.pack('<I',value & 0xffffffff))
    machine.reg_write(UC_ARM64_REG_SP,0x200ff000)
    machine.reg_write(UC_ARM64_REG_X29,0x200ff050)
    machine.reg_write(UC_ARM64_REG_X19,root)
    calls.clear();failures.clear()
    machine.emu_start(0x72caec,0x72cc90,timeout=1000000,count=100000)
    assert not failures and machine.reg_read(UC_ARM64_REG_PC)==0x72cc90,(name,failures)
    # Empty omission is independently grounded in the retained pair-helper strlen guards;
    # the native pair/JSON formatter itself is intercepted and is not executed.
    selected={k:v for k,v in calls if v}
    expected={'retry_attempt':'0'}
    for key,value in [('product',product),('surface',surface)]:
        if value:expected[key]=value
    if purpose in purpose_names:expected['purpose']=purpose_names[purpose]
    if category in category_names:expected['request_category']=category_names[category]
    if is_ad in [1,2]:expected['is_ad']='0' if is_ad==1 else '1'
    assert selected==expected,(name,selected,expected)
    # application_tags is outside the executed network-tags selection window.
    results.append({'case':name,'raw_purpose':purpose,'raw_category':category,'raw_is_ad':is_ad,
                    'pair_calls':[{'key':k,'value':v} for k,v in calls],
                    'nonempty_selected_pairs':selected,'matches_independent_enum_mapping':True})
print(json.dumps({'scope':'native network-tag branch selection only; append/zero-retry adapters; no JSON serialization or I/O',
 'unicorn_version':'2.1.4','instruction_cap_per_case':100000,'timeout_microseconds_per_case':1000000,'cases':results},indent=2))
