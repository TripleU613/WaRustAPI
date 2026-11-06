"""Bounded direct-call/nearby-immediate discovery, never an exhaustive value flow."""
import collections,json,re,sys
from pathlib import Path
lines=Path(sys.argv[1]).read_text().splitlines()
getters=[]
for i,line in enumerate(lines):
    if '\tbl\t0x3ef220 ' in line:
        getters.append({'call':line.strip(),'context':lines[max(0,i-4):i+2]})
w1=collections.defaultdict(list)
for i,line in enumerate(lines):
    if not re.search(r'\tmov\tw1, #0x3\s',line):continue
    for following in lines[i+1:i+9]:
        match=re.search(r'\tbl\t(0x[0-9a-f]+) ',following)
        if match:
            w1[match[1]].append(line.split(':')[0].strip());break
w2=[];group3=[]
for i,line in enumerate(lines):
    match=re.match(r'\s+([0-9a-f]+):.*\tmov\tw2, #0x3\s',line)
    if match and 0x4f0000<=int(match[1],16)<0x530000:
        w2.append(lines[max(0,i-3):i+7])
    if '\tbl\t0x668084 ' in line and any(re.search(r'\tmov\tw1, #0x3\s',q) for q in lines[max(0,i-12):i]):
        group3.append(lines[max(0,i-12):i+2])
print(json.dumps({'scope':'Direct numeric BL and bounded nearby immediate patterns only; no producer or absence proof','integer_getter_direct_calls':getters,'w1_3_first_bl_within8':dict(sorted(w1.items())),'w2_3_in_4f0000_to_530000':w2,'group_setter_with_w1_3_within12':group3},indent=2))
