#!/usr/bin/env python3
"""Builds the port from Mielon's The Sift jar:
  content/  -> target/the_sift-<version>.hmod        (blocks, items, assets, data: a content mod)
  datapack/ -> mixins/the_sift_native/data/the_sift_end.zip (The Sift in the End's place)
  mixins/the_sift_native/ -> target/the_sift_native-<version>.hnmod (native, one bundle: the End as The Sift,
                            its terrain, ichor, blocks, items and mobs)
Every zip has fixed dates and modes, so the same inputs give the same bytes.
    python3 tools/build.py [unzipped 2.0.1 jar]
    (without a jar: packages content/ and datapack/ as they are)
"""
import shutil, hashlib, json, subprocess, sys, zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
TOOLS = ROOT / 'tools'

def zip_dir(src, out, skip=lambda rel: False, licence=None):
    out.parent.mkdir(parents=True, exist_ok=True)
    files = {}
    for p in sorted(src.rglob('*')):
        rel = p.relative_to(src).as_posix()
        if p.is_file() and not skip(rel):
            files[rel] = p.read_bytes()
    if licence:
        files.update(licence)
    with zipfile.ZipFile(out, 'w', zipfile.ZIP_DEFLATED) as z:
        for name, data in sorted(files.items()):
            i = zipfile.ZipInfo(name, (2026, 1, 1, 0, 0, 0))
            i.compress_type = zipfile.ZIP_DEFLATED
            i.external_attr = 0o100644 << 16
            z.writestr(i, data)
    return out

def report(path):
    return dict(file=path.name, size=path.stat().st_size, sha256=hashlib.sha256(path.read_bytes()).hexdigest())

# The harness reads the bundles' data where the launcher stages it (native-data/<id>/).
link = ROOT / 'harness' / 'native-data' / 'the_sift_native'
if not link.exists():
    link.parent.mkdir(parents=True, exist_ok=True)
    link.symlink_to(Path('..') / '..' / 'mixins' / 'the_sift_native' / 'data')

if len(sys.argv) > 1:
    jar = sys.argv[1]
    subprocess.run([sys.executable, TOOLS / 'gen_content.py', jar, ROOT / 'content'], check=True)
    # The End's data pack (and the native mod's placed-feature table), the templates the native mod builds
    # in, and the harness's copy of the mod's block states.
    shutil.rmtree(ROOT / 'datapack', ignore_errors=True)
    subprocess.run([sys.executable, TOOLS / 'gen_datapack.py', jar, ROOT / 'datapack'], check=True)
    subprocess.run([sys.executable, TOOLS / 'upgrade_pack.py', ROOT / 'datapack'], check=True)
    subprocess.run([sys.executable, TOOLS / 'gen_templates.py', jar, ROOT / 'mixins' / 'the_sift_native' / 'data' / 'templates'], check=True)
    subprocess.run([sys.executable, TOOLS / 'gen_mock_blocks.py'], check=True)

# The content mod carries the licence and credits as assets (a content mod holds only assets/ and data/).
credits = {
    'assets/the_sift/license.txt': (ROOT / 'LICENSE').read_bytes(),
    'assets/the_sift/notice.md': (ROOT / 'NOTICE.md').read_bytes(),
}
import tomllib
version = tomllib.loads((ROOT / 'content' / 'horizon-mod.toml').read_text())['version']
hmod = zip_dir(ROOT / 'content', ROOT / 'target' / f'the_sift-{version}.hmod', licence=credits)
pack = zip_dir(ROOT / 'datapack', ROOT / 'mixins' / 'the_sift_native' / 'data' / 'the_sift_end.zip',
               skip=lambda rel: rel == 'REMOVED.md')
out = subprocess.run([sys.executable, TOOLS / 'package.py'], check=True, capture_output=True, text=True).stdout
hnmods = [ROOT / b['file'] for b in json.loads(out)]
print(json.dumps([report(hmod), report(pack)] + [report(h) for h in hnmods], indent=2))
