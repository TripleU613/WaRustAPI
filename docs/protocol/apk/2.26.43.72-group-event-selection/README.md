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
