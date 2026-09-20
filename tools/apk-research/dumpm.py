#!/usr/bin/env python3
"""dumpm.py DUMP ANCHOR [ANCHOR...] [--max-methods N] [--max-lines N] [--full]
Print each method whose body contains a const-string exactly equal to an anchor
(compact view: const-strings, invokes, new-instance, consts)."""
import sys,re
args=sys.argv[1:]; path=args[0]; anchors=[]; mm=4; ml=90; full=False
i=1
while i<len(args):
    a=args[i]
    if a=='--max-methods': mm=int(args[i+1]); i+=2
    elif a=='--max-lines': ml=int(args[i+1]); i+=2
    elif a=='--full': full=True; i+=1
    else: anchors.append(a); i+=1
want={f'"{a}"' for a in anchors}
cls=None; buf=None; hit=None; found={a:0 for a in anchors}
def flush():
    global buf,hit
    if buf and hit and found[hit]<mm:
        found[hit]+=1
        print(f"===== {cls} {buf[0].strip()}  [anchor {hit}]")
        body=[l for l in buf[1:] if full or any(t in l for t in ('const-string','invoke','new-instance','const/','iput','sput','return'))]
        for l in body[:ml]: print("  "+l.rstrip()[:180])
        if len(body)>ml: print(f"  ... {len(body)-ml} more")
    buf=None; hit=None
with open(path,encoding='utf-8',errors='replace') as fh:
    for line in fh:
        if line.startswith('# class:'): flush(); cls=line.strip()[9:-1]; continue
        if line.startswith('.method'): flush(); buf=[line]; continue
        if buf is None: continue
        buf.append(line)
        if 'const-string' in line and hit is None:
            m=re.search(r'const-string(?:/jumbo)?\s+v\d+, (".*")\s*$', line.rstrip('\n'))
            if m and m.group(1) in want: hit=m.group(1).strip('"')
        if line.startswith('.end method'): flush()
for a in anchors:
    if found[a]==0: print(f"===== no method contains {a!r}")
