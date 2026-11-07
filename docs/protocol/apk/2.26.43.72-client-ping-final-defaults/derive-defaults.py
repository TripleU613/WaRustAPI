"""Pinned linear DEX proof for the 08Y integer-default builder only.
No APK execution. Input exports must include signatures and exception metadata.
"""
import argparse, hashlib, json, re, subprocess
from pathlib import Path

UNKNOWN = ("unknown",)
INPUTS = {"<init>": "constructor", "A0B": "A0B", "A0I": "A0I", "A0Q": "integers"}
TARGETS = (15103, 15183, 15270)
PINNED_APK = "887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e"
EXPECTED_EXPORTS = {'<init>': '009d605b135aedae3c507b09f88bafd2f847eab8bbb7db29f2785e6d014b138d', 'A0B': '105798ee6f76957aed0e6e515f9ac6ddbe778dcc3e48c549a2881662d54f3380', 'A0I': '7c5a0b305e3ec8dd63723621c46ddaafc894b16cb1ebf092f0051465d905ccf3', 'A0Q': '1aabc3db01f657440201c00ebedaebaaf17a235221b0ec3782cfb469d986fb8a'}
parser = argparse.ArgumentParser(description=__doc__)
group = parser.add_mutually_exclusive_group(required=True)
group.add_argument("--apk", type=Path)
group.add_argument("--exports-dir", type=Path)
parser.add_argument("--xtask", type=Path, default=Path("target/debug/xtask"))
parser.add_argument("--policy", choices=("client-ping", "token-buckets"), default="client-ping")
args = parser.parse_args()
if args.policy == "token-buckets":
    TARGETS = (865, 909, 996, 997)
if args.apk:
    digest = hashlib.sha256()
    with args.apk.open("rb") as apk:
        for block in iter(lambda: apk.read(1024 * 1024), b""):
            digest.update(block)
    if digest.hexdigest() != PINNED_APK:
        raise SystemExit("pinned APK mismatch")
    sources = {m: subprocess.check_output([
        str(args.xtask.resolve()), "apk-smali", str(args.apk),
        "--class", "LX/08Y;", "--method", m, "--signatures", "--exceptions"
    ], text=True, stderr=subprocess.PIPE) for m in INPUTS}
else:
    sources = {m: (args.exports_dir / ("wa-ping-default-"+n+".txt")).read_text()
               for m,n in INPUTS.items()}
if {m: hashlib.sha256(s.encode()).hexdigest() for m,s in sources.items()} != EXPECTED_EXPORTS:
    raise SystemExit("pinned export identity changed")

