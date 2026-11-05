# Native metrics credential lifecycle — APK 2.26.43.72

**Confirmed static / derived**, recovered ARM64 libwhatsapp.so SHA256
`be34e7520883f6fcedbdae0ed0167a1d84a7cdc5bbe63576a4115f89d3b782f5`;
pinned APK SHA256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
[Recovery provenance](../2.26.43.72-superpack-reconstructed-members/README.md).
**Observed synthetic emulation:** 17 bounded storage/setter/scheduler fixtures.
**Observed live:** none. Local Rizin exports and synthetic emulation are not
REA Evidence envelopes. No provider, signature operation, Android app or network
request was executed. No Rust production behavior changed.

## Admission and consumption

The [retained complete sender](../2.26.43.72-metrics-native-transport-responses/buffer-sender-assembly.txt)
0x7280dc reads current credential manager+0x78, spare +0x80. Counter +0x38
must be nonnegative (negative aborts); zero drops current and promotes spare.
For positive counter, clock 0x72c4fc must be <= credential+0x30. Expiry also
releases/promotes; admission repeats for the promoted object. With no admitted
credential, it calls refresh 0x728b24 and callback-table +0x20, then leaves the
selected construction path. An admitted credential's counter is decremented at
0x728224 **before** body/request creation. Failure after admission is therefore
not proved to restore a use. Endpoint removal likewise precedes creation; see
[request framing](../2.26.43.72-metrics-native-request-builder/README.md).

After decrement, counter <5 OR clock+10000 > expiry prompts refresh if no spare
exists (0x7282d4..0x728308). Equality at the time threshold follows the no-refresh
branch unless the counter threshold already fired. These are selected native
metrics conditions, not account/message retry rules or server rate limits.
Clock is MCFAbsoluteTimeGetCurrent plus imported epoch-offset double, followed
by sign-dependent half adjustment and integer conversion with bounds checks.
Its full clock/epoch contract is not executed by this fixture; time increments
are retained as **raw units**, not inferred deadlines for another subsystem.

## Construction and local counters

Constructor **0x7212f4** stores and retains arguments x0/x1/x2 at object
+0x18/+0x20/+0x28. It sets +0x30 to clock+**172800** and +0x38 to **64**.
Setter **0x721368** accepts nonnegative signed32 values and aborts on negatives.
Three constructor fixtures use distinct raw clocks and check every stored slot;
seven setter fixtures check zero, positive boundaries and negative rejection.
Clock/type/allocation/retention are explicit controlled adapters, not real MCF
allocation or reference-count correctness tests.

Completion at 0x72a6b4..0x72a6c4 overrides the new object's counter with **180**
when manager signed mode at +0x30 equals zero. Therefore 64 is a constructor
default, not a universal use count. This override is static evidence only.
Credential payload/cryptographic meanings of the three retained inputs remain
**unknown**; their storage is established without manufacturing values.

## Refresh guards and completion ownership

Scheduler **0x728b24** logs renewal, then:

- If provider +0x48 exists and predicate 0x6fef18 returns low bit zero, it logs
  not connected/not ready and returns. Full Java predicate behavior is untraced.
- Byte +0x88 equal to one returns as already renewing.
- Nonnull temporary slot +0x98 returns as already having a token.
- Otherwise it sets +0x88 to one and schedules thunk **0xbd8fb8**, context
  manager, through MCIGlobalExecuteAsyncHelper. The thunk branches to **0x7298b8**.

Seven synthetic scheduling cases check these branches, including no provider,
provider-not-ready, pending, temporary token and combined guards. A diagnostic
raw flag value two also schedules: the code tests equality with one, not generic
nonzero. This is a bounded malformed-state observation, not a recommendation to
use noncanonical state. Async execution is intercepted and recorded, never run.

Refresh worker requires temporary slots +0x98/+0x90 initially null (otherwise
abort), uses provider +0x48, and clears +0x88 if no provider exists. With provider
it constructs transient data, installs manager+0x50 as provider context +0x38,
and calls bridge 0x6ecd60 with selector **7** and data. Complete key generation,
JNI dispatch, server request and asynchronous completion binding remain open.
No absence of a timeout is claimed from this selected scope.

Completion **0x729cd0** requires a usable weak manager and its execution context,
then clears +0x88. The raw argument w3 selects this five-entry table (not HTTP
statuses and not the separate regular-task enum):

| Raw result | Branch / selected behavior |
| --- | --- |
| 0 | Validation/cryptographic path; two response data lengths must be 32; full verification remains untraced |
| 1 | Logs retriable error, then cleanup |
| 2 | Logs unretriable error, cleans temporary slots, calls 0x727bd0 |
| 3 | Cleans temporary slots; logs timeout |
| 4 | Cleans temporary slots; logs response fails validation |

Unknown values above four skip this switch. This is not proof they are valid
caller inputs. The table is independently decoded by inspect-lifecycle.py.
The raw0 path can log `Invalid signature received from server`; no valid
signature was generated, accepted or bypassed in this work.

At the successful construction tail, new credential goes into +0x78 if empty;
that branch schedules 0x727d40 and observer helper 0x728bfc. Otherwise +0x80 must
be empty and receives the new spare; an occupied spare aborts. Cleanup
**0x72ad9c** releases/zeros temporary +0x98/+0x90. Full stale-completion fencing,
provider-context lifetime, failure recovery and durability remain **unknown**.
These are in-memory credential transitions, not durable account-state deletion.

## Reproduction and limits

`python inspect-lifecycle.py RECONSTRUCTED_LIBWHATSAPP.so` must reproduce
lifecycle-boundaries.json: digest-bound windows, log labels, switch and decoded
thunk targets. Rizin exports use `pD SIZE @ VA` for these windows; complete
selected bodies are retained, not complete semantic coverage.

With Unicorn 2.1.4, run `python check-lifecycle.py RECONSTRUCTED_LIBWHATSAPP.so`;
stdout must reproduce lifecycle-results.json. Each fixture is capped at 100000
instructions and one second. Loader rejects unsupported relocation kinds;
unexpected imports and target interrupts fail. Clock/type/log/readiness,
allocation/retain and async scheduling adapters are explicit. Native constructor,
setter and scheduler instructions run; cryptographic worker/completion do not.
No I/O or real scheduler is supplied to native code.

**Inferred:** preserving credential-use and refresh ordering is necessary when
porting this selected metrics path. **Unknown:** whether a particular deployment
needs this subsystem, complete provider/wire schema, clock units, cryptographic
contract and all server acceptance requirements. No restriction-avoidance or
full protocol-coverage guarantee follows. The next trace should bind selector7
and completion arguments to Java request/response producers, then compare the
actual Rust surface before adding behavior.
