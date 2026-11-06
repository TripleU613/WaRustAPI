"""Retain direct-call census; snippets do not establish complete key value flow."""
import json,sys
from pathlib import Path
result={}
for name,path in zip(['libwhatsappmerged.so','libffsingletonmerged.so','libmobileconfig-jni.so'],sys.argv[1:]):
    lines=Path(path).read_text().splitlines();rows={}
    if name=='libwhatsappmerged.so':
        for target in ['0x667d94','0x667d8c','0x667ba8','0x507c9c']:
            rows[target]=[dict(call=line.strip(),context=lines[max(0,i-6):i+1]) for i,line in enumerate(lines) if '\tbl\t'+target+' ' in line or '\tb\t'+target+' ' in line]
    else:
        for marker in ['TigonPropertyContainer11setProperty','TigonPropertyContainer5Group11setProperty','TigonPropertyContainer16setPropertyGroup']:
            rows[marker]=[dict(call=line.strip(),context=lines[max(0,i-8):i+1]) for i,line in enumerate(lines) if marker in line and ('\tbl\t' in line or '\tb\t' in line)]
    result[name]=rows
print(json.dumps(dict(scope='Direct numeric B/BL or symbol-labelled PLT branches only; no inlining, indirect, outlined-key, dynamic-loading or runtime absence proof',libraries=result),indent=2))
