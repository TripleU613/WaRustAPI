"""Fault checks for the bounded proof model, not Android behavior tests.
Forwards --apk/--xtask or --exports-dir arguments to derive-defaults.py.
"""
import contextlib
import io
import json
from pathlib import Path
import runpy
import sys

script = Path(__file__).with_name("derive-defaults.py")
sys.argv = [str(script), *sys.argv[1:]]
with contextlib.redirect_stdout(io.StringIO()):
    ns = runpy.run_path(str(script))
sources = ns["sources"]
original = dict(sources)
mutations = [
    ("control_flow", "A0I", lambda s: s.replace(".end method", "    ffff..10003  if-eqz  v0, +0x1\n.end method")),
    ("unresolved_key", "A0I", lambda s: s.replace("const/16  v0, #0x3b4f", 'const-string  v0, "unresolved"')),
    ("builder_escape", "<init>", lambda s: s.replace("LX/08Y;->A0Q(", "LX/08Y;->A0Y(")),
    ("wrong_owner", "<init>", lambda s: s.replace("move-object/from16  v0, v302", "move-object/from16  v0, v301")),
]
results = []
for name, method, mutate in mutations:
    sources[method] = mutate(original[method])
    if sources[method] == original[method]:
        raise SystemExit("mutation anchor changed")
    try:
        ns["Proof"]().run("<init>")
    except ValueError as error:
        results.append({"mutation": name, "rejected": True, "reason": str(error)})
    else:
        raise SystemExit("guard did not reject " + name)
    finally:
        sources[method] = original[method]
print(json.dumps(results, indent=2))
