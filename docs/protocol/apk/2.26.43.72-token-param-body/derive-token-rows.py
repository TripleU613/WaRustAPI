"""Pinned selected DEX payload and packaged parameter rows; no APK execution."""
import argparse, hashlib, json, struct, zipfile
from pathlib import Path

parser=argparse.ArgumentParser(description=__doc__)
parser.add_argument('--apk',type=Path,required=True)
args=parser.parse_args()
h=hashlib.sha256()
with args.apk.open('rb') as f:
    for chunk in iter(lambda:f.read(1024*1024),b''):h.update(chunk)
if h.hexdigest()!='887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e':raise ValueError('APK mismatch')
base=Path(__file__).parent
specifiers=json.loads((base.parent/'2.26.43.72-token-mobileconfig-binding/specifiers.json').read_text())['selected']
headers=json.loads((base.parent/'2.26.43.72-token-param-header/asset-headers.json').read_text())['selected']
def uleb(data,pos):
    value=0
    for shift in range(0,35,7):
        byte=data[pos];pos+=1;value|=(byte&127)<<shift
        if byte<128:return value,pos
    raise ValueError('ULEB overflow')
def method_code(data):
    u32=lambda p:struct.unpack_from('<I',data,p)[0]
    strings=u32(0x3c);types=u32(0x44);methods=u32(0x5c)
    def string(i):
        pos=u32(strings+4*i);_,pos=uleb(data,pos)
        return data[pos:data.index(0,pos)].decode('utf-8',errors='replace')
    def typename(i):return string(u32(types+4*i))
    for pos in range(u32(0x64),u32(0x64)+32*u32(0x60),32):
        if typename(u32(pos))!='LX/09o;':continue
        ptr=u32(pos+24);sizes=[]
        for _ in range(4):v,ptr=uleb(data,ptr);sizes.append(v)
        for _ in range(sizes[0]+sizes[1]):
            _,ptr=uleb(data,ptr);_,ptr=uleb(data,ptr)
        found=[]
        for count in sizes[2:]:
            index=0
            for _ in range(count):
                delta,ptr=uleb(data,ptr);index+=delta
                access,ptr=uleb(data,ptr);code,ptr=uleb(data,ptr)
                if string(u32(methods+8*index+4))=='<clinit>':found.append((access,code))
        if len(found)!=1:raise ValueError('initializer coverage changed')
        return found[0]
    return None
with zipfile.ZipFile(args.apk) as apk:
    matches=[]
    for name in apk.namelist():
        if name.startswith('classes') and name.endswith('.dex'):
            data=apk.read(name);found=method_code(data)
            if found:matches.append((name,data,*found))
    if len(matches)!=1:raise ValueError('class coverage changed')
    dex,data,access,code=matches[0];start=code+16
    # Retained09o initializer binds the first5000-entry payload into the first
    # concatenated segment of unit2's config-key map, used for these four indices.
    if data[start+0x4e:start+0x54]!=struct.pack('<Hi',0x0826,(0xd00-0x4e)//2):raise ValueError('payload reference changed')
    payload=start+0xd00
    if struct.unpack_from('<HHI',data,payload)!=(0x300,4,5000):raise ValueError('payload identity changed')
    keys=struct.unpack_from('<5000i',data,payload+8)
    assets=[];selected={}
    for head in headers:
        raw=apk.read(head['path'])
        if hashlib.sha256(raw).hexdigest()!=head['sha256']:raise ValueError('asset changed')
        body=raw[head['header_bytes']:];count=head['PHU_A02'];partition=head['PHU_A01']
        if len(body)!=count*8 or not 0<=partition<=count:raise ValueError('body/partition coverage changed')
        rows=list(struct.iter_unpack('<II',body))
        order=[(b>>12,b&4095) for a,b in rows]
        if order[:partition]!=sorted(order[:partition]) or order[partition:]!=sorted(order[partition:]):raise ValueError('binary-search partition unsorted')
        if len({b>>12 for a,b in rows})!=head['PHU_A00'] or len({b for a,b in rows})!=count:raise ValueError('config/parameter uniqueness changed')
        assets.append({'path':head['path'],'distinct_config_keys':len({b>>12 for a,b in rows}),'pairs':count,'first_partition_pairs':partition,'body_sha256':hashlib.sha256(body).hexdigest()})
        if head['path'].endswith('_u2.txt'):
            for prop,entry in specifiers.items():
                spec=int(entry['specifier_hex'],16);config=(spec>>32)&65535;param=(spec>>16)&65535
                if (spec>>54)&63!=2 or config>=5000:raise ValueError('selected unit/index changed')
                key=keys[config]
                hits=[(i,a,b) for i,(a,b) in enumerate(rows) if (b>>12,b&4095)==(key,param)]
                if len(hits)!=1:raise ValueError('selected row missing/ambiguous')
                i,a,b=hits[0];signed=a if a<2**31 else a-2**32
                slot=signed>>16
                selected[prop]={'specifier_hex':hex(spec),'config_index':config,'config_key':key,'parameter_id':param,'pair_index':i,'pair_hex':[f'0x{a:08x}',f'0x{b:08x}'],'translated_slot_signed':slot,'original_slot':spec&65535}
    print(json.dumps({'kind':'pinned packaged translation rows; not effective values','dex':dex,'dex_sha256':hashlib.sha256(data).hexdigest(),'initializer_code_offset':code,'initializer_access_flags':access,'selected_key_payload_sha256':hashlib.sha256(data[payload+8:payload+8+20000]).hexdigest(),'assets':assets,'selected':selected},indent=2))
