"""Reproduce the full pinned framed-block experiment from repository root.
Run with the research Python environment containing unicorn==2.1.4.
Inputs/output are under ignored ../android-lab; no target network or filesystem APIs.
"""
from pathlib import Path
import json,hashlib,lzma,subprocess,zipfile,sys
apk=Path('../android-lab/apks/WhatsApp.1.apk')
assert hashlib.sha256(apk.read_bytes()).hexdigest()=='887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e'
with zipfile.ZipFile(apk) as z:c=z.read('lib/arm64-v8a/libs.so')
assert hashlib.sha256(c).hexdigest()=='1387f4a589266b39551c5ace0bd1cf11e4b658b341519e6bb38e70bfa48f33d5'
blocks=json.loads(Path('docs/protocol/apk/2.26.43.72-superpack-transform-blocks/blocks.json').read_text())['blocks']
out=Path('../android-lab/output/metrics-native/framed-decoded');out.mkdir(exist_ok=True)
results=[]
for block in blocks:
 i=block['ordinal'];path=out/f'{i:03d}.bin';start=0x870+int(block['payload_archive_offset'],16);payload=c[start:start+block['encoded_bytes']]
 assert hashlib.sha256(payload).hexdigest()==block['payload_sha256']
 if block['kind']==1:
  decoder=lzma.LZMADecompressor(format=lzma.FORMAT_XZ,memlimit=128*1024*1024)
  b=decoder.decompress(payload,max_length=block['declared_output_bytes']+1)
  assert decoder.eof and not decoder.unused_data and len(b)==block['declared_output_bytes'],i
  path.write_bytes(b);method='python-lzma-XZ'
 else:
  old=Path(f'../android-lab/output/metrics-native/block-{i:03d}-emulated.bin')
  if old.exists():
   prior=json.loads(Path('docs/protocol/apk/2.26.43.72-superpack-kind42-emulation/decoded-candidates.json').read_text())
   expected=next(x for x in prior if x['ordinal']==i)
   assert hashlib.sha256(old.read_bytes()).hexdigest()==expected['sha256']
   path.write_bytes(old.read_bytes());method='prior-bounded-Unicorn'
  else:
   p=subprocess.run([sys.executable,'docs/protocol/apk/2.26.43.72-superpack-kind42-emulation/emulate-block.py',str(apk),str(i),str(path)],capture_output=True,text=True)
   (out/f'{i:03d}.log').write_text(p.stdout+p.stderr)
   if p.returncode:print(f'FAILED {i}: '+p.stdout+p.stderr,flush=True);raise SystemExit(1)
   method='bounded-Unicorn'
  b=path.read_bytes();assert len(b)==block['declared_output_bytes']
 results.append({'ordinal':i,'kind':block['kind'],'method':method,'bytes':len(b),'sha256':hashlib.sha256(b).hexdigest(),'prefix_hex':b[:16].hex(),'starts_elf':b.startswith(b'\x7fELF')})
 (out/'results.json').write_text(json.dumps(results,indent=2)+'\n')
 if block['kind']==42 or i%25==0:print(f'Decoded {i+1}/213; kind {block["kind"]}; {len(b)} bytes',flush=True)
print('COMPLETE '+str(len(results)),flush=True)
