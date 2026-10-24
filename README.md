# AIGovern-Cyber POC009: Knowledge and Memory Continuity

**Software identifier:** [10.5281/zenodo.23614687](https://doi.org/10.5281/zenodo.23614687). **Preprint identifier:** [10.5281/zenodo.23614714](https://doi.org/10.5281/zenodo.23614714). Local validation and the verified Windows/Ubuntu CI run are described below; identifiers and hosting do not establish independent review.

POC009 investigates scoped computational support and counterevidence across five restarted PD cycles. It combines the actual extracted PD engine, the actual science-domains ERA/KiE computation and a source-derived one-use FFI gate with new public memory/storage adapters. The full native `knowledge_verdict` remains `null` and `authority_transferred` remains `false`.

Author: **Bledar Gjata, Independent Researcher, Albania**, [ORCID](https://orcid.org/0009-0003-7854-266X). Human-led architecture and AI-assisted engineering are disclosed in [WORKING_METHOD.md](WORKING_METHOD.md).

## The five-cycle question

The numerical task uses IAU 2000 Earth Rotation Angle through `sofars::erst::era00`, with UT1 Julian date `2400000.5 + 54388.0`. The conformance reference is `0.4022837240028158102` radians and the declared tolerance is `1e-12` radians. The model and its published implementation are existing work: [IAU SOFA](https://www.iausofa.org/2023-10-11c), [sofars](https://github.com/astro-xao/sofars).

| Cycle | Input | Observed behavior in the five-process test |
| --- | --- | --- |
| 1 | Science input missing | Requested scoped inputs; no counterproof created |
| 2 | Claim `0` radians | Computed model counterevidence |
| 3 | Corrected claim `0.4022837240028158102` | Computed model support |
| 4 | Repeat claim `0` with a new operation | Retrieved the same negative content; retained another occurrence reference |
| 5 | Repeat corrected claim with a new operation | Retrieved the same positive content; retained another occurrence reference |

These are two distinct numerical propositions in one case. A supported correction does not erase the earlier countered claim. Repeated computation is not independent evidence. A record's content identity is scoped and checked by full content equality even within a SHA-256 bucket.

## Source and new work

- [Base provenance](base_provenance.json): the POC008 PD/codec/storage starting point.
- [Science provenance](science_provenance.json): original ERA evaluation, codecs and KiE predicates; three sofars numerical files retain exact bytes.
- [FFI provenance](ffi_provenance.json): the source gate, legacy no-write compatibility path and disclosed new helper.
- [Adapter provenance](adapter_provenance.json): the POC009 storage adaptation. `knowledge.rs` supplies the new scoped content index.

The legacy `sovereign_validate_and_write` consumes its handle and returns `CompatibilityHold` (`-8`) for valid nonempty input; it does not write knowledge. New `with_borrowed_payload` consumes the gate and borrows the same pointer/bytes for a callback. A separate owner recomputes and validates before persistence. This is a boundary-specific borrow, not an end-to-end allocation-free pipeline or native authorization.

The index is reconstructed from checked WAL records. Distinct operation records can reference identical content without multiplying unique candidates. The adapter is new POC code, not the complete original KnowledgeVault.

## Reproduce

From the capsule root, with Rust/Cargo and Python 3 available, fetch the locked dependencies once and run the offline checks. The recorded local toolchain is Rust 1.98.0 on Windows GNU; CI also exercises Ubuntu. A minimum supported Rust version is not established by these runs.

```text
cargo fetch --locked
cargo test --locked --offline
cargo test --locked --offline --manifest-path dependencies/science-domains/Cargo.toml
cargo build --release --locked --offline
```

Run the release demonstrations with **new output directories outside the capsule**. The sibling paths below must not already exist. They collect copied runners, fixture bytes, actual outputs, hashes and result receipts without changing the package.

Windows:

```text
python tools/demo.py --binary target/release/poc009-memory.exe --output-dir ../poc009-demo
python tools/ffi_probe.py --library target/release/poc009_knowledge_memory.dll --output-dir ../poc009-ffi
```

Linux:

```text
python tools/demo.py --binary target/release/poc009-memory --output-dir ../poc009-demo
python tools/ffi_probe.py --library target/release/libpoc009_knowledge_memory.so --output-dir ../poc009-ffi
```

The direct command surface is `poc009-memory step STORE INPUT.json` and `poc009-memory inspect STORE CASE`. The supplied fixtures are under `fixtures/`; `inspect` opens storage and can initialize or repair an eligible incomplete tail.

Local Windows GNU tests on Rust 1.98.0 passed **50 root tests: 32 unit tests (including 12 FFI tests) and 18 integration tests**, with zero failed or ignored. The five-process test compared complete operation records and reconstructed memory against a persistent control at every cycle, and compared exact checkpoint bytes against uninterrupted PD execution. Its final index held one positive and one negative content entry, each with two occurrence references, and one missing-evidence event.

Separately, **19 extracted science tests passed**, zero failed or ignored, using locked offline dependencies and `-Dwarnings`; see [dependency validation](dependencies/science-domains/VALIDATION.json). These are separate suites. Initial regressions exposed JSON float round-trip loss at the equality boundary and CLI storage creation/tail repair before invalid scientific input was rejected. Enabling `serde_json`'s `float_roundtrip` feature and evaluating input before opening storage corrected them; the rejection checks remain in the suite.

The locked offline release build passed in **21.04 seconds** using the direct Cargo, rustc and rustdoc toolchain binaries. The final root run again passed all 50 tests; its integration section took 0.88 seconds. See the [final test log](evidence/windows-gnu/cargo_test_final.log) and [release build log](evidence/windows-gnu/cargo_release.log).

The release CLI demonstration separately passed **93 checks across five fresh processes**; its [result record](evidence/windows-gnu/five-cycle-demo/RESULT.json) binds executable, input and output hashes. The actual exported C ABI was called through Python `ctypes` and passed **17 checks**; see the [ABI result](evidence/windows-gnu/ffi-abi/RESULT.json). Valid nonempty calls to the legacy write export return no-write `CompatibilityHold` (`-8`), while invalid/reused handles return their rejection codes. The CLI's new borrowed callback is a distinct Rust interface; the C probe does not demonstrate native knowledge writing. These check counts are separate from the Rust test totals.

[GitHub Actions run 40083832092](https://github.com/gjatalegacy-create/AIGovern-Cyber-POC009-Knowledge-Memory/actions/runs/40083832092) succeeded on both `windows-latest` and `ubuntu-latest` at commit `a24736cb28233331ecefc108317e76cf7e1a5ef0`. Both jobs passed the root tests, separate science tests, release build, five-process demonstration and actual C ABI probe. This is automated CI evidence for that commit, separate from the local run and from independent review. [Saved run metadata](evidence/github-actions/run-40083832092.json), [job/step results](evidence/github-actions/jobs-40083832092.json) and [selected validation log lines](evidence/github-actions/selected-validation-lines-40083832092.log) accompany the capsule.

## Limits, license and review

Model predicates require supplied UT1 inputs and tolerance; they are not independent physical observation. Missing evidence and equality at the tolerance boundary are not strict counterproofs. Full native admission is excluded. Valid old history may be accepted; source-version binding, finite checkpoint bounds and full replay costs remain material. See [CLAIM_BOUNDARY.md](CLAIM_BOUNDARY.md).

The capsule uses [Apache-2.0](LICENSE), with retained third-party MIT/SOFA notices in [the sofars LICENSE](dependencies/vendor/sofars-0.6.1/LICENSE) and [extraction notice](dependencies/vendor/sofars-0.6.1/NOTICE.md). SOFA endorsement is not claimed.

Follow [REVIEW_PROTOCOL.md](REVIEW_PROTOCOL.md) for counterexamples. Artifact locations: [GitHub](https://github.com/gjatalegacy-create/AIGovern-Cyber-POC009-Knowledge-Memory) and [HF](https://huggingface.co/datasets/gjata-legacy/ess-mai-poc-009-knowledge-memory). [CITATION.cff](CITATION.cff) identifies the software; the separate [manuscript](paper/manuscript.md), *Scoped Model Support and Counterevidence Across Five Memory Cycles in ESS-MAI*, has the preprint identifier above and a [PDF](paper/scoped_knowledge_memory.pdf).

Independent critique is invited through the [public PREreview request](https://prereview.org/preprints/doi-10.5281-zenodo.23614714). A published request is not a completed independent review.

Research lineage: [ESS-MAI](https://github.com/gjatalegacy-create/ESS-MAI), [POC003–006 archive](https://doi.org/10.5281/zenodo.22750188), [POC007 software](https://doi.org/10.5281/zenodo.23596907), [POC007 preprint](https://doi.org/10.5281/zenodo.23597026), [POC008 software](https://doi.org/10.5281/zenodo.23613613), [POC008 preprint](https://doi.org/10.5281/zenodo.23613785). Their identifiers and results remain separate from POC009.

## Project, artifact and author links

[GitHub — source](https://github.com/gjatalegacy-create/AIGovern-Cyber-POC009-Knowledge-Memory) · [Hugging Face — public mirror](https://huggingface.co/datasets/gjata-legacy/ess-mai-poc-009-knowledge-memory) · [LinkedIn — author](https://www.linkedin.com/in/bledar-gjata-1a527030b/) · [Zenodo — software archive](https://doi.org/10.5281/zenodo.23614687) · [OpenAIRE — software metadata](https://explore.openaire.eu/search/result?pid=10.5281%2Fzenodo.23614687) · [ORCID — author](https://orcid.org/0009-0003-7854-266X)

POC009 is a bounded public research capsule derived from [ESS-MAI](https://github.com/gjatalegacy-create/ESS-MAI), not the full private AIGovern-Cyber runtime. The software archive and its separate manuscript keep distinct identifiers and claim boundaries. LinkedIn and ORCID identify author-declared profiles for Bledar Gjata (Gjata Legacy). The OpenAIRE DOI lookup returned the matching software metadata on 24 October 2026; indexing and these cross-links do not establish peer review, certification or endorsement.
