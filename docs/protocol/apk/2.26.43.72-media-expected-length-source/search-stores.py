"""Reproduce exact physical operand censuses; not a field-alias analysis."""
import contextlib,io,re,runpy,subprocess,sys
from pathlib import Path
with contextlib.redirect_stdout(io.StringIO()):
 runpy.run_path('docs/protocol/apk/2.26.43.72-media-body-provider/inspect-native-tuples.py')
lines=subprocess.run(['llvm-objdump','-d','--no-show-raw-insn',sys.argv[1]],check=True,capture_output=True,text=True).stdout.splitlines()
exact=[];near=[]
for line in lines:
 m=re.match(r'\s*([0-9a-f]+):\s+(.*)',line)
 if not m:continue
 address=int(m[1],16);instruction=m[2]
 if re.search(r'\b(str|stp|stur)\b.*#0x74\]',instruction):exact.append(line)
 if 0x530000<=address<0x540000 and re.search(r'\b(str|stp|stur)\b.*#0x(6c|70|74|78)\]',instruction):near.append(line)
out=Path(sys.argv[2]);out.mkdir(parents=True,exist_ok=True)
for name,rows in [('exact-store-census',exact),('near-store-census',near)]:
 (out/(name+'.txt')).write_text('\n'.join(rows)+'\n')
print(f'exact={len(exact)} near={len(near)}; physical operand matches only')
