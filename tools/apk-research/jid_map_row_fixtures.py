#!/usr/bin/env python3
"""Independent SQLite fixtures for APK 37.71 0Xx cursor / 0eU selection.

Only synthetic rows are used. No account, APK, network or Rust implementation
is opened. The literal SQL and reverse-map replacement are sourced from
docs/protocol/companion-address-mapping-evidence.txt.
"""
import argparse
import itertools
import json
import sqlite3
from pathlib import Path

SQL = (
    "SELECT lid_row_id, sort_id FROM jid_map WHERE jid_row_id = ? "
    "ORDER BY sort_id DESC, lid_row_id DESC"
)


def generate():
    connection = sqlite3.connect(":memory:")
    connection.execute(
        "CREATE TABLE jid_map(jid_row_id INTEGER, lid_row_id INTEGER, sort_id INTEGER)"
    )

    def case(name, rows, phone=7):
        connection.execute("DELETE FROM jid_map")
        connection.executemany("INSERT INTO jid_map VALUES(?,?,?)", rows)
        reverse_map = {}
        for lid, sort in connection.execute(SQL, (phone,)):
            reverse_map[sort] = lid
        return {
            "name": name,
            "jid_row_id": phone,
            "rows": [dict(zip(["jid_row_id", "lid_row_id", "sort_id"], row)) for row in rows],
            "selected_lid_row_id": reverse_map[max(reverse_map)] if reverse_map else None,
        }

    cases = [
        case("empty", []),
        case("other-phone-only", [(8, 20, 10)]),
        case("greatest-sort-not-greatest-lid", [(7, 100, 4), (7, 20, 5)]),
        case("equal-sort-overwrites-to-smaller-lid", [(7, 100, 5), (7, 20, 5)]),
        case("invalid-winner-does-not-fallback", [(7, 20, 1), (7, -1, 2)]),
        case("zero-winner-retained", [(7, 20, 1), (7, 0, 2)]),
        case("signed-sort-extremes", [(7, 9, -(2**63)), (7, 8, 2**63 - 1)]),
        case("signed-lid-extremes-at-equal-sort", [(7, 2**63 - 1, 5), (7, -(2**63), 5)]),
        case("negative-sort-values", [(7, 100, -5), (7, 20, -4)]),
        case("duplicate-identical", [(7, 20, 1), (7, 20, 1)]),
        case("negative-phone-row-query", [(-1, 20, 1)], -1),
        case("zero-phone-row-query", [(0, 20, 1)], 0),
    ]
    for index, rows in enumerate(
        itertools.permutations([(7, 100, 5), (7, 20, 5), (7, 10, 4), (8, 1, 100)])
    ):
        cases.append(case("permutation-" + str(index), rows))
    connection.close()
    return {
        "apk_version": "2.26.37.71",
        "apk_sha256": "1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c",
        "source": "0Xx.A01/A00 SQL; 0eU.<init>/A00 reverse TreeMap with equal-sort replacement",
        "generation": "Independent Python SQLite executes the literal APK SQL; cursor entries overwrite a dictionary by sort_id; greatest signed sort selects its retained lid. All rows are synthetic, not account data.",
        "sql": SQL,
        "cases": cases,
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true", help="compare without modifying the fixture")
    arguments = parser.parse_args()
    destination = Path(__file__).resolve().parents[2] / "docs/protocol/jid-map-row-selection-fixtures.json"
    fixture = generate()
    text = json.dumps(fixture, indent=2) + "\n"
    if arguments.check:
        if destination.read_text() != text:
            raise SystemExit("JID-map SQLite fixture differs; regenerate and review")
    else:
        destination.write_text(text)
    print(f"{'Verified' if arguments.check else 'Generated'} {len(fixture['cases'])} synthetic SQLite cases")


if __name__ == "__main__":
    main()
