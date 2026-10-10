import json,itertools,random,subprocess,argparse
from pathlib import Path
parser=argparse.ArgumentParser(description='Reproduce synthetic GsB parse/serialization vectors with the independent Java oracle.')
parser.add_argument('--classpath',required=True,help='Pinned protobuf-javalite JAR plus directory containing compiled AdvDetailsProtoOracle, separated by colon.')
parser.add_argument('--output',type=Path,required=True)
args=parser.parse_args()
def var(n):
 b=[]
 while n>127:b.append((n&127)|128);n>>=7
 return bytes(b+[n])
cases=set([b'',b'\x08',b'\x00',b'\x80'*10,b'\x08'+b'\xff'*9+b'\x7f',b'\x08'+b'\x80'*10,b'\x32'+var(0xffffffff)])
fields=[b'\x08\x07',b'\x10\x09',b'\x18\x02',b'\x20\x00',b'\x28\x00']
for p in itertools.permutations(fields):cases.add(b''.join(p))
base=b'\x08\x07\x10\x09\x18\x02'
values=[0,1,2,3,127,128,255,0xffffffff,0x100000000,0x100000001,0x7fffffffffffffff,0xffffffffffffffff]
for tag in [8,16,24,32,40,48]:
 for n in values:
  field=bytes([tag])+var(n)
  for prefix in [b'',base,base+field]:cases.add(prefix+field)
  if n<128:cases.add(base+bytes([tag])+bytes([n|128,0]))
unknowns=[b'\x30\x81\x00',b'\x39'+bytes(range(8)),b'\x42\x03abc',b'\x4b\x08\x87\x00\x4c',b'\x55'+bytes(range(4)),b'\x4c',b'\x4b\x54',b'\x4b',b'\x42\x05abc',b'\x0a\x01x']
for u in unknowns:
 for p in [b'',base,base+u]:cases.add(p+u)
 for p in [b'',base]:cases.add(u+p)
for depth in [1,2,98,99,100]:cases.add(base+b'\x4b'*depth+b'\x08\x07'+b'\x4c'*depth)
rng=random.Random(3771)
for _ in range(250):
 chunks=[rng.choice(fields+unknowns+[bytes([rng.choice([8,16,24,32,40])])+var(rng.choice(values))]) for j in range(rng.randrange(1,9))]
 cases.add(b''.join(chunks))
ordered=sorted(cases)
r=subprocess.run(['java','-cp',args.classpath,'AdvDetailsProtoOracle'],input=''.join(x.hex()+'\n' for x in ordered),capture_output=True,text=True,check=True)
lines=r.stdout.splitlines();assert len(lines)==len(ordered)
rows=[]
for value,line in zip(ordered,lines):
 row={'hex':value.hex()}
 if line=='malformed':row['serialized']=None;row['fields']=None
 else:
  parts=line.split('|');row['serialized']=parts[0];row['fields']=[None if x=='null' else int(x) for x in parts[1:]]
 rows.append(row)
obj={'apk_build':'2.26.37.71','source':'GsB.dynamicMethod plus IQg/Azc enum 0..2','oracle':'desktop protobuf-javalite 4.36.1, not Android execution','jar_sha256':'91d3dba2521322103230c509d41a017da3a5e6766610adcb10c5bb127cbb26c5','cases':rows}
args.output.write_text(json.dumps(obj,indent=2)+'\n')
print(len(rows),len(json.dumps(obj)))
