# Group notification event selection

**Confirmed static**, APK2.26.43.72, SHA-256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Checked DEX text, not REA Evidence envelopes. Commands: xtask apk-smali APK
--class LX/0Tp; --method A0D --signatures --exceptions; same for LX/0w9;
--method A09. The latter export is bounded here with complete-source SHA-256;
selected offsets are method-relative bytes, branch deltas are DEX code units.

0Tp.A0D returns children[0] or throws LIL if absent/empty.0w9.A09 calls it at
0020..0028 and keeps that child in v0. Feature0x36fe is checked at0056..0062;
false jumps0062+0x17e*2=035e, where v0 is tested for create, remove and add.
Selected demote0786 and promote08da tests likewise inspect v0. The true branch
re-reads the first child at0066..006e for its separate feature-specific path.
This is first-event selection, not an outer-child iteration applying each
sibling as an additional membership change. Full helper side effects, gate
values, all event families and state/timestamp/address handling remain unported
or outside these selected ranges.

The existing [missing-identification trace](../2.26.43.72-group-missing-identification/README.md)
separately shows direct marker lookup on the whole notification. Extra siblings
can carry metadata; this finding does not justify rejecting every multi-child
notification or ignoring all sibling metadata in a full reconstruction.

**Confirmed local defect/fix:** groups::apply_change_notification iterated every
outer child, so unsupported+remove or add+remove could mutate membership using
an unselected sibling. It now applies only the first event. Inner participant
iteration remains unchanged. Empty/unsupported first events perform no supported
snapshot change; the separate ACK policy is unchanged, not claimed identical
to native missing-child exceptions. Existing envelope/store guards remain.

The expanded change_notifications_update_only_a_saved_snapshot regression fails
on the old unsupported+remove case. With the correction it preserves exact saved
bytes there; add+remove applies add while preserving the existing member, and
later standalone valid role/subject/remove controls remain. No change to live
accounts, automatic queries, identity policies or retransmission.

Checksums record all other bundle files' bytes and SHA-256.

## Dirty event with metadata siblings — 2026-11-05

**Confirmed static:**0w9.A09 tests the same selected first child v0 for
groups_dirty at2868..2878, then calls A0N(group) on that child and loops its
selected group children. The retained excerpt closes this specific branch;
there is no outer-node child-count==1 prerequisite in that selected test.
The broader request scheduling path remains covered by the existing37.71
and current group-lifecycle traces, not re-established by this short excerpt.

**Confirmed local defect/fix:** handle_dirty_notification required exactly one
outer child, so groups_dirty plus missing_participant_identification skipped
refresh scheduling. The generic notification_ack then admitted the outer type,
allowing ACK without that work. Both helpers now recognize first-child dirty:
the dedicated handler retains the durable queue/dedup-before-ACK path, while
the generic helper returns None to prevent a second ACK or fallback ACK after
local dirty-content refusal. Additional siblings remain captured; this does not
implement missing-identification state or authorize secondary sibling events.
Local bounds on the inner group list and envelope remain unchanged.

The existing queue-before-ACK/reopen regression now supplies that metadata
sibling. It fails at missing dedicated ACK on the original code, then checks
one queued refresh, no generic ACK, and dedup/original reservation retention
after reopen. Changed-content same-ID refusal remains. No live request,
identity change, automatic replay or additional query family was introduced.

## Dirty list named-child filtering — 2026-11-05

**Confirmed static:**0w9.A09's dirty branch at2882..288e invokes0Tp.A0N(group)
on the selected event. The complete A0N helper is now retained: it iterates
children, adds only exact TextUtils.equals(tag,name) matches and otherwise
continues. Null child-array returns EMPTY_LIST. A non-group child is not an
invalid group entry in that selected path.

**Confirmed local defect/fix:** Rust rejected the entire list on any non-group
child. A metadata sibling inside groups_dirty could suppress a valid group's
queued refresh and ACK. The helper now selects only group children for group
parsing, while keeping the256-total-child local resource bound and all selected
entry checks (group identity, duplicate, device, missing/empty supported list),
envelope, persistence and ACK-ownership checks. This is not a blanket valid
result for every arbitrary/empty list: unknown-only still has no supported work.

The queue-before-ACK/reopen test now prepends metadata with an unusable jid to
its valid group list. It fails on the previous missing-ACK path, then confirms
exactly one intended refresh and the same post-reopen reservation/dedup guards.
The malformed selected-group matrix remains unchanged. No new query family,
identity policy, live operation or native all-error-ACK parity is claimed.
