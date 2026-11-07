# Post-publication keyed follow-up

**Confirmed selected static evidence**, same pinned APK and native member as
README.md. No target execution or live operations. REA evidence:
`ev_e454fe658d9239b0e2d2fa9fdcfe5bafba6391a3cd3f5ba4edba7d87785c1608`.

The 0x87cbc..0x87d04 caller gates on owner+0x440, candidate nonnull and its
root probe. It calls 0xc4338 with x0=owner+0x440, w1=word+0x408,
x2=string+0x428 and x3=a constructed empty string. It does not pass the
candidate reader or the published status byte as an argument.

0xc4338 saves those arguments through outlined helpers 0xcb7f4 and 0xcb72c.
After initializing a shared registry through 0xc3984, it looks up the key
whose pointer at 0xdb638 has RELATIVE addend **0x1a176**:
`mc_1st_set_latest_cache`. Missing lookup returns without the later dispatch.
For a present key it obtains the registry value through 0xc3e2c and compares
it with the receiver's virtual+0x80 result through 0xc3e64. That comparison is
signed `receiver_result >= registry_value`; false skips dispatch. The receiver
method is called with string key from 0xdb5f8, RELATIVE addend **0x18584**:
`mobileconfig_freshinstall_track_version`.

On true it locks receiver+0x15c and calls 0xc3efc with the original mode and
strings, selected event key, and w5=1, then unlocks. The selected 0xc3efc body
builds string/map-like fields using 0xc41f8 and 0xc42e0, includes literal keys
`v` (0x1630d) and `extra_info` (0x182b9), obtains system_clock::now(), divides
its returned integer by 1000 and formats that value with to_string. It also
uses receiver virtual+0x28 and ultimately dispatches receiver virtual+0x50 at
0xc4074. Timestamp units are not assigned here beyond that literal operation.

**Inference:** keyed event/version bookkeeping is a better description of
this boundary than reader consumption. **Unknown:** concrete receiver type,
virtual+0x80/+0x28/+0x50 implementations, registry mutation and downstream
effects. An implementation could access other state internally; the absence
of a reader argument is not proof that all downstream code ignores it. This
does not prove durable activation, server acceptance, or mandatory telemetry.
No Rust request or persistence policy is changed from this evidence alone.

Reproduce the bounded static reads with `llvm-objdump -d
--no-show-raw-insn --start-address=0xc4338 --stop-address=0xc442c LIB`,
then 0xc3e2c..0xc40e8 and the outlined argument helpers. Resolve data pointers
with `readelf -rW LIB`, not zero-filled raw relocation slots.
