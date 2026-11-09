#!/usr/bin/env python3
"""Packs the port's source as target/the_sift-horizon-source.zip: everything but build outputs (target/, the
harness's cargo target and its native-data link), with fixed dates and modes, so the same tree gives the same
bytes. Unzipped, `python3 tools/build.py` rebuilds the .hmod and .hnmods byte for byte and
`cd harness && cargo test --release --workspace` runs the tests."""
import hashlib, zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SKIP_TOP = {'target', '.git'}

def wanted(rel):
    parts = rel.parts
    if parts[0] in SKIP_TOP or '__pycache__' in parts:
        return False
    if parts[0] == 'harness' and len(parts) > 1 and parts[1] in ('target', 'native-data'):
        return False
    if any(p == 'target' for p in parts[1:]) and parts[0] in ('harness', 'tools'):
        return False
    return True

out = ROOT / 'target' / 'the_sift-horizon-source.zip'
out.parent.mkdir(exist_ok=True)
files = sorted(p for p in ROOT.rglob('*') if p.is_file() and not p.is_symlink() and wanted(p.relative_to(ROOT)))
with zipfile.ZipFile(out, 'w', zipfile.ZIP_DEFLATED) as z:
    for p in files:
        i = zipfile.ZipInfo('the_sift-horizon/' + p.relative_to(ROOT).as_posix(), (2026, 1, 1, 0, 0, 0))
        i.compress_type = zipfile.ZIP_DEFLATED
        i.external_attr = (0o100755 if p.suffix == '.py' else 0o100644) << 16
        z.writestr(i, p.read_bytes())
print(out.name, len(files), 'files', hashlib.sha256(out.read_bytes()).hexdigest())
