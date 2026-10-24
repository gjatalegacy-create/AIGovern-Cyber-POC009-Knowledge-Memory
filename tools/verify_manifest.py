"""Verify exact release bytes; no writes and no network."""
import hashlib
from pathlib import Path

root = Path(__file__).resolve().parents[1]
rows = (root / 'artifact_hashes.sha256').read_text(encoding='utf-8').splitlines()
for row in rows:
    expected, name = row.split('  ', 1)
    path = (root / name).resolve()
    if not path.is_relative_to(root) or hashlib.sha256(path.read_bytes()).hexdigest() != expected:
        raise SystemExit('MISMATCH: ' + name)
print(f'{len(rows)} release files match SHA-256 manifest')
