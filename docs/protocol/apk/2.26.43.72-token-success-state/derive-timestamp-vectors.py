"""Replay the selected 0ht.A0V SQL statements against synthetic SQLite rows.

No account data, APK execution or network. Captured time is request data.
"""
import json
import sqlite3

captured = 1800000000
cases = [None, (captured-1, 0), (captured, 0), (captured+1, 0),
         (captured+1, None), (captured+1, captured+1),
         (captured+1, captured), (captured+1, -1), (captured-1, captured+100)]
vectors = []
for before in cases:
    with sqlite3.connect(":memory:") as db:
        db.execute("CREATE TABLE wa_trusted_contacts_send (jid TEXT UNIQUE, sent_tc_token_timestamp INTEGER, real_issue_timestamp INTEGER)")
        if before is not None:
            db.execute("INSERT INTO wa_trusted_contacts_send VALUES ('synthetic-user',?,?)", before)
        changed = db.execute("UPDATE wa_trusted_contacts_send SET sent_tc_token_timestamp=?, real_issue_timestamp=NULL WHERE jid=? AND sent_tc_token_timestamp <= ?", (captured, 'synthetic-user', captured)).rowcount
        if changed:
            effect = 'scheduled_timestamp_confirmed'
        else:
            changed = db.execute("UPDATE wa_trusted_contacts_send SET real_issue_timestamp=? WHERE jid=? AND real_issue_timestamp IS NOT NULL AND real_issue_timestamp <= ?", (captured, 'synthetic-user', captured)).rowcount
            if changed:
                effect = 'earlier_issue_timestamp_advanced'
            else:
                changed = db.execute("INSERT OR IGNORE INTO wa_trusted_contacts_send (jid,sent_tc_token_timestamp) VALUES (?,?)", ('synthetic-user', captured)).rowcount
                effect = 'inserted_completed_state' if changed else 'no_timestamp_change'
        after = db.execute("SELECT sent_tc_token_timestamp,real_issue_timestamp FROM wa_trusted_contacts_send").fetchone()
        vectors.append({'before': None if before is None else {'sent': before[0], 'real_issue': before[1]}, 'captured': captured, 'after': {'sent': after[0], 'real_issue': after[1]}, 'effect': effect})
schedule_vectors = []
for raw in (None, -1, 0, captured):
    with sqlite3.connect(":memory:") as db:
        # 0hv.A08's cursor projection, then A0W's nonnegative carry.
        projected = db.execute("SELECT CASE WHEN ? IS NULL THEN ? WHEN ?=0 THEN NULL ELSE ? END", (raw, captured, raw, raw)).fetchone()[0]
        carried = db.execute("SELECT CASE WHEN ? IS NOT NULL AND ?>=0 THEN ? ELSE 0 END", (projected, projected, projected)).fetchone()[0]
        schedule_vectors.append({'before': {'sent': captured, 'real_issue': raw}, 'after_real_issue': carried})
print(json.dumps({'source': 'APK43.72 0ht.A0V selected conditional SQL, synthetic SQLite replay', 'vectors': vectors, 'schedule_vectors': schedule_vectors}, indent=2))
