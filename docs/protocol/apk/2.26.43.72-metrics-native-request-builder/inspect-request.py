"""Resolve static request literals in the digest-pinned recovered native image."""
import contextlib, io, json, runpy, struct
from pathlib import Path
with contextlib.redirect_stdout(io.StringIO()):
    e = runpy.run_path(str(Path(__file__).resolve().parent.parent / '2.26.43.72-metrics-native-event-routing/resolve-jni.py'))
data, offset, relative = e['data'], e['file_offset'], e['relocations']
def raw(address, size):
    start = offset(address,size)
    return data[start:start+size]
def string(address):
    start = offset(address)
    end = data.index(0,start)
    return raw(address,end-start).decode('ascii')
strings = [{'va':hex(a),'value':string(a)} for a in [0x10c3ed,0xa9693,0xa01be,0x9ca2d,0xc3761,0x167252,0x16a166,0xe8d21,0x890d4,0x890e8,0x89110,0xcab15]]
descriptors=[]
for slot in range(0xc0aba0,0xc0abf0,16):
    address=relative[slot]
    length=struct.unpack('<Q',raw(slot+8,8))[0]
    descriptors.append({'slot_va':hex(slot),'literal_va':hex(address),'length':length,'value':raw(address,length).decode('ascii')})
fixed=[{'va':hex(a),'length':n,'value':raw(a,n).decode('ascii')} for a,n in [(0x2fd7cc,64),(0x2fd80d,5),(0x2fd813,156),(0x2fd8b0,84)]]
method=[]
for section in e['sections']:
    if section[1] != 4: continue
    for pos in range(section[4],section[4]+section[5],24):
        address,info,addend=struct.unpack_from('<QQq',data,pos)
        if address != 0xc27d98: continue
        symbols=e['sections'][section[6]]
        symbol=struct.unpack_from('<IBBHQQ',data,symbols[4]+24*(info>>32))
        names=e['sections'][symbols[6]]
        start=names[4]+symbol[0]
        name=data[start:data.index(0,start)].decode('ascii')
        assert info & 0xffffffff == 1025 and addend == 0
        assert name == 'kMCIURLRequestHTTPMethodPOST'
        method.append({'got_va':hex(address),'relocation_type':1025,'symbol':name})
assert len(method)==1
print(json.dumps({'scope':'static literals and relocated method symbol; no request executed','method':method,'strings':strings,'descriptors':descriptors,'fixed_body_slices':fixed},indent=2))
