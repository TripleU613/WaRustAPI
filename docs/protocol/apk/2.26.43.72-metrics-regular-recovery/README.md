# Regular metrics recovery gates — APK 2.26.43.72

**Confirmed static / bounded native decision slices.** APK SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`;
reconstructed ARM64 libwhatsapp.so SHA256
`be34e7520883f6fcedbdae0ed0167a1d84a7cdc5bbe63576a4115f89d3b782f5`.
[Reconstruction](../2.26.43.72-superpack-reconstructed-members/README.md).
No app, network, account, real timer or durable storage operation was executed.
Rizin/Unicorn exports are local evidence, not REA Evidence envelopes.

## Status callback checks provider identity before taking action

Complete selected body **0x727740..0x727908** first resolves a weak regular
manager from typed parameters, then a weak provider. Failed resolution exits.
Helper0xbe4674 saves incoming w1 as raw status and x2 as parameter object;
incoming x0 is retained for provider weak-reference resolution. The resolved
provider must equal **manager+0x28** before the status branch below runs.
An obsolete provider therefore does not trigger these status actions. This is
an identity check for this callback, not proof of generation fencing for every
outstanding task completion.

| Raw status, matching provider | Selected behavior |
| --- | --- |
| 0, busy byte+0x50 bit0 clear | start regular drain; create/store/retain/start timer |
| 0, busy byte bit0 set | skip drain start and timer creation |
| 1 | cancel existing timer if present; call drain-end with raw error1 |
| other | abort |

A mismatched provider skips the status branch, including its abort, and reaches
common cleanup. These status interpretations are local to this body; complete
external registration and status producer policy remain untraced.

The status0 timer is created with **interval30.0**, callback
**0xbd8f88 →0x727908**, weak context **manager+0x38**, allocator+0x18 and the
execution queue obtained from global queue+0x20. It stores the new timer in
**manager+0x58**, retains it, releases the old stored pointer, starts the new
timer through0x7360fc, then releases the local timer reference. This ready branch
has no explicit0x73629c cancellation call before replacing the old pointer;
release/destructor effects are separate and not executed here. On status1,
canceling a timer does not clear the stored+0x58 pointer in this body.

The byte+0x50 is the regular drain's active flag, not proof of a connected socket.
The status branch checks its low bit; the drain-end helper itself requires the
byte to equal1 before clearing/notifying. Those predicates differ for synthetic
values2/3; broader valid-state constraints remain open. See the
[drain notifier](../2.26.43.72-metrics-native-drain-outcomes/README.md).

## Timer callback retries only when ready and idle

Complete selected callback **0x727908..0x7279a4** resolves the weak regular
manager. Once resolved it calls provider-readiness bridge0x6fef18. Only if that
result's **low bit is1 AND manager+0x50 low bit is0** does it call regular drain
start0x727294. Otherwise it releases the resolved manager and returns.
There is no provider-identity comparison in this timer callback's selected body;
readiness and weak-reference lifetime are its local gates.

The shared timer constructor/arm/event/cancel implementation was already traced
and tested in the [timer bundle](../2.26.43.72-metrics-native-recovery-timer/README.md):
its repeat byte defaults1, and it requeues after the stored interval unless
canceled. Thus this selected regular path has periodic readiness/idle rechecks,
not a demonstrated exponential backoff or per-task retry count. This timer is
separate from the credential HTTP manager's+0xa8 timer. It does not prove all
regular failures are retried: full registration, destruction and task admission
still matter. A drain attempt can find no work, or be inhibited by Java gates;
see the [regular IQ job](../2.26.43.72-metrics-regular-iq/README.md).

## Reproduction and limits

```sh
python inspect-recovery.py /path/to/libwhatsapp.so
python check-recovery.py /path/to/libwhatsapp.so
```

The inspector requires the pinned image and reproduces `recovery-boundaries.json`.
The checker requires Unicorn2.1.4 and reproduces `recovery-results.json`.
The full selected bodies and argument helpers are exported using windows.json:
`rizin -2 -q -e scr.color=0 -c 'pD SIZE @ VA; q' IMAGE`.

**72 status cases** execute instructions0x727820..0x7278bc after synthetic weak
resolution. They vary provider identity, raw status0/1/2, busy byte0/1/2 and
presence of an old timer. Assertions cover action ordering, timer interval,
callback/context/queue/allocator arguments, retain/release sequence, stored
pointer and expected aborts. **16 timer cases** execute0x72796c..0x727984 after
synthetic weak resolution, varying readiness0..3 and busy byte0..3. They assert
when drain start is called. Real drain, timer constructor/start/cancel and
provider-readiness operations are intercepted; the chosen decisions and their
argument setup are actual pinned native instructions. No real delay is measured.

These slices do not execute weak-reference resolution, object destructors,
concurrency or the actual drain. Those surrounding bodies are retained for
static inspection; the earlier shared-timer harness covers a different scope.
The count88 is a fixture count, not a coverage percentage.

**Inferred:** retaining failed regular tasks plus this readiness/idle timer can
permit a later drain attempt, subject to lifecycle and queue membership.
**Observed live:** none. **Unknown:** complete callback bindings, provider/status
producers, valid busy states, full task generation fencing, durable buffer
removal/recovery and server restriction effects. No Rust behavior, telemetry
sender or new retry authority was added.
