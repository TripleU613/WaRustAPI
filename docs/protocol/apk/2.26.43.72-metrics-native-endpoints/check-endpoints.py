"""Execute selected endpoint append/store instructions with synthetic objects."""
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
from unicorn.arm64_const import UC_ARM64_REG_X19,UC_ARM64_REG_X21,UC_ARM64_REG_X22,UC_ARM64_REG_X23
manager,dynamic,output,task,dictionary=0x10000100,0x10000200,0x10000300,0x10000400,0x10000500
fallback_ascii,fallback_cstring,fallback_url=0x10000600,0x10000700,0x10000800
items=[]
appended=[]
failures=[]
stored=False
fallback_valid=True
def ret(uc,value):
    uc.reg_write(UC_ARM64_REG_X0,value);uc.reg_write(UC_ARM64_REG_PC,uc.reg_read(UC_ARM64_REG_LR))
def controlled(uc,address,size,user):
    global stored
    if address==0x7216b0:ret(uc,0);return  # diagnostic adapter
    if address==0x730a54:
        assert uc.reg_read(UC_ARM64_REG_X0)==0x16
        ret(uc,fallback_ascii);return  # explicit fixed-string selector adapter
    if address not in imports:return
    name=imports[address]
    x0,x1,x2=[uc.reg_read(r) for r in [UC_ARM64_REG_X0,UC_ARM64_REG_X1,UC_ARM64_REG_X2]]
    if name=='MCFArrayGetCount':
        assert x0 in [dynamic,output]
        ret(uc,len(items) if x0==dynamic else len(appended))
    elif name=='MCFArrayGetValueAtIndex':
        assert x0==dynamic and x1<len(items)
        ret(uc,0x10001000+x1*16)
    elif name=='MCFURLCreateWithString':
        i=(x0-0x10001000)//16
        assert 0<=i<len(items)
        ret(uc,0x10002000+i*16 if items[i] else 0)
    elif name=='MCFArrayAppendValue':
        assert x0==output;appended.append(x1);ret(uc,0)
    elif name=='MCFASCIIStringGetCString':
        assert x0==fallback_ascii;ret(uc,fallback_cstring)
    elif name=='MCFURLCreateWithCString':
        assert x0==fallback_cstring;ret(uc,fallback_url if fallback_valid else 0)
    elif name=='MCFDictionarySetValue':
        assert (x0,x1,x2)==(dictionary,task,output)
        stored=True;ret(uc,0)
    elif name=='MCFRelease':ret(uc,0)
    elif name=='abort':
        failures.append('native abort');uc.emu_stop()
    else:
        failures.append('unexpected import '+name);uc.emu_stop()
def interrupted(*args):
    failures.append('target interrupt');machine.emu_stop()
machine.hook_add(UC_HOOK_CODE,controlled)
machine.hook_add(UC_HOOK_INTR,interrupted)
results=[]
for name,present,validity,default_ok in [
 ('absent-provider-list',False,[],True),('empty-list',True,[],True),
 ('one-valid',True,[True],True),('all-invalid',True,[False,False],True),
 ('mixed-order',True,[True,False,True],True),('several-valid',True,[True,True,True],True),
 ('invalid-fallback',True,[True],False)]:
    items=validity;fallback_valid=default_ok;stored=False;appended.clear();failures.clear()
    machine.mem_write(manager+0x68,struct.pack('<Q',dictionary))
    machine.reg_write(UC_ARM64_REG_SP,0x200ff000)
    for r,v in [(UC_ARM64_REG_X19,manager),(UC_ARM64_REG_X21,task),(UC_ARM64_REG_X22,output),(UC_ARM64_REG_X23,dynamic if present else 0)]:machine.reg_write(r,v)
    machine.emu_start(0x727ef8,0x728024,timeout=1000000,count=100000)
    expected=[0x10002000+i*16 for i,valid in enumerate(items) if valid]
    if default_ok:
        assert not failures and stored and machine.reg_read(UC_ARM64_REG_PC)==0x728024,(name,failures)
        expected.append(fallback_url)
    else:
        assert failures==['native abort'] and not stored,(name,failures)
    assert appended==expected,(name,appended,expected)
    results.append({'case':name,'dynamic_parse_results':items,'fallback_parse_result':default_ok,
                    'appended_synthetic_urls':[hex(v) for v in appended],'stored':stored,'aborted':bool(failures)})
print(json.dumps({'scope':'synthetic endpoint append/store only; URL parsing, provider, strings and network not executed',
 'unicorn_version':'2.1.4','instruction_cap_per_case':100000,'timeout_microseconds_per_case':1000000,
 'cases':results},indent=2))
