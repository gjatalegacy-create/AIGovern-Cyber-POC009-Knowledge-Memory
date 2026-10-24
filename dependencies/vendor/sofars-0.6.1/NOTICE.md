# Minimal sofars 0.6.1 dependency extraction

The three numerical files `src/erst/era00.rs`, `src/consts.rs` and
`src/vm/anp.rs` are unchanged bytes from the selected upstream sofars 0.6.1
source. The Cargo manifest and module wrapper are newly narrowed for POC009;
the remainder of the upstream crate is omitted. Numerical algorithms and
constants were not modified by this extraction.

This package uses routines and computations derived from software provided
by the IAU SOFA Board under license. It does not itself constitute software
provided by or endorsed by SOFA. The sofars Rust implementation is attributed
to its upstream authors; no authorship of the original SOFA algorithms is claimed.

`LICENSE` is copied byte-for-byte and contains both the Astro XAO permission
notice and the full IAU SOFA software license. Those notices and requirements
remain applicable; the surrounding capsule's license does not replace them.

Upstream: https://github.com/astro-xao/sofars
The original source's declared provider commit is
`1cb4bdf1f84176ba0128bfa3aa82c205e75716e7`. Exact selected-file hashes and
wrapper changes are recorded in the package's `science_provenance.json`.
