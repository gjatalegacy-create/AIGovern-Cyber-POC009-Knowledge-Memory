#!/usr/bin/env python3
"""Extract the actual process-local FFI gate from an explicitly supplied runtime.

The source runtime is read only. --check verifies public outputs without it.
The added callback adapter does not confer a native verdict or a write grant.
"""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import tempfile


ROOT = Path(__file__).resolve().parents[1]
MANIFEST = ROOT / "ffi_provenance.json"
OUTPUT = ROOT / "src/ffi_gate.rs"

HEADER = r'''//! Source-derived process-local one-use FFI gate, plus a disclosed callback adapter.
//!
//! The original extern C write surface consumes a handle but returns
//! CompatibilityHold (-8) for nonempty data. It never writes knowledge.
//! `with_borrowed_payload` is NEW POC009 glue: it uses the source gate and borrowed
//! slice, then lets the caller independently recompute and persist model evidence.
//! No native verdict, receipt, capability lineage, or knowledge authority is copied
//! into that caller. The callback's own checks determine its effects.
//!
//! Handles are process-local. Their generation resets when the process restarts;
//! never persist, restore, or transfer them across process lifetimes. Nonces are
//! deterministic labels, not secrets or authentication. The source registry has no
//! revocation operation and retains consumed slots; it is not a hostile in-process
//! sandbox. Pointer validity remains the unsafe caller's responsibility.

'''

BRIDGE = r'''
/// NEW POC009 adapter over the exact source gate and source borrowed-payload view.
///
/// The source handle is consumed before payload validation and before the callback.
/// Copying its numeric fields does not copy the registry's one-use state. The
/// callback receives bytes only; it must perform its own model/scope/write checks.
/// A callback error or panic does not restore the consumed handle.
///
/// # Safety
/// For a nonnull, nonempty payload, `data_ptr` must identify one live allocation
/// readable for `data_len` bytes, with `data_len <= isize::MAX`. It must remain
/// valid and unmodified for the whole callback. The caller retains ownership.
/// Null or empty payloads are rejected without dereferencing the pointer.
pub unsafe fn with_borrowed_payload<T>(
    handle: CapHandle,
    data_ptr: *const u8,
    data_len: usize,
    use_bytes: impl FnOnce(&[u8]) -> T,
) -> Result<T, i32> {
    let _token = gate()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .validate_and_burn(handle)
        .map_err(|error| match error {
            FfiErr::AlreadyConsumed => -1,
            FfiErr::HandleNotFound => -2,
            FfiErr::NonceMismatch => -3,
        })?;
    // SAFETY: the caller provides the allocation and lifetime required above.
    let bytes = unsafe { borrow_ffi_payload(data_ptr, data_len) }?;
    Ok(use_bytes(bytes))
}
'''

