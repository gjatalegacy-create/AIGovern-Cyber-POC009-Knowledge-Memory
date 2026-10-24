# Working method and assistance

Bledar Gjata, Independent Researcher in Albania, directs the ESS-MAI architecture, research question, terminology and interpretation. The author reports reviewing generated work through shell-based checks. Command logs establish particular executions; they do not show that every command was manually typed or every line independently audited.

OpenAI Codex assisted source inspection, extraction, adapter development, test preparation, documentation and consistency review. The environment identifies the assistant as GPT-6 based; no finer provider-verified model build is attached here. Multiple agent passes and AI agreement are not independent human review.

The workflow separates four records:

1. **Source lineage:** selected files/ranges, hashes, transformations, preserved numerical code and third-party notices. Source inputs are inspected without modification.
2. **Mechanism:** actual model evaluation and KiE checks, scoped content addressing, rebuilt memory, the legacy no-write FFI surface and new borrowed callback helper. Names are not evidence of native authority.
3. **Execution:** final artifact tests, five fresh-process cycles, exact comparison semantics, adverse results, toolchain and source/test/lock identities. The isolated science dependency's tests are a separate check.
4. **Publication/review:** frozen archive, distinct software/preprint identifiers, verified mirrors and voluntary invitations. Hosting, DOI registration and invitations do not establish scientific correctness or completed review.

The numerical example is a conformance/model comparison. Caller-supplied UT1 and provenance are bound into records, not independently authenticated. Repeating the model or reusing historical content does not create independent evidence. Stored positive and negative propositions remain separately scoped.

On 24 October 2026 the local root suite passed 32 unit and 18 integration tests (50 total), with zero failed or ignored. Separately, the extracted science suite passed 19 tests under locked offline dependencies and `-Dwarnings`. Its final validation used direct Rust 1.98.0 compiler/rustdoc binaries; an earlier rustup-shim run's path-canonicalization warning remains disclosed in the validation record.

The five-cycle root test compares complete operation records and memory against a persistent control, plus exact checkpoint bytes against uninterrupted PD at every cycle. Early failures exposed JSON floating-point round-trip loss at an equality boundary and invalid scientific input causing storage initialization or tail repair before rejection. The fixes enabled precise float parsing and moved model evaluation before storage opening; the rejection assertions were retained. These are software corrections, not evidence of independent scientific validation.

Read-only verification confirmed all seven source/output records, four generated files and the extractor against `science_provenance.json`. The original KiE source already contains `measure_kie` and `KieModelMeasurement`; they were not added by this extraction. The final root test run again passed 50 tests, with 0.88 seconds for integration tests; the locked offline release build passed in 21.04 seconds using direct toolchain binaries. Logs are `cargo_test_final.log` and `cargo_release.log` in the execution evidence. Software DOI `10.5281/zenodo.23614687` and preprint DOI `10.5281/zenodo.23614714` are reserved, with publication pending. CI and external review must be recorded separately when verified.
