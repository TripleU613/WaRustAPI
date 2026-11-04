# Server ping-to-pong path

Current APK **2.26.43.72**, SHA-256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
[Interpretation, Rust change, regression and unknowns](../../server-pong.md).

The nine JSON records contain complete REA method Evidence. The three `.txt`
files are complete structural disassembly from the repository's checked DEX
reader, not REA pseudocode. Reproduce them against this exact APK:

```sh
target/debug/xtask apk-smali /path/to/WhatsApp.1.apk \
  --class 'LX/' --method CPm --signatures --exceptions > reader-handler-routing.txt
target/debug/xtask apk-smali /path/to/WhatsApp.1.apk \
  --class 'LX/0vI;' --signatures --exceptions > writer-handler-routing.txt
target/debug/xtask apk-smali /path/to/WhatsApp.1.apk \
  --class 'LX/0v0;' --method A00 --signatures --exceptions > writer-dispatch-checked.txt
```

The first command is an exact method-name declaration sweep; its two declarations
identify the interface and concrete implementation, not complete interface or
runtime dispatch verification. The concrete Handler bodies retain the actual
Message forwarding used in the interpreted path. `ownership.json` links this
bounded method evidence to the existing connection module; it does not claim
whole-method or whole-APK Rust parity.

Checked source SHA-256 values:

| File | SHA-256 |
| --- | --- |
| reader-handler-routing.txt | `7c33ac44e6ba78d7ed1fe6a551f7dce259b3ce082b383386a5af4b2601a337c7` |
| writer-handler-routing.txt | `994a512e13e91cb1123ec01f1aba8ae7294384686810e16b17f4d3dbd34e5424` |
| writer-dispatch-checked.txt | `6ec286521e703b2e16b3ff057c27e744fe5835683b8edd815cfeefec12c32d3d` |

Raw APK bytes and broad decompiler trees remain outside Git. Static method
observations do not establish native scheduling or live server acceptance.
