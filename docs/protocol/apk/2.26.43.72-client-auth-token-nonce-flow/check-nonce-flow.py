"""Digest-bound, method-specific conservative CFG check; not a general DEX analyzer.

Input is the reviewed xtask checked DEX export. Branch deltas are code units,
printed instruction addresses and switch targets are bytes. All protected
instructions get edges to every listed exception handler, even if they cannot
throw. Post-write definitions propagate on these extra edges, which overstates
possibilities for the three nonthrowing move definitions. This cannot establish
runtime reachability or trusted provenance. Assertions bind the reviewed writes,
source register stability, graph targets, reaching definition and call chain.
"""
from pathlib import Path
import re,collections,hashlib,sys
p=Path(sys.argv[1])
expected='df65a93664e6d6c9be99020e891993030df015fd0a2b230dc772c00ffed4d5ba'
if hashlib.sha256(p.read_bytes()).hexdigest()!=expected:
 raise SystemExit('Retained method digest differs from the reviewed export')
s=p.read_text(); ins={}
for line in s.splitlines():
 m=re.match(r'\s+([0-9a-f]+)\.\.([0-9a-f]+)\s+(\S+)(?:\s+(.*))?$',line)
 if m:
  a,b=int(m[1],16),int(m[2],16);ins[a]=(b,m[3],m[4] or '')
edges={};bad=[]
for a,(b,op,args) in ins.items():
 normal=[]
 if 'payload' in op: pass
 elif op.startswith(('return','throw')):pass
 elif op.startswith('goto'):
  normal=[a+2*int(args,16)]
 elif op.startswith('if-'):
  normal=[b,a+2*int(args.rsplit(', ',1)[1],16)]
 elif op in ('sparse-switch','packed-switch'):
  normal=[int(x,16) for x in re.findall(r':@0x([0-9a-f]+)',args)]+[b]
 else:normal=[b]
 edges[a]=set(normal)
 for t in normal:
  if t not in ins:bad.append((a,t))
for m in re.finditer(r'# catch .* code_units ([0-9a-f]+)\.\.([0-9a-f]+) -> ([0-9a-f]+)',s):
 lo,hi,t=[2*int(x,16) for x in m.groups()]
 for a in ins:
  if lo<=a<hi:edges[a].add(t)
assert len(ins)==2585 and not bad
assert all(t in ins for targets in edges.values() for t in targets)
print('instruction count',len(ins),'invalid normal targets',bad)
defs={a for a,(b,op,args) in ins.items() if args.startswith('v31,') and not op.startswith(('invoke','if-','iput','sput','aput'))}
assert defs=={0x134,0x1d28,0x287e}
# All predecessor-register writes are reviewed object moves, not wide writes.
assert [(hex(a),op) for a,(b,op,args) in ins.items() if args.startswith('v30,') and not op.startswith(('invoke','if-','iput','sput','aput'))]==[('0x70','move-object/from16'),('0x1d3c','move-object/from16'),('0x287a','move-object/from16')]
assert not any(args.startswith(('v68,','v69,')) and not op.startswith(('invoke','if-','iput','sput','aput')) for b,op,args in ins.values())
print('v31 definitions',[hex(a) for a in sorted(defs)])
q=collections.deque([(min(ins),'entry')]);seen=set(q);parents={}
while q:
 a,d=q.popleft();out=hex(a) if a in defs else d
 for t in edges[a]:
  st=(t,out)
  if st not in seen:seen.add(st);parents[st]=(a,d);q.append(st)
reaching=sorted(d for a,d in seen if a==0x2702)
assert reaching==['0x134']
print('definitions reaching2702',reaching)
for target,want in [(0x26fe,{0x26fa}),(0x2702,{0x26fe}),(0x2706,{0x2702})]:
 incoming={a for a,targets in edges.items() if target in targets}
 assert incoming==want
 print('predecessors',hex(target),[hex(a) for a in sorted(incoming)])
for definition in sorted(defs):
 q=collections.deque([definition]);seen2={definition}
 while q:
  a=q.popleft()
  for t in edges[a]:
   if t in defs and t!=definition:continue
   if t not in seen2:seen2.add(t);q.append(t)
 print(hex(definition),'can reach2702 before another definition',0x2702 in seen2)