BRIDGE_TESTS = r'''

    // NEW adapter tests; the tests above are extracted from the private runtime.
    #[test]
    fn callback_preserves_every_byte_and_pointer_then_denies_reuse() {
        let data = (0..=u8::MAX).collect::<Vec<_>>();
        let handle = sovereign_issue_capability(S_SHADOW_WRITE);
        // SAFETY: data remains live and immutable throughout both calls.
        let result = unsafe { with_borrowed_payload(handle, data.as_ptr(), data.len(), |view| {
            assert_eq!(view.as_ptr(), data.as_ptr());
            assert_eq!(view, data.as_slice());
            view.len()
        }) };
        assert_eq!(result, Ok(256));
        assert_eq!(unsafe { with_borrowed_payload::<()>(handle, data.as_ptr(), data.len(), |_| {
            panic!("a consumed handle must not reach the callback")
        }) }, Err(-1));
    }

    #[test]
    fn empty_and_null_calls_consume_the_grant_without_callback() {
        for (pointer, length) in [(std::ptr::null(), 0), (std::ptr::null(), 7), (b"x".as_ptr(), 0)] {
            let handle = sovereign_issue_capability(S_SHADOW_WRITE);
            // SAFETY: these invalid/empty shapes are rejected before any borrow.
            assert_eq!(unsafe { with_borrowed_payload::<()>(handle, pointer, length, |_| {
                panic!("empty input must not reach the callback")
            }) }, Err(-5));
            assert_eq!(unsafe { sovereign_validate_and_write(handle, std::ptr::null(), 0) }, -1);
        }
    }

    #[test]
    fn wrong_nonce_does_not_consume_the_real_handle() {
        let data = b"source-bound";
        let handle = sovereign_issue_capability(S_SHADOW_WRITE);
        let forged = CapHandle { gen: handle.gen, nonce: handle.nonce ^ 1 };
        // SAFETY: data remains live and immutable throughout both calls.
        assert_eq!(unsafe { with_borrowed_payload(forged, data.as_ptr(), data.len(), |_| ()) }, Err(-3));
        assert_eq!(unsafe { with_borrowed_payload(handle, data.as_ptr(), data.len(), |_| ()) }, Ok(()));
    }

    #[test]
    fn callback_error_and_panic_do_not_restore_consumed_grants() {
        let data = b"source-bound";
        let first = sovereign_issue_capability(S_SHADOW_WRITE);
        // SAFETY: data remains live and immutable throughout each callback.
        let result = unsafe { with_borrowed_payload(first, data.as_ptr(), data.len(), |_| {
            Err::<(), _>("model rejected")
        }) };
        assert_eq!(result, Ok(Err("model rejected")));
        assert_eq!(unsafe { sovereign_validate_and_write(first, data.as_ptr(), data.len()) }, -1);
        let second = sovereign_issue_capability(S_SHADOW_WRITE);
        let panicked = std::panic::catch_unwind(|| unsafe {
            with_borrowed_payload::<()>(second, data.as_ptr(), data.len(), |_| panic!("callback failed"))
        });
        assert!(panicked.is_err());
        assert_eq!(unsafe { sovereign_validate_and_write(second, data.as_ptr(), data.len()) }, -1);
    }

    #[test]
    fn copied_handles_racing_in_one_process_reach_exactly_one_callback() {
        let handle = sovereign_issue_capability(S_SHADOW_WRITE);
        let results = std::thread::scope(|scope| {
            let threads = (0..16).map(|_| scope.spawn(move || {
                let bytes = b"same-process";
                // SAFETY: static immutable bytes outlive the call.
                unsafe { with_borrowed_payload(handle, bytes.as_ptr(), bytes.len(), |_| ()) }
            })).collect::<Vec<_>>();
            threads.into_iter().map(|thread| thread.join().unwrap()).collect::<Vec<_>>()
        });
        assert_eq!(results.iter().filter(|result| **result == Ok(())).count(), 1);
        assert_eq!(results.iter().filter(|result| **result == Err(-1)).count(), 15);
    }
'''


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def selected(source_root: Path, relative: str, ranges: list[tuple[int, int]]) -> tuple[str, dict]:
    raw = (source_root / relative).read_bytes()
    lines = raw.decode("utf-8").splitlines(keepends=True)
    for start, end in ranges:
        if not 1 <= start <= end <= len(lines):
            raise ValueError(f"Invalid extraction range {relative}:{start}-{end}")
    value = "".join("".join(lines[start - 1:end]) for start, end in ranges)
    entry = {
        "source_path_relative_to_runtime": relative,
        "source_sha256": sha(raw),
        "source_bytes": len(raw),
        "source_lines": len(lines),
        "selected_line_ranges_inclusive": [list(pair) for pair in ranges],
        "selected_source_utf8_sha256": sha(value.encode("utf-8")),
    }
    return value.replace("\r\n", "\n"), entry


