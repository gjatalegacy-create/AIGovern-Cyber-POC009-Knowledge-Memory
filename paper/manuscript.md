# Scoped Model Support and Counterevidence Across Five Memory Cycles in ESS-MAI

Bledar Gjata · Independent Researcher, Albania  
[ORCID 0009-0003-7854-266X](https://orcid.org/0009-0003-7854-266X)

**24 October 2026.** Preprint identifier: [10.5281/zenodo.23614714](https://doi.org/10.5281/zenodo.23614714). Software identifier: [10.5281/zenodo.23614687](https://doi.org/10.5281/zenodo.23614687).

## Abstract

POC009 tests whether an executable dialogue mechanism can retain model support and counterevidence across five process restarts without confusing their propositions or granting additional authority. It combines an extracted Progressive Debatic (PD) engine, Earth Rotation Angle evaluation and KiE predicates, a source-derived one-use FFI gate, and new memory/storage code. The sequence begins with missing inputs, evaluates an incorrect claim, evaluates a corrected claim, and repeats both. Complete operation records and memory matched a persistent control at every cycle; checkpoint bytes matched uninterrupted PD. The final index retained one positive and one negative entry, each with two occurrence references. Fifty root tests and nineteen separate science tests passed. “Knowledge” remains scoped computational evidence; the full native verdict is null.

## 1. Numerical and research scope

The numerical provider is the IAU 2000 Earth Rotation Angle model. SOFA publishes the reference C library [1]; sofars supplies a Rust implementation [2]. POC009 preserves exact bytes of the selected sofars `era00.rs`, `consts.rs` and `anp.rs` files. Source hashes and extraction transformations identify the dependency closure. The surrounding manifests/module exports are narrowed, and the full MIT/SOFA notices are retained. This is neither a new astronomical formula nor SOFA endorsement.

For UT1 Julian date `2400000.5 + 54388.0`, the retained conformance fixture uses `0.4022837240028158102` radians and tolerance `1e-12`. The provider computes the angle; it does not read this expected value as its answer. Let theta denote the computed angle, c the supplied claim, and d the shortest circular angular distance. The source classifies d < epsilon as support, d > epsilon as counterevidence, and equality as unresolved. These classifications depend on the supplied time scale, values and tolerance; they are not independent observations of the Earth.

POC007 documented a bounded governance capsule [3,4]. POC008 isolated operational checkpoint continuation [5,6]. POC009 adds a focused question: can separately identified supporting and countering propositions remain available across restarted dialogue cycles while current computation and storage checks retain their distinct roles? Prior test totals do not answer this question.

## 2. Source, memory and authority

The science dependency retains the actual evaluation, query/candidate codecs and KiE computation. Canonical candidate decoding re-evaluates the pinned provider and compares the complete encoded record. `CandidateAuthority::None` remains unchanged. KiE predicate endpoints express Boolean model outcomes, not confidence, probability or independent corroboration.

The new memory index is rebuilt from verified WAL records. Content identity includes context, canonical proposition, outcome, declared source and numerical result bits. A SHA-256 bucket still requires full content equality. A repeated candidate can acquire another origin-record reference without becoming another unique candidate. The claim of zero radians and the corrected claim are different propositions in the same case: positive support for one does not cancel negative evidence against the other.

The storage owner restores the current PD checkpoint, recomputes the next model result and verifies the request before append. Replay reconstructs both state and index with corresponding checks. This is a new limited public adapter, not the original full KnowledgeVault. The output deliberately retains `knowledge_verdict: null` and `authority_transferred: false`; computational classifications do not reproduce full native admission.

## 3. FFI boundary

The extracted legacy `sovereign_validate_and_write` consumes a handle but returns `CompatibilityHold` (`-8`) for valid nonempty input. It does not write knowledge. POC009 adds `with_borrowed_payload`: after consuming the gate, it passes a borrowed slice to a callback. The command-line owner checks pointer and byte identity, decodes the typed input, independently recomputes within the process and invokes storage.

The helper therefore does not turn the old no-write surface into an original native commit path. Its callback is new code with its own obligations. Pointer/byte preservation at this boundary is narrower than an allocation-free pipeline; parsing, serialization and persistence can allocate or copy. Handles are process-local, nonces are deterministic labels, and unsafe callers must supply valid readable memory for the borrow. These properties do not establish a hostile-process sandbox or authentication.

A separate Python `ctypes` probe loaded the release DLL and passed 17 checks against its actual C exports. Valid nonempty write calls returned `-8`; copied-handle reuse and invalid inputs were rejected. Sixteen concurrent copies consumed one grant once, and the caller-owned byte buffer remained unchanged. This probes the legacy no-write ABI, not the CLI's new Rust callback or native knowledge admission.

## 4. Five-cycle protocol and measured results

Each cycle supplies typed JSON, an operation identity, case, text and explicit tick to a new process. The observed sequence was:

| Cycle | Science input | Observed result |
| --- | --- | --- |
| 1 | Missing | Request evidence; no negative candidate |
| 2 | Claim 0 radians | Model counterevidence |
| 3 | Corrected conformance claim | Model support |
| 4 | Claim 0 repeated | Retrieve negative content; retain new occurrence |
| 5 | Corrected claim repeated | Retrieve positive content; retain new occurrence |

The final index contained one positive and one negative unique content entry, each referenced by two distinct occurrences, plus one missing-evidence event. At every prefix the test compared complete operation records and reconstructed memory against a persistent-process control, and exact canonical checkpoint bytes against uninterrupted PD execution. The command-line boundary preserved input pointer/bytes, and consumed-handle reuse was refused. A separate old-operation retry test verified unchanged memory and storage size after the head advanced.

The release executable was also run through five fresh CLI processes followed by inspection. All 93 scripted checks passed; the final index again retained one entry per polarity with two origins each. The recorded inputs, output transcripts, process return codes and executable hash accompany this demonstration. These checks are separate from the root tests and the external ABI probe.

On Windows GNU with Rust 1.98.0, the root suite passed 32 unit tests (including 12 FFI checks) and 18 integration tests: 50 passed, zero failed or ignored. The final integration run took 0.88 seconds; the locked offline release build passed in 21.04 seconds using direct Cargo/compiler/rustdoc binaries. Its cases include scoped retrieval, canonical candidate mutation, recomputed public record hashes, stale operations, incomplete-tail recovery, complete CRC corruption and OS lock release after process termination. The valid-old-prefix test intentionally accepted rollback. Separately, 19 extracted science tests passed with zero failed or ignored in 0.03 seconds, using locked offline dependencies and `-Dwarnings`; direct compiler/rustdoc invocation emitted no warnings.

Initial tests exposed floating-point JSON round-trip loss at an exact equality boundary. Enabling `serde_json`'s `float_roundtrip` feature restored the intended comparison. Two subsequent CLI regressions showed that invalid scientific input could create storage or repair an existing incomplete tail before rejection. Moving evaluation before storage opening corrected both without weakening rejection assertions. After `cargo fetch --locked`, reproduction uses `cargo test --locked --offline`; dependency validation adds `--manifest-path dependencies/science-domains/Cargo.toml`. The build command is `cargo build --release --locked --offline`. Under `evidence/windows-gnu/`, the final logs are `cargo_test_final.log` and `cargo_release.log`; demonstration and ABI receipts are `five-cycle-demo/RESULT.json` and `ffi-abi/RESULT.json`.

Separately, [CI run 40083832092](https://github.com/gjatalegacy-create/AIGovern-Cyber-POC009-Knowledge-Memory/actions/runs/40083832092) succeeded on Windows and Ubuntu at commit `a24736cb28233331ecefc108317e76cf7e1a5ef0`, including root/dependency tests, release build, five-process demonstration and actual C ABI probe. Saved run/job metadata and selected log lines are under `evidence/github-actions/`. These are automated checks for the stated revision, not independent peer review.

## 5. Limits and contribution boundary

Five cycles cannot prove all-sequences or indefinite continuity. The inherited PD checkpoint format has a 4096-turn and 8 MiB ceiling, alongside input and record limits. Full WAL replay and complete checkpoints impose costs not benchmarked here. Inspection may initialize a store or repair an eligible incomplete tail.

Unkeyed hashes and deterministic FFI nonces do not authenticate authors or freshness. A coherent old valid history may be accepted. Source-byte/provider binding can reject older records after code changes; no migration guarantee is established. No physical power-cut or injected write/fsync failure campaign is reported. The retained FFI registry is process-local and retains consumed slots.

The phrase “negative knowledge” is therefore restricted to a stored model-conditional counter-result. Missing evidence, model boundary uncertainty and resource refusal are not scientific disproofs. No general understanding of unrestricted text, full native knowledge pipeline, certification or independent scientific validation is claimed. The contribution is the inspectable composition and its bounded measured behavior, not invention of replay, hashing or negative evidence.

## 6. Authorship, access and review

Bledar Gjata directs the architecture and interpretation. OpenAI Codex assisted extraction, engineering, tests and writing; the author reports shell-based review. Automated checks and multiple agent passes are not independent human peer review. The capsule uses Apache-2.0 with retained third-party license obligations.

Artifact locations are [GitHub](https://github.com/gjatalegacy-create/AIGovern-Cyber-POC009-Knowledge-Memory) and [HF](https://huggingface.co/datasets/gjata-legacy/ess-mai-poc-009-knowledge-memory). The software and preprint identifiers distinguish executable evidence from this manuscript; identifiers and hosting do not establish scientific validation. Reviewers should report exact versions, inputs, identities, commands and the earliest differing cycle, particularly any unintended authority promotion, scoped reuse, deduplication error or unsupported continuity claim.

## References

1. IAU SOFA Board. *SOFA Library Issue 2023-10-11 for ANSI C*. [Official release](https://www.iausofa.org/2023-10-11c).
2. Astro XAO / sofars contributors. *sofars: Rust implementation of the IAU SOFA library*. [Upstream repository](https://github.com/astro-xao/sofars). Selected 0.6.1 source identity is recorded in `science_provenance.json`.
3. Gjata, B. *AIGovern-Cyber POC007 public reference capsule*. [doi:10.5281/zenodo.23596907](https://doi.org/10.5281/zenodo.23596907).
4. Gjata, B. *Constitutional Self-Governance in ESS-MAI: Evidence-Bound Authority and an OPL2S Reference Capsule*. [doi:10.5281/zenodo.23597026](https://doi.org/10.5281/zenodo.23597026).
5. Gjata, B. *AIGovern-Cyber POC008: Evidence-Bound Continuity*. [doi:10.5281/zenodo.23613613](https://doi.org/10.5281/zenodo.23613613).
6. Gjata, B. *Bounded Operational Continuity in ESS-MAI: A Source-Derived PD Checkpoint and Replay Experiment*. [doi:10.5281/zenodo.23613785](https://doi.org/10.5281/zenodo.23613785).
