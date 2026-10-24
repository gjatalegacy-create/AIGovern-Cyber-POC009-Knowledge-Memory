#!/usr/bin/env python3
"""Exercise the actual exported C ABI with ctypes, separately from the Rust demo.

Usage: python tools/ffi_probe.py --library PATH --output-dir NEW_DIRECTORY
The output must be new and outside this package. The original nonempty write
surface returns CompatibilityHold (-8), never successful knowledge admission.
Issuance accepts caller-chosen labels; a mismatched existing-handle nonce fails.
"""

from __future__ import annotations

import argparse
import concurrent.futures
import ctypes
import hashlib
import json
import os
from pathlib import Path
import shutil
import sys


ROOT = Path(__file__).resolve().parents[1]


class CapHandle(ctypes.Structure):
    _fields_ = [("gen", ctypes.c_uint64), ("nonce", ctypes.c_uint64)]


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def fnv(data: bytes) -> int:
    result = 14695981039346656037
    for byte in data:
        result = ((result ^ byte) * 1099511628211) & ((1 << 64) - 1)
    return result


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--library", required=True, type=Path)
    parser.add_argument("--output-dir", required=True, type=Path)
    args = parser.parse_args()
    library = args.library.resolve(strict=True)
    output = args.output_dir.resolve()
    if output == ROOT or ROOT in output.parents:
        parser.error("Output must be outside the source package")
    try:
        output.mkdir(parents=True, exist_ok=False)
    except OSError as error:
        parser.error(str(error))
    (output / "runner").mkdir()
    library_name = "runner/" + library.name
    shutil.copy2(library, output / library_name)
    checks: list[dict] = []
    calls: list[dict] = []
    result = {"schema": "POC009/C-ABI-PROBE/1", "status": "RUNNING",
              "library": library_name, "library_sha256": sha(library.read_bytes()),
              "script_sha256": sha(Path(__file__).read_bytes()),
              "load_call": {"function": "ctypes.CDLL", "argument": library_name, "cwd": ". (this output directory)"},
              "calls": calls, "checks": checks,
              "scope": "Real C ABI invocation of source issue/validate exports. The valid nonempty path is a consumed grant plus CompatibilityHold, not a knowledge write. No revoke or cross-process protection is asserted."}

    def check(name: str, passed: bool) -> None:
        checks.append({"name": name, "passed": bool(passed)})
        if not passed:
            raise AssertionError(name)

    previous_directory = Path.cwd()
    try:
        os.chdir(output)
        api = ctypes.CDLL(library_name)
        issue = api.sovereign_issue_capability
        issue.argtypes, issue.restype = [ctypes.c_uint64], CapHandle
        validate = api.sovereign_validate_and_write
        validate.argtypes, validate.restype = [CapHandle, ctypes.c_void_p, ctypes.c_size_t], ctypes.c_int32
        check("repr(C) handle is exactly two u64 fields", ctypes.sizeof(CapHandle) == 16 and CapHandle.nonce.offset == 8)
        label = fnv(b"SHADOW_WRITE_KERNEL") ^ fnv(b"GJATA_LEGACY")
        data = bytes(range(256))
        buffer = ctypes.create_string_buffer(data, len(data))

        def issued(seal: int, name: str) -> CapHandle:
            handle = issue(seal)
            calls.append({"name": name, "symbol": "sovereign_issue_capability", "module_seal": seal,
                          "returned": {"gen": handle.gen, "nonce": handle.nonce}})
            check(name + ": issued data handle", handle.gen != 0 and handle.nonce == seal)
            return handle

        def called(handle: CapHandle, name: str, pointer=None, length=None) -> int:
            pointer = ctypes.cast(buffer, ctypes.c_void_p) if pointer is None else pointer
            length = len(data) if length is None else length
            code = validate(handle, pointer, length)
            calls.append({"name": name, "symbol": "sovereign_validate_and_write",
                          "handle": {"gen": handle.gen, "nonce": handle.nonce},
                          "payload": "null" if not pointer else "all-byte fixture", "length": length, "returncode": code})
            return code

        original = issued(label, "issue valid handle")
        copied = CapHandle.from_buffer_copy(bytes(original))
        check("nonempty direct write is held", called(original, "first nonempty use") == -8)
        check("copied handle second use is refused", called(copied, "copied handle reuse") == -1)
        check("third use remains refused", called(original, "third use") == -1)
        fresh = issued(label, "issue for nonce test")
        forged = CapHandle(fresh.gen, fresh.nonce ^ 1)
        check("wrong nonce is refused", called(forged, "forged nonce") == -3)
        check("wrong nonce did not consume actual handle", called(fresh, "original after wrong nonce") == -8)
        check("unknown generation is refused", called(CapHandle((1 << 64) - 1, label), "unknown generation") == -2)
        empty = issued(label, "issue for empty test")
        check("empty payload is refused", called(empty, "empty payload", ctypes.c_void_p(), 0) == -5)
        check("empty rejection consumed its handle", called(empty, "reuse after empty") == -1)
        arbitrary = issued(0, "issue caller-selected zero label")
        check("caller-chosen label is not authentication and still cannot write", called(arbitrary, "arbitrary label held") == -8)
        racing = issued(label, "issue for copied-handle race")
        # ctypes.CDLL releases the GIL around each call. Every thread owns its own
        # numeric copy of the handle; the source Rust registry owns consumption.
        def racer(_: int) -> int:
            return validate(CapHandle(racing.gen, racing.nonce), ctypes.cast(buffer, ctypes.c_void_p), len(data))
        with concurrent.futures.ThreadPoolExecutor(max_workers=16) as pool:
            race_codes = list(pool.map(racer, range(16)))
        calls.append({"name": "16 copied handles in concurrent C calls", "symbol": "sovereign_validate_and_write",
                      "handle": {"gen": racing.gen, "nonce": racing.nonce}, "length": len(data), "returncodes": race_codes})
        check("concurrent copies consume exactly once", race_codes.count(-8) == 1 and race_codes.count(-1) == 15)
        check("C calls leave caller-owned byte buffer unchanged", buffer.raw == data)
        result["fixture"] = {"bytes": len(data), "sha256": sha(data), "unchanged": buffer.raw == data}
        result["status"] = "PASS"
    except Exception as error:
        result["status"] = "FAIL"
        result["failure"] = f"{type(error).__name__}: {error}"
    finally:
        os.chdir(previous_directory)
        result["passed_checks"] = sum(check["passed"] for check in checks)
        result["failed_checks"] = sum(not check["passed"] for check in checks)
        (output / "RESULT.json").write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8", newline="\n")
    print(json.dumps({"status": result["status"], "passed_checks": result["passed_checks"], "failed_checks": result["failed_checks"]}))
    return 0 if result["status"] == "PASS" else 1


if __name__ == "__main__":
    sys.exit(main())