class Proof:
    def __init__(self):
        self.builders = 0
        self.events = []
        self.target_builder = None
        self.field_builder = None
        self.invocations = []
        self.puts = 0
        self.target_built = False
    def run(self, method, argument=None):
        text = sources[method]
        if text.count('.method ') != 1 or '# exception table: 0 try range(s)' not in text:
            raise ValueError('method/exception coverage changed')
        size = int(re.search(r'\.registers (\d+)',text)[1])
        regs = {size-1: argument if argument is not None else ("this",)}
        pending = UNKNOWN
        for line in text.splitlines():
            match = re.match(r'\s+([0-9a-f]+)\.\.([0-9a-f]+)\s+(\S+)(?:\s+(.*))?$',line)
            if not match: continue
            pc,end,op,args = match.groups(); args=args or ''
            if not op.startswith(('move-result', 'invoke-')):
                pending = UNKNOWN
            if op.startswith('const'):
                dest=int(re.match(r'v(\d+),',args)[1])
                if 'string' in op: regs[dest]=UNKNOWN
                else:
                    val=int(re.search(r'#0x([0-9a-f]+)',args)[1],16)
                    decimal=re.search(r'\((-?\d+)\)',args)
                    if decimal: val=int(decimal[1])
                    elif op in ('const','const/high16') and val >= 2**31: val-=2**32
                    elif op=='const/16' and val>=2**15: val-=2**16
                    regs[dest]=('int',val)
            elif op.startswith('move-result'):
                regs[int(re.match(r'v(\d+)',args)[1])]=pending; pending=UNKNOWN
            elif op.startswith('move-object'):
                dest,src=map(int,re.findall(r'v(\d+)',args)); regs[dest]=regs.get(src,UNKNOWN)
            elif op=='check-cast': pass
            elif op.startswith('invoke-'):
                regtext,target=args.split('}, ',1); indices=list(map(int,re.findall(r'v(\d+)',regtext)))
                if '/range' in op: indices=list(range(indices[0],indices[-1]+1))
                values=[regs.get(i,UNKNOWN) for i in indices]; pending=UNKNOWN
                if target=='Ljava/lang/Integer;->valueOf(I)Ljava/lang/Integer;':
                    if values[0][0]!='int': raise ValueError('unknown boxed integer')
                    pending=('boxed',values[0][1])
                elif target=='Lcom/google/common/collect/ImmutableMap;->builder()Lcom/google/common/collect/ImmutableMap$Builder;':
                    pending=('builder',self.builders); self.builders+=1
                    if self.builders==2: self.target_builder=pending[1]
                elif target=='Lcom/google/common/collect/ImmutableMap$Builder;->put(Ljava/lang/Object;Ljava/lang/Object;)Lcom/google/common/collect/ImmutableMap$Builder;':
                    self.puts+=1
                    builder,key,value=values
                    if builder[0]!='builder': raise ValueError('unknown put receiver')
                    if builder[1]==self.target_builder:
                        if self.target_built: raise ValueError('write after integer map snapshot')
                        if key[0]!='boxed' or value[0]!='boxed': raise ValueError('unresolved integer map write')
                        self.events.append({'method':method,'pc':pc,'key':key[1],'value':value[1]})
                    pending=builder
                elif target=='Lcom/google/common/collect/ImmutableMap$Builder;->buildKeepingLast()Lcom/google/common/collect/ImmutableMap;':
                    if values[0][0]!='builder': raise ValueError('unknown build receiver')
                    pending=('snapshot',values[0][1])
                    if values[0][1]==self.target_builder: self.target_built=True
                elif target.startswith('LX/08Y;->') and '(Lcom/google/common/collect/ImmutableMap$Builder;)V' in target:
                    called=target.split('->')[1].split('(')[0]
                    if values[0][0]!='builder': raise ValueError('unknown helper receiver')
                    self.invocations.append({'method':called,'pc':pc,'builder':values[0][1]})
                    if values[0][1]==self.target_builder:
                        if called not in ('A0B','A0I','A0Q'): raise ValueError('integer builder escapes to unknown helper')
                        self.run(called,values[0])
                else:
                    if any(v[0] in ('builder', 'snapshot') and v[1]==self.target_builder for v in values) and target!='LX/00l;->A05(Ljava/lang/Object;)V':
                        raise ValueError('integer builder passed to unsupported invocation')
            elif op=='iput-object':
                src,owner=map(int,re.findall(r'v(\d+)',args.split(', LX/')[0])); value=regs.get(src,UNKNOWN)
                if value[0]=='builder': raise ValueError('builder escapes into field')
                if 'LX/08Y;->A02:Lcom/google/common/collect/ImmutableMap;' in args:
                    if value != ('snapshot',self.target_builder) or regs.get(owner)!=('this',): raise ValueError('wrong final integer map/owner')
                    self.field_builder=value[1]
            elif op=='return-void': pass
            else: raise ValueError('unsupported instruction: '+op)
proof=Proof();proof.run('<init>')
if proof.builders!=5 or proof.target_builder!=1 or proof.field_builder!=1:
    raise ValueError('builder binding changed')
if proof.puts!=5551: raise ValueError('put coverage changed')
selected={str(k):[row for row in proof.events if row['key']==k] for k in TARGETS}
if any(len(rows)!=1 for rows in selected.values()): raise ValueError('selected key is missing or overwritten')
expected = ({15103:32000,15183:240000,15270:15000} if args.policy == "client-ping"
            else {865:604800,909:4,996:604800,997:4})
if any(rows[0]['value']!=expected[int(k)] for k,rows in selected.items()): raise ValueError('default values changed')
print(json.dumps({'source_sha256':{m:hashlib.sha256(s.encode()).hexdigest() for m,s in sources.items()},
                  'all_visited_puts':proof.puts,'integer_puts':len(proof.events),
                  'distinct_integer_keys':len({e['key'] for e in proof.events}),
                  'constructor_builder_calls':proof.invocations,
                  'selected':selected},indent=2))
