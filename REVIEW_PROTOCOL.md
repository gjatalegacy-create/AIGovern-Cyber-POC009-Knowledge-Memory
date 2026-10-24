# Reproduction and critical review

**Local validation:** 50 root tests and 19 separate science tests passed, plus 93 release CLI checks and 17 external C ABI checks in separate runs. See [CLI evidence](evidence/windows-gnu/five-cycle-demo/RESULT.json) and [ABI evidence](evidence/windows-gnu/ffi-abi/RESULT.json). Separately, [CI run 40083832092](https://github.com/gjatalegacy-create/AIGovern-Cyber-POC009-Knowledge-Memory/actions/runs/40083832092) passed the test/build/demo/ABI steps on Windows and Ubuntu at `a24736cb28233331ecefc108317e76cf7e1a5ef0`. No completed independent review is asserted.

Read the source provenance, [claim boundary](CLAIM_BOUNDARY.md) and [manuscript](paper/manuscript.md), then run the exact locked artifact with its supplied five-cycle fixtures. Distinguish a source observation, a proposed test and an executed counterexample.

Please investigate:

1. Whether the sequence missing → claim-0 counterevidence → corrected-claim support → repeated counterevidence → repeated support preserves the actual scoped records and restored PD state at every observed cycle.
2. Whether the two distinct propositions coexist without cancellation; repeated content should retain occurrence references without increasing unique-candidate or independent-evidence counts. Exact retry should not masquerade as a new occurrence.
3. Whether changing case, scope, source, input bytes, time, model version or predecessor can bypass recomputation or reuse another record incorrectly.
4. Whether malformed/caller-edited candidate records pass canonical decoding, current-owner recomputation or replay after public hashes are recomputed.
5. Whether the source FFI call remains no-write `CompatibilityHold`, the helper really borrows the pointer/bytes, and a consumed handle is refused. Exercise the unsafe interface only with valid memory satisfying its contract.
6. Whether any output or prose promotes model support/counterevidence into physical truth, native admission or independently observed evidence.
7. Whether rollback, torn-tail recovery, full replay costs, version binding and finite bounds are accurately described.

Report the exact commit/tag, OS, toolchain, commands, input bytes and identities, the first differing cycle, expected behavior, observed behavior and smallest reproducible case. Include byte comparisons where exact equality is claimed. Do not publish private histories or credentials with a report.

Negative results, corrections and independent replications are welcome. Review and citation are voluntary. A successful five-cycle run supports only its measured scope. No favorable conclusion or endorsement is expected.
