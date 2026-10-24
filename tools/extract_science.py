"""Extract the bounded ERA/KiE science closure; never modify source inputs.

Inputs are explicit local directories. Public provenance uses logical paths,
not the caller's machine-specific paths. The destination is this tool's package.
"""
from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import struct


def digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def replace_exact(data: bytes, old: bytes, new: bytes, count: int) -> bytes:
    actual = data.count(old)
    if actual != count:
        raise ValueError(f"expected {count} matches of {old!r}, found {actual}")
    return data.replace(old, new)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--science-source", required=True, type=Path)
    parser.add_argument("--sofars-source", required=True, type=Path)
    args = parser.parse_args()
    package = Path(__file__).resolve().parents[1]
    destination = package / "dependencies"
    source_science = args.science_source.resolve(strict=True)
    source_vendor = args.sofars_source.resolve(strict=True)
    files: list[dict] = []
    source_checks: list[tuple[Path, str]] = []

    def write_derived(source: Path, source_logical: str, output: str,
                      transforms: list[dict], data: bytes | None = None,
                      ranges: list[list[int]] | None = None) -> None:
        original = source.read_bytes()
        content = original if data is None else data
        target = destination / output
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(content)
        original_sha = digest(original)
        source_checks.append((source, original_sha))
        files.append({
            "source_path_logical": source_logical,
            "source_sha256": original_sha,
            "source_bytes": len(original),
            "selected_line_ranges_inclusive": ranges or [[1, len(original.splitlines())]],
            "transformations": transforms,
            "output_path": target.relative_to(package).as_posix(),
            "output_sha256": digest(content),
            "output_bytes": len(content),
            "byte_exact_copy": content == original,
        })

    lib_path = source_science / "src/lib.rs"
    original_lib = lib_path.read_bytes()
    lib = replace_exact(original_lib, b"pub mod exact_tests;\r\n", b"", 1)
    lib = replace_exact(lib, b'"../../../vendor/sofars-0.6.1/',
                        b'"../../vendor/sofars-0.6.1/', 4)
    if b"pub enum CandidateAuthority { None }" not in lib:
        raise ValueError("CandidateAuthority::None source invariant changed")
    write_derived(lib_path, "science-domains/src/lib.rs", "science-domains/src/lib.rs", [
        {"operation": "omit line 4", "text": "pub mod exact_tests;",
         "reason": "Unrelated Fisher module excluded from the ERA/KiE closure."},
        {"operation": "replace include_bytes path prefix", "count": 4,
         "from": "../../../vendor/sofars-0.6.1/", "to": "../../vendor/sofars-0.6.1/",
         "reason": "Resolve exact numerical source bytes in sibling dependencies/vendor."},
    ], lib, [[1, 3], [5, len(original_lib.splitlines())]])
    write_derived(source_science / "src/kie.rs", "science-domains/src/kie.rs",
                  "science-domains/src/kie.rs", [])

    original_cargo = (source_science / "Cargo.toml").read_bytes()
    cargo = replace_exact(original_cargo, b'path = "../../vendor/sofars-0.6.1"',
                          b'path = "../vendor/sofars-0.6.1"', 1)
    cargo_newline = b"\r\n" if b"\r\n" in original_cargo else b"\n"
    cargo = replace_exact(cargo, b"[lints]" + cargo_newline + b"workspace = true",
                          b"[lints.rust]" + cargo_newline + b'warnings = "deny"', 1)
    write_derived(source_science / "Cargo.toml", "science-domains/Cargo.toml",
                  "science-domains/Cargo.toml", [
        {"operation": "replace path dependency", "from": "../../vendor/sofars-0.6.1",
         "to": "../vendor/sofars-0.6.1"},
        {"operation": "replace workspace lint inheritance",
         "from": "[lints] workspace = true", "to": '[lints.rust] warnings = "deny"'},
    ], cargo)

    model_paths = ["src/erst/era00.rs", "src/consts.rs", "src/vm/anp.rs"]
    for path in model_paths + ["LICENSE"]:
        write_derived(source_vendor / path, "vendor/sofars-0.6.1/" + path,
                      "vendor/sofars-0.6.1/" + path, [])

    generated = {
        "vendor/sofars-0.6.1/Cargo.toml": '''# Minimal POC009 wrapper; numerical files are unchanged upstream bytes.
[package]
name = "sofars"
version = "0.6.1"
edition = "2024"
authors = ["Huxulm <huxulm@gmail.com>"]
description = "POC009 minimal ERA subset of sofars 0.6.1; not the complete upstream crate"
license-file = "LICENSE"
repository = "https://github.com/astro-xao/sofars"

[lib]
name = "sofars"
path = "src/lib.rs"
''',
        "vendor/sofars-0.6.1/src/lib.rs": '''//! POC009 minimal module wrapper, not the complete upstream sofars crate.
//! Numerical files retain exact upstream bytes; see LICENSE and NOTICE.md.
pub mod consts;
pub mod erst {
    mod era00;
    pub use era00::era00;
}
pub mod vm {
    mod anp;
    pub use anp::anp;
}
''',
        "vendor/sofars-0.6.1/NOTICE.md": '''# Minimal sofars 0.6.1 dependency extraction

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
''',
        "science-domains/EXTRACTION.md": '''# POC009 science dependency boundary

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
''',
    }
    generated_records = []
    for path, text in generated.items():
        target = destination / path
        target.parent.mkdir(parents=True, exist_ok=True)
        content = text.encode("utf-8")
        target.write_bytes(content)
        generated_records.append({"path": target.relative_to(package).as_posix(),
                                  "sha256": digest(content), "classification": "new extraction wrapper or notice"})

    compiled = hashlib.sha256(b"ESS-MAI/SCIENCE/COMPILED-MODEL/1\0")
    for path in model_paths:
        raw = (source_vendor / path).read_bytes()
        encoded_path = path.encode("utf-8")
        compiled.update(struct.pack("<I", len(encoded_path)))
        compiled.update(encoded_path)
        compiled.update(struct.pack("<Q", len(raw)))
        compiled.update(raw)
    for path, expected in source_checks:
        if digest(path.read_bytes()) != expected:
            raise ValueError("source changed during extraction")
    provenance = {
        "schema": 1,
        "artifact": "POC009 actual ERA/KiE science dependency extraction",
        "source_root_handling": "Explicit local input directories; absolute machine paths excluded.",
        "authority": "CandidateAuthority::None unchanged; model computation is not independent observation or knowledge admission.",
        "retained_test_declarations": lib.count(b"#[test]") + (source_science / "src/kie.rs").read_bytes().count(b"#[test]"),
        "execution_status": "Extraction recorded; test outcomes must be recorded separately after execution.",
        "excluded": ["science-domains/src/exact_tests.rs (Fisher module)", "Other sofars algorithms, tests and examples"],
        "extractor": "tools/extract_science.py",
        "extractor_sha256": digest(Path(__file__).read_bytes()),
        "compiled_model_sha256": compiled.hexdigest(),
        "source_files_unchanged": True,
        "files": files,
        "generated_files": generated_records,
    }
    (package / "science_provenance.json").write_text(
        json.dumps(provenance, indent=2) + "\n", encoding="utf-8", newline="\n")
    print(json.dumps({"source_files": len(files), "generated_files": len(generated_records),
                      "retained_test_declarations": provenance["retained_test_declarations"],
                      "compiled_model_sha256": compiled.hexdigest(),
                      "source_files_unchanged": True}))


if __name__ == "__main__":
    main()
