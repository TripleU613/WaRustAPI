"""Execute pinned 43.72 pre-ACK SQL against synthetic in-memory rows only."""
import json
from pathlib import Path
import sqlite3
import sys

ROOT = Path(__file__).resolve().parents[3]
FIXTURE = ROOT / "crates/whatsapp-api/fixtures/prekeys/2.26.43.72-preack-sql-reference.json"
PAGE = "SELECT _id, ptn FROM preacks WHERE _id > ? ORDER BY _id LIMIT ?"
PRUNE = """DELETE FROM preacks WHERE _id IN (
    SELECT _id FROM preacks
    WHERE _id > (SELECT _id FROM preacks ORDER BY _id LIMIT 1 OFFSET ?)
    ORDER BY _id DESC LIMIT ?
)"""


def database(ids):
    db = sqlite3.connect(":memory:")
    db.execute("CREATE TABLE preacks (_id INTEGER PRIMARY KEY, ptn BLOB NOT NULL)")
    db.executemany("INSERT INTO preacks VALUES (?, ?)", ((i, b"synthetic") for i in ids))
    return db


def reference():
    # Deliberate row-ID gaps distinguish row watermarks from counts/positions.
    ids = [7 + 3 * i for i in range(1205)]
    db = database(ids)
    page = [r[0] for r in db.execute(PAGE, (ids[199], 3))]
    assert page == ids[200:203]
    deleted = db.execute("DELETE FROM preacks WHERE _id <= ?", (ids[16],)).rowcount
    remaining = [r[0] for r in db.execute("SELECT _id FROM preacks ORDER BY _id")]
    assert deleted == 17 and remaining == ids[17:]
    db.close()

    db = database(ids)
    passes = []
    while True:
        count = db.execute(PRUNE, (200 - 1, 500)).rowcount
        if count == 0:
            break
        passes.append(count)
        assert len(passes) <= 3
    kept = [r[0] for r in db.execute("SELECT _id FROM preacks ORDER BY _id")]
    assert passes == [500, 500, 5] and kept == ids[:200]
    db.close()
    return {
        "scope": "synthetic SQLite semantics only; not native queue/callback or node-codec parity",
        "source": "APK 2.26.43.72 1iY.A02/A03 and 2JI.run selector42",
        "row_ids": {"count": len(ids), "first": ids[0], "stride": 3},
        "strict_after_page": page,
        "retirement": {"watermark": ids[16], "deleted": deleted, "next": remaining[0]},
        "pruning": {"cap": 200, "limit_per_delete": 500, "deleted_per_pass": passes,
                    "kept": len(kept), "first": kept[0], "last": kept[-1]},
    }


if __name__ == "__main__":
    result = reference()
    if sys.argv[1:] == ["--check"]:
        assert result == json.loads(FIXTURE.read_text()), "pre-ACK SQL fixture differs"
        print("Pre-ACK SQL fixture matches synthetic SQLite execution.")
    elif not sys.argv[1:]:
        print(json.dumps(result, indent=2))
    else:
        raise SystemExit("usage: preack_sql_reference.py [--check]")
