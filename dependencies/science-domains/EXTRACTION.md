# POC009 science dependency boundary

This is a selected source extraction of the actual science-domains ERA
evaluation, canonical query/candidate codecs and KiE predicates/measurement.
All original tests in lib.rs and kie.rs are retained. The unrelated Fisher
exact_tests module is omitted. Its exclusion is not a claim about Fisher tests.

CandidateAuthority remains None. The model computes support/counter relations
under supplied UT1 inputs and tolerances; it does not admit knowledge, provide
independent observation, write a Vault or confer native authority.

Only module omission, relative include paths, the Cargo path dependency and
standalone lint configuration change in the science crate. KiE source and the
three compiled sofars numerical sources are byte-exact copies. Original
provenance strings use logical paths `vendor/sofars-0.6.1/...`; their public
package location is `dependencies/vendor/sofars-0.6.1/...`.

The complete third-party LICENSE and extraction notice accompany that vendor
subset. See `science_provenance.json` for exact source/output hashes and ranges.
