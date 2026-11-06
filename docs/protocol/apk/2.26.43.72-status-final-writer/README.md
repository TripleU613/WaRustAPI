# Status final writer and transport output — APK 2.26.43.72

**Confirmed selected static**, build264307238, APK SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Complete selected xtask DEX exports, not REA Evidence or runtime observations.
No account operations or Rust behavior change.

## Reuse the established channel, not a guessed interface override

[Existing channel binding](../2.26.43.72-channel-queue-predicate/README.md)
connects1A3.CdY (what4), connection handler,0ZJ.A0r,0vI.CdY (what2), then
writer0v0.A00. These envelopes preserve logicalarg1=8 for status sends.
The retained superclass-only override query returns only abstract1A0.CdY;
its declared scope excludes interface dispatch. That negative is not evidence
that no implementation exists. The existing concrete binding supplies it.

0Sf.A00 calls1i8.A07 to prepare pre-ACK traffic. A07's branches excludearg1=8,
so it returns null for this selected status message;0Sf.A05 copies the original
Message instead of replacing it witharg1=43/pre-ACK. This narrows the earlier
queue-preparation gap; it does not generalize to receipts or other message types.

## Type8 reaches prepared message construction

The [complete retained writer](../2.26.43.72-iq-promotion-shutdown/writer.txt)
at03ae..0412 castsobj toEng, calls0uz.A00(connection,Eng), then logs and
invokes0uv.C4a(messageKey).0uz delegates to0ux.A00. That method takes
Eng.A06.values as root attributes andEng.A05 as ordered children, constructs
root `message`, then calls connection.A06:1r0.Cyt at005a BEFORE its subsequent
metadata/bookkeeping branches. No new required attribute or child is inserted
in this selected final handler. No local account/row/routing revalidation is
visible before Cyt; producer-time guards remain separate evidence.

C4a is a local notification after normal handler return: concrete0uw posts
what2, whose handler calls0ZJ.A0x. FullA0x effects are untraced here. This is
not a server response, nor proof of acceptance. Post-Cyt metadata branches
can themselves throw (e.g. required null checks), so the absence of this
notification would not prove that no bytes were written.

## Selected codec binding, counter, framing and flush

The [retained0ZJ.A0z](../2.26.43.72-about-callback-lifetime/owner-initialization.txt)
at0904..0916 obtains1o3.A06 and constructs1r2 in v14; at0a0e..0a24 it passes
that v14 as1rj's1r0 constructor argument. Retained constructors store those
exact objects. The exact constructor caller census has one physical caller;
this is not exhaustive runtime lifecycle or alias proof.

1o3.A06 wrapsA07:1oH plusA09:1qw in1r1.1r2.Cyt invokesCyu(node,1).
Cyu's bit2 test is false for this path: write zero flag byte, serialize through
A01, obtain bytes; its separate bit2 branch uses DeflaterOutputStream and flag2,
not selected byCyt. The full node serializerA01, allocation provider and
size-exception producers remain untraced here.

At00ec..0122 Cyu reads the1qw transport context, obtains
AtomicLong.getAndIncrement BEFORE1oU.A01 encrypts the record using empty
associated data, then callsOutputStream.write(ciphertext) andflush.1oU.A01
constructs a12-byte nonce with the supplied counter written atoffset4 through
1nr.A02; endianness and complete cipher implementation are not inferred here.
The counter is consumed before output. No counter rollback is shown inCyu.

1oH.write(byte[],offset,length) rejects length>=0x1000000 withE5W; otherwise
writes1nr.A04(length)'s three-byte big-endian length, then ciphertext bytes,
then flushes its underlying stream. Header and payload are separate writes:
IOException can follow partial output. Full underlying stream/socket creation,
all connection header/teardown paths and successful remote receipt remain open.
This framing bound is distinct from the earlier large-stanza diagnostic and
is not a new server payload or safe-retry rule.

## Failure notification is not generic replay authority

Writer type8 has selectedE5X/E5W catches at0414/0426; both useEng.A03'sEoh,
route to0uv.C4g, then rethrow. Concrete0uw.C4g calls0ln.A0B, which removes
that exact full key under the0Th map lock and invokesBsR outside it. Unlike
0Sf.A0T, this selected body does not normalize again or compare owner identity.
Unknown size-exception producers prevent a blanket claim that these failures
always precede encryption or output. Missing callback does not settle anything.

The writer's separateIOException handler at2578 setsA01=true, queues aMessage
copy at the front and reportsCGH. Type8 is in its offline-drop switch, so a
later processing attempt whileA01=true goes toC6j; the queued copy is not proof
of a retransmission. Reconnect/generation ordering andC6j effects for non-IQ
status ownership remain untraced. No native universal retry rule follows.

## Rust comparison and limits

`transport/handshake.rs` EncryptedConnection.write takes its inner state before
encrypt/write and restores it only on success. Split writer usesInFlight to
poison the shared channel on error/cancellation. `socket.rs` takes the stream
before write_all/flush and restores it only on success. This preserves the
existing local boundary: after uncertain encrypted output, consume the channel
and retain original durable uncertainty; do not replay or roll back the nonce.
This is a bounded compatibility finding, not full native recovery parity.

Offline controls: `partial_encrypted_comparison_write_keeps_durable_uncertainty_and_consumes_channel`
and `interrupted_writes_and_reads_close_permanently`. These exercise real Rust
failure boundaries, not native Android execution. No new runtime fields, timers,
retries, status settlement or feature capability are added. Remaining work:
complete node serializer/size exception attribution, post-write metadata,
underlying stream, disconnect callback ownership and ciphertext-store authority.

## Reproduce

From checkout root with the pinned APK outside Git; compare output bytes with
these retained filenames (run redirects in a separate directory):

```sh
cargo build --locked -p xtask
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/1A0;' --signatures --exceptions > interface.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/0Sf;' --method 'A00' --signatures --exceptions > prepare.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/0uz;' --method 'A00' --signatures --exceptions > encrypted-delegate.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/0ux;' --method 'A00' --signatures --exceptions > final-render.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/1i8;' --method 'A07' --signatures --exceptions > preack-selection.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/0uw;' --method 'C4a' --signatures --exceptions > write-notify.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/0uw;' --method 'handleMessage' --signatures --exceptions > write-notify-handler.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/0uw;' --method 'C4g' --signatures --exceptions > size-failure.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/0ln;' --method 'A0B' --signatures --exceptions > size-failure-registry.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/1rj;' --method '<init>' --signatures --exceptions > connection-constructor.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/1r2;' --method 'Cyt' --signatures --exceptions > codec-entry.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/1r2;' --method 'Cyu' --signatures --exceptions > codec-write.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/1r2;' --method '<init>' --signatures --exceptions > codec-constructor.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/1o3;' --method 'A06' --signatures --exceptions > output-binding.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/1r1;' --method '<init>' --signatures --exceptions > output-constructor.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/1oU;' --method 'A01' --signatures --exceptions > record-encrypt.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/1oH;' --signatures --exceptions > framing.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/1nr;' --method 'A04' --signatures --exceptions > frame-length.txt
target/debug/xtask apk-callers ../android-lab/apks/WhatsApp.1.apk --class 'LX/1rj;' --method '<init>' > binding-callers.txt
target/debug/xtask apk-virtual-overrides ../android-lab/apks/WhatsApp.1.apk --class 'LX/1A0;' --method CdY --signature '(Landroid/os/Message;)V' > superclass-overrides.txt
```

PCs are bytes; branch deltas are code units. All exports are complete selected
methods/classes; direct-reference/ancestry census limitations stay explicit.
