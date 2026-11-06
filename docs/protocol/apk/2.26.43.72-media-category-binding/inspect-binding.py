"""Decode actual AArch64 load/store operands for category/purpose binding."""
import contextlib,hashlib,io,json,runpy,struct
from pathlib import Path
with contextlib.redirect_stdout(io.StringIO()):
    e=runpy.run_path(str(Path(__file__).resolve().parent.parent/'2.26.43.72-media-body-provider/inspect-native-tuples.py'))
data,off,rel,string=(e[k] for k in ('data','off','rel','string'))
def unsigned_word_transfer(address,load,register,base,displacement):
    word=struct.unpack_from('<I',data,off(address,4))[0]
    assert word&0xffc00000==(0xb9400000 if load else 0xb9000000)
    got=(word&31,(word>>5)&31,((word>>10)&0xfff)*4)
    assert got==(register,base,displacement),(hex(address),got)
    return dict(va=hex(address),instruction_word=hex(word),operation='LDR W' if load else 'STR W',register=register,base_register=base,displacement=hex(displacement))
transfers=[unsigned_word_transfer(0x651c44,True,21,0,0),unsigned_word_transfer(0x651c58,True,27,0,0),unsigned_word_transfer(0x651de0,False,21,31,0x260),unsigned_word_transfer(0x651de4,False,27,31,0x264)]
assert rel[0x778d10+0x10]==0x6548ec
reader_name=string(rel[rel[0x778d10-8]+8]);assert 'Deserializer' in reader_name
windows=[]
for name,(a,z) in json.loads((Path(__file__).parent/'native-windows.json').read_text()).items():
    a,z=int(a,16),int(z,16);o=off(a,z-a)
    windows.append(dict(name=name,start_va=hex(a),end_va=hex(z),file_offset=hex(o),bytes=z-a,sha256=hashlib.sha256(data[o:o+z-a]).hexdigest()))
print(json.dumps(dict(library_sha256=hashlib.sha256(data).hexdigest(),reader_type_name=reader_name,transfers=transfers,native_request_base_from_sp='0x80',fields={'category':hex(0x260-0x80),'purpose':hex(0x264-0x80)},standard_upload_serialized_pair=struct.pack('<II',8,0).hex(),windows=windows,scope='Static selected normal decode/store path; not whole-method execution or wire/account proof'),indent=2))
