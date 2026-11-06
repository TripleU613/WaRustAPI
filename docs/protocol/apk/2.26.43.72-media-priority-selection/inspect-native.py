"""Verify the scoped native priority setter window, without target execution."""
import contextlib,hashlib,io,json,runpy
from pathlib import Path
with contextlib.redirect_stdout(io.StringIO()):
    e=runpy.run_path(str(Path(__file__).resolve().parent.parent/'2.26.43.72-media-body-provider/inspect-native-tuples.py'))
a,z=0x663b10,0x663b2c
o=e['off'](a,z-a);data=e['data']
print(json.dumps(dict(library_sha256=hashlib.sha256(data).hexdigest(),start_va=hex(a),end_va=hex(z),file_offset=hex(o),bytes=z-a,sha256=hashlib.sha256(data[o:o+z-a]).hexdigest(),scope='Static native setter window, not downstream serialization'),indent=2))