def extract(source_root: Path, rustfmt: str) -> dict:
    gate, gate_entry = selected(source_root, "shadow/src/sovereign_ffi_gate.rs", [
        (22, 25), (31, 47), (54, 54), (61, 67), (95, 106), (113, 148),
        (154, 157), (239, 249), (267, 294), (296, 340),
    ])
    cfg = '#[cfg(any(test, feature = "legacy_ffi_compat"))]\n'
    if gate.count(cfg) != 4:
        raise ValueError("Expected four original legacy FFI feature gates")
    gate = gate.replace(cfg, "")
    old_lineage = "crate::knowledge_lineage::Lineage::derive_capability"
    if gate.count(old_lineage) != 1:
        raise ValueError("Expected one original lineage dependency")
    gate = gate.replace(old_lineage, "capability_lineage::Lineage::derive_capability")
    gate_entry["transformations"] = [
        "Omit native VerificationReceipt, seal_verified_output, and receipt generation; preserve source registry, one-use CAS, unsafe byte borrow, and both legacy extern C functions.",
        "Remove four legacy_ffi_compat cfg attributes to expose this narrowly selected demonstration ABI in the standalone crate.",
        "Redirect the one lineage dependency to the extracted capability_lineage module; its original formulas and fields are unchanged.",
        "Omit the historical header and obsolete write-success return-code prose; the implementation still returns CompatibilityHold (-8) and performs no write.",
    ]
    lineage, lineage_entry = selected(source_root, "shadow/src/knowledge_lineage.rs", [
        (44, 62), (97, 112), (143, 159), (169, 176),
    ])
    # Reintroduce the original impl wrapper around exactly selected methods.
    split = lineage.index("    /// Ndërton gjurmën")
    lineage = lineage[:split] + "impl Lineage {\n" + lineage[split:] + "}\n"
    lineage_entry["transformations"] = [
        "Extract FNV helpers, MIN_CHAIN, exact Lineage data fields, derive_capability, and is_traceable only; reintroduce the impl wrapper.",
        "Place source items in public capability_lineage module so unused native data fields remain unchanged; this module is integrity/lineage data, not a native admission authority.",
    ]
    tests, tests_entry = selected(source_root, "shadow/src/sovereign_ffi_gate.rs", [(347, 384), (467, 497)])
    tests_entry["transformations"] = [
        "Retain seven original gate tests; omit the native-verification-receipt test and its authority dependencies.",
    ]
    result = HEADER + gate + "\n/// Source capability-lineage primitives used only by the held compatibility path.\npub mod capability_lineage {\n" + lineage + "}\n" + BRIDGE
    result += "\n#[cfg(test)]\nmod tests {\n    use super::*;\n" + tests + BRIDGE_TESTS + "}\n"
    rustfmt_version = subprocess.check_output([rustfmt, "--version"], text=True).strip()
    with tempfile.TemporaryDirectory(prefix="poc009-ffi-extract-") as temporary:
        staged = Path(temporary) / "ffi_gate.rs"
        staged.write_text(result, encoding="utf-8", newline="\n")
        subprocess.run([rustfmt, "--edition", "2021", "--config", "skip_children=true,newline_style=Unix", str(staged)], check=True)
        output = staged.read_bytes()
    OUTPUT.parent.mkdir(parents=True, exist_ok=True)
    OUTPUT.write_bytes(output)
    return {
        "schema": 1,
        "artifact": "POC009 source-derived FFI gate and new callback adapter",
        "source_product": "AIGovern-Cyber (ESS-MAI / Gjata Legacy lineage)",
        "source_root_logical": "codex_zvn/runtime",
        "source_root_handling": "Explicit operator argument; no machine-specific absolute path is published.",
        "sources": [gate_entry, lineage_entry, tests_entry],
        "new_code": [
            "English module/API boundary documentation.",
            "with_borrowed_payload callback adapter: source gate validation/burn, source byte borrow, no native verdict or write function.",
            "Five adapter tests for all-byte/pointer preservation, empty/null rejection, wrong nonce, callback errors/panic, and a 16-thread copied-handle race.",
        ],
        "limitations": [
            "Process-local lifetime only; generation restarts in a new process, consumed slots are retained, and there is no revoke operation.",
            "Deterministic nonce is not secret authentication; public issuance does not authenticate a principal.",
            "Unsafe pointer validity is the caller's obligation; this is not isolation against hostile code in the same address space.",
            "Legacy sovereign_validate_and_write stays no-write CompatibilityHold; the new adapter delegates effects to an independent verifier callback.",
        ],
        "formatting": "Normalize line endings to LF and rustfmt with Rust 2021 edition.",
        "rustfmt": rustfmt_version,
        "extractor": "tools/extract_ffi.py",
        "extractor_sha256": sha(Path(__file__).read_bytes()),
        "output_path": "src/ffi_gate.rs",
        "output_sha256": sha(output),
        "output_bytes": len(output),
        "output_lines": len(output.splitlines()),
    }


def check() -> None:
    manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
    if sha(OUTPUT.read_bytes()) != manifest["output_sha256"]:
        raise SystemExit("Provenance hash mismatch: src/ffi_gate.rs")
    if sha(Path(__file__).read_bytes()) != manifest["extractor_sha256"]:
        raise SystemExit("Provenance hash mismatch: tools/extract_ffi.py")
    print("PASS: FFI output and extractor hashes")


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source-root", type=Path, help="Read-only source runtime directory")
    parser.add_argument("--rustfmt", default=shutil.which("rustfmt"), help="Path to rustfmt")
    parser.add_argument("--check", action="store_true", help="Verify public hashes without private source")
    args = parser.parse_args()
    if args.check:
        check()
        return
    if args.source_root is None or not args.rustfmt:
        parser.error("Extraction requires --source-root and rustfmt")
    manifest = extract(args.source_root.resolve(), args.rustfmt)
    MANIFEST.write_text(json.dumps(manifest, indent=2, ensure_ascii=False) + "\n", encoding="utf-8", newline="\n")
    check()


if __name__ == "__main__":
    main()
