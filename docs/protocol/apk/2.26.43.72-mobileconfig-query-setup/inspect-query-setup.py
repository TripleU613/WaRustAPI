"""Pinned MobileConfig query setup strings and selected call bindings, offline only."""
import contextlib,io,json,runpy,struct
from pathlib import Path
base=Path(__file__).resolve().parent
with contextlib.redirect_stdout(io.StringIO()):
 elf=runpy.run_path(str(base.parent/'2.26.43.72-mobileconfig-native-completion/inspect-binding.py'))
b,offset,cstring=[elf[k] for k in ('b','offset','cstring')]
expected={0x17ede:'BadRequesterSetup',0x16f77:'FBMobileConfigRequester::getQueryParams',0x1ac1d:'empty deviceId',0x19fcf:'no parameter schemas available',0x18e9e:'query_hash',0x18e82:'one_query_hash',0x18ebd:'global_value_hash',0x18e91:'request_data_query_hash',0x1a716:'ep_refresh_id',0x16260:'bool_opt_policy',0x17137:'with_names',0x196bf:'true',0x186aa:'param_list_checksum',0x1717b:'value_hashes'}
for va,value in expected.items():
 if cstring(va)!=value:raise ValueError(f'string mismatch at {va:x}')
calls={0xac0cc:0xca4a8,0xac15c:0xcade4,0xac1ec:0xaa704,0xac1f8:0xaa89c,0xac668:0xae330,0xac6f4:0xcd090,0xac704:0x7cc24,0xac750:0xae2e4,0xac598:0xaa8f8,0xac5b4:0xae37c,0xacb2c:0xcd1f0,0xacb48:0xae2b0,0xac4b4:0xad9a0,0xac4c0:0xada04}
for site,target in calls.items():
 word=struct.unpack_from('<I',b,offset(site))[0];imm=word&0x3ffffff
 if imm&(1<<25):imm-=1<<26
 if word>>26!=0b100101 or site+4*imm!=target:raise ValueError(f'call mismatch at {site:x}')
print(json.dumps({'member_sha256':elf['digest'],'strings':{hex(k):v for k,v in expected.items()},'selected_calls':{hex(k):hex(v) for k,v in calls.items()},'producer_entry':'0xac09c','early_empty_results':['requester deviceId string at +0x28 empty','request-data mode +0x9c != 1 and both selected schema-container +0x20 words zero'],'selected_metadata_sources':{'global_value_hash':'nonempty request-data string +0xa8','one_query_hash':'nonempty requester configuration string +0x160','request_data_query_hash':'checksum getter when request-data byte +0x29 == 1','ep_refresh_id':'signed request-data int +0xc0 >= 1, decimal string','bool_opt_policy':'decimal raw configuration byte +0x194 OR (2 if byte +0x208 != 0 else 0)'},'scope':'selected native setup/metadata gates; complete query/hash algorithms, actual values and live acceptance remain unresolved'},indent=2))
