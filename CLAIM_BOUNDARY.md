# Claim boundary

**Local evidence, 24 October 2026:** 50 root tests and 19 separately run science tests passed, with zero failed or ignored; the locked offline release build passed in 21.04 seconds. Software and preprint DOIs are reserved as recorded in the README. Publication and CI evidence remain pending.

“Positive knowledge” here means retained model-conditional support for an exact numerical proposition. “Negative knowledge” means retained model-conditional counterevidence for an exact proposition. Neither means independently observed physical truth, full native admission or unrestricted understanding of free text. Typed JSON supplies the science inputs.

For computed angle theta, claimed angle c and declared tolerance epsilon, the source compares the shortest circular residual d: support if d < epsilon, counterevidence if d > epsilon, unresolved if d = epsilon. Missing science data produces a request for evidence, not negative knowledge. `CandidateAuthority::None`, `knowledge_verdict: null` and `authority_transferred: false` remain explicit.

| Observed test boundary | Evidence | Limit |
| --- | --- | --- |
| Five-cycle continuity | Complete operation records and reconstructed memory matched a persistent control; exact PD checkpoint bytes matched uninterrupted execution at each of five prefixes | Finite examples, not all-sequences or infinite continuity |
| Two retained polarities | One positive and one negative unique content entry remained after the five-cycle run | They do not cancel; they are different claims, not independent observations |
| Repeated content is deduplicated | The final two entries each retained two occurrence references; the separate old-operation retry test left memory and storage size unchanged | New occurrence is not new independent evidence; exact retry differs from a new operation |
| FFI payload is borrowed unchanged | Pointer and byte equality at the helper boundary; consumed handle cannot be reused there | Unsafe caller must provide valid memory; later processing can allocate/copy |
| Storage owns recomputation | Recomputed current candidate and full request comparison before append, and equivalent replay checks | Same model and process, not independent scientific verification |

The original legacy FFI write call remains a no-write `CompatibilityHold`. The new callback helper does not inherit a native verdict or make its callback trustworthy by name. Handles are process-local, nonces deterministic, consumed slots retained, and no hostile in-process sandbox or authenticated capability system is established.

The new storage/index code is distinct from the complete KnowledgeVault. Reopening replays the whole WAL; inspection can initialize or repair a trailing incomplete record. A coherent older valid history may be accepted without an external freshness anchor. Changed adapter source bytes or scientific provider bytes can invalidate earlier records; migration is absent.

The inherited checkpoint format permits at most 4096 turns and 8 MiB, with other record/input limits. These are code bounds, not proof that every boundary or sequence was tested. No constant-space/time, physical power-cut safety, write/fsync fault-injection coverage, certification, AI Act compliance or third-party endorsement is asserted.

The root integration suite records these checks in `tests/knowledge_memory.rs`; source FFI checks are in `src/ffi_gate.rs`. Initial failures at the floating-point equality boundary and invalid-input CLI storage ordering required corrections; both regression cases are retained. The valid-old-prefix test intentionally accepts the rollback, documenting a limitation rather than a prevented attack. Release and CI evidence must identify their own exact source revision.
