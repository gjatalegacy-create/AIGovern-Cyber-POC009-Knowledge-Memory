#!/usr/bin/env python3
"""Run five explicit public fixture cycles, starting a fresh binary process each time.

Usage: python tools/demo.py --binary PATH --output-dir NEW_DIRECTORY
The output must not exist and must be outside this package. It includes a copied
runner, exact inputs, actual relative commands, raw process output, and demo.wal.
This is scoped computational memory, not native or physical knowledge admission.
"""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import sys
import time


ROOT = Path(__file__).resolve().parents[1]
FIXTURES = ["01-missing.json", "02-counter.json", "03-support.json",
            "04-counter-repeat.json", "05-support-repeat.json"]
STATUSES = ["missing_evidence", "model_counterproof", "model_support",
            "model_counterproof", "model_support"]
COUNTS = [(0, 0, 1), (0, 1, 1), (1, 1, 1), (1, 1, 1), (1, 1, 1)]


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def write_json(path: Path, value: object) -> None:
    path.write_text(json.dumps(value, indent=2, ensure_ascii=False) + "\n", encoding="utf-8", newline="\n")


def create_output(path: Path) -> Path:
    path = path.resolve()
    if path == ROOT or ROOT in path.parents:
        raise ValueError("Output must be outside the source package")
    path.mkdir(parents=True, exist_ok=False)
    return path


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", required=True, type=Path)
    parser.add_argument("--output-dir", required=True, type=Path)
    args = parser.parse_args()
    binary = args.binary.resolve(strict=True)
    if not binary.is_file():
        parser.error("--binary must be a file")
    try:
        output = create_output(args.output_dir)
    except (OSError, ValueError) as error:
        parser.error(str(error))
    (output / "runner").mkdir()
    (output / "inputs").mkdir()
    runner_name = "poc009-memory.exe" if binary.suffix.lower() == ".exe" else "poc009-memory"
    runner = "runner/" + runner_name
    shutil.copy2(binary, output / runner)
    checks: list[dict] = []
    commands: list[dict] = []
    responses: list[dict] = []
    result = {
        "schema": "POC009/DEMO/1", "status": "RUNNING", "cycle_count": 5,
        "fresh_process_per_step": True, "working_directory": ". (this output directory)",
        "runner": runner, "runner_sha256": sha(binary.read_bytes()),
        "script_sha256": sha(Path(__file__).read_bytes()), "inputs": [],
        "commands": commands, "checks": checks,
        "scope": "Two distinct model-conditional propositions and one missing-input event. No native verdict or physical observation. Repeated results are not independent evidence.",
    }

    def check(name: str, passed: bool) -> None:
        checks.append({"name": name, "passed": bool(passed)})
        if not passed:
            raise AssertionError(name)

    def invoke(label: str, arguments: list[str]) -> dict:
        command = [runner, *arguments]
        started = time.monotonic()
        # Windows resolves the executable before applying cwd. Keep the actual
        # argv portable while resolving this same copied runner explicitly.
        with subprocess.Popen(command, executable=str(output / runner), cwd=output, stdout=subprocess.PIPE,
                              stderr=subprocess.PIPE) as process:
            process_id = process.pid
            try:
                stdout, stderr = process.communicate(timeout=60)
                code, timed_out = process.returncode, False
            except subprocess.TimeoutExpired:
                process.kill()
                stdout, stderr = process.communicate()
                code, timed_out = process.returncode, True
        stdout_path, stderr_path = label + ".stdout", label + ".stderr"
        (output / stdout_path).write_bytes(stdout)
        (output / stderr_path).write_bytes(stderr)
        commands.append({"label": label, "argv": command, "cwd": ".", "process_id": process_id, "returncode": code,
                         "timed_out": timed_out, "elapsed_seconds": round(time.monotonic() - started, 6),
                         "stdout": stdout_path, "stderr": stderr_path,
                         "stdout_sha256": sha(stdout), "stderr_sha256": sha(stderr)})
        check(label + ": process completed successfully", code == 0 and not timed_out)
        value = json.loads(stdout)
        check(label + ": JSON object output", isinstance(value, dict))
        return value

    try:
        previous_memory = {"positive_candidates": 0, "negative_candidates": 0,
                           "missing_evidence_events": 0, "unresolved_events": 0, "candidates": []}
        previous_record = "0" * 64
        texts: list[str] = []
        candidate_ids: dict[str, str] = {}
        for index, name in enumerate(FIXTURES, 1):
            raw = (ROOT / "fixtures" / name).read_bytes()
            fixture = json.loads(raw)
            input_path = "inputs/" + name
            (output / input_path).write_bytes(raw)
            result["inputs"].append({"path": input_path, "sha256": sha(raw), "bytes": len(raw)})
            value = invoke(f"{index:02d}-step", ["step", "demo.wal", input_path])
            responses.append(value)
            with (output / "transcript.jsonl").open("a", encoding="utf-8", newline="\n") as stream:
                stream.write(json.dumps({"cycle": index, "input": input_path, "output": value}, ensure_ascii=False) + "\n")
            texts.append(fixture["text"])
            prefix = f"cycle {index}: "
            check(prefix + "expected current model classification", value["model_result"] == STATUSES[index - 1])
            check(prefix + "revision and PD turn continuity", value["revision"] == index and value["total_turns"] == index)
            check(prefix + "exact ordered user text retained", value["user_turns"] == texts)
            check(prefix + "previous record chain", value["previous_record"] == previous_record)
            check(prefix + "memory survives preceding process exit", value["memory_before"] == previous_memory)
            check(prefix + "raw input bytes bound", value["exact_input_sha256"] == sha(raw))
            check(prefix + "borrowed ingress preserves pointer and bytes", value["ffi_same_pointer"] is True and value["ffi_exact_bytes"] is True)
            check(prefix + "same-process grant reuse denied", value["ffi_reuse_code"] == -1)
            check(prefix + "no native verdict or transferred authority", value["knowledge_verdict"] is None and value["authority_transferred"] is False)
            check(prefix + "model execution matches input availability", value["science_executed"] == (index != 1))
            check(prefix + "exact prior retrieval", value["retrieved_before"]["found"] == (index >= 4))
            check(prefix + "no tail repair on clean run", value["recovered_tail_bytes"] == 0)
            memory = value["memory_after"]
            p, n, missing = COUNTS[index - 1]
            check(prefix + "separate support counterproof and missing counts", (memory["positive_candidates"], memory["negative_candidates"], memory["missing_evidence_events"]) == (p, n, missing))
            check(prefix + "missing is not numerical equality", memory["unresolved_events"] == 0)
            if index > 1:
                status = value["model_result"]
                if status in candidate_ids:
                    check(prefix + "repeat keeps exact candidate identity", value["candidate_sha256"] == candidate_ids[status])
                    check(prefix + "previous occurrence retrieved", value["retrieved_before"]["occurrences"] == 1)
                else:
                    candidate_ids[status] = value["candidate_sha256"]
            else:
                check(prefix + "missing input manufactures no candidate", value["candidate_sha256"] == "0" * 64 and not memory["candidates"])
            previous_memory, previous_record = memory, value["record_id"]
        rows = previous_memory["candidates"]
        check("final: exactly two different candidate identities", len(rows) == 2 and len({row["candidate_sha256"] for row in rows}) == 2)
        check("final: support and counterproof name different propositions", len({row["proposition_sha256"] for row in rows}) == 2)
        check("final: each proposition has two distinct durable occurrences", all(row["occurrences"] == 2 and len(set(row["origin_record_sha256"])) == 2 for row in rows))
        before_inspect = (output / "demo.wal").read_bytes()
        inspected = invoke("06-inspect", ["inspect", "demo.wal", "public-era00-five-cycles"])
        check("final: fresh inspect reconstructs exact memory", inspected["memory"] == previous_memory)
        check("final: fresh inspect preserves head", inspected["record_id"] == previous_record)
        check("final: clean inspection leaves WAL bytes unchanged", (output / "demo.wal").read_bytes() == before_inspect)
        result["status"] = "PASS"
        result["completed_cycles"] = len(responses)
        result["final_memory"] = previous_memory
    except Exception as error:
        result["status"] = "FAIL"
        result["completed_cycles"] = len(responses)
        result["failure"] = f"{type(error).__name__}: {error}"
    finally:
        wal = output / "demo.wal"
        if wal.exists():
            data = wal.read_bytes()
            result["wal"] = {"path": "demo.wal", "bytes": len(data), "sha256": sha(data)}
        result["passed_checks"] = sum(check["passed"] for check in checks)
        result["failed_checks"] = sum(not check["passed"] for check in checks)
        write_json(output / "RESULT.json", result)
    print(json.dumps({"status": result["status"], "completed_cycles": result["completed_cycles"],
                      "passed_checks": result["passed_checks"], "failed_checks": result["failed_checks"]}))
    return 0 if result["status"] == "PASS" else 1


if __name__ == "__main__":
    sys.exit(main())
