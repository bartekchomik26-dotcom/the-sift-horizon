#!/usr/bin/env python3
"""Packages each bundle as a source-only `.hnmod` (one bundle a file): `horizon-native.json` plus every `.rs` and `.toml`
under `mixins/<id>/` (the bundle crate's folders included) and anything under `mixins/<id>/data/`,
with fixed dates and modes so the same sources give the same bytes. Adapted from the native mod
template's packager (MIT)."""
import hashlib, json, tomllib, zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
GAME = '26.3'

def package(manifest_path):
    compat = json.loads((ROOT / 'compatibility.json').read_text())
    bundle = manifest_path.parent
    mixin = tomllib.loads(manifest_path.read_text())
    mod = mixin['mod']
    if bundle.name != mod['id']:
        raise SystemExit(f"rename mixins/{bundle.name}/ to mixins/{mod['id']}/")
    if mixin.get('compatibility', {}).get('horizon_revision') != compat['horizon_revision']:
        raise SystemExit('horizon-mixin.toml and compatibility.json pin different Horizon revisions')
    manifest = dict(schema=1, id=mod['id'], version=mod['version'], game=GAME,
                    horizon=compat['horizon_revision'], mixins=compat['horizon_mixins_revision'])
    files = {'horizon-native.json': (json.dumps(manifest, indent=2, sort_keys=True) + '\n').encode()}
    for path in sorted(bundle.rglob('*')):
        rel = path.relative_to(bundle)
        if not path.is_file() or 'target' in rel.parts or path.name == 'Cargo.lock':
            continue
        if path.suffix in ('.rs', '.toml') or rel.parts[0] == 'data' or rel.name.startswith('LICENSE') or rel.name == 'NOTICE.md':
            files[path.relative_to(ROOT).as_posix()] = path.read_bytes()
    if len(files) > 512:
        raise SystemExit('a .hnmod holds at most 512 entries')
    out = ROOT / 'target' / f"{mod['id']}-{mod['version']}.hnmod"
    out.parent.mkdir(parents=True, exist_ok=True)
    with zipfile.ZipFile(out, 'w', compression=zipfile.ZIP_DEFLATED) as archive:
        for name, content in sorted(files.items()):
            info = zipfile.ZipInfo(name, (2026, 1, 1, 0, 0, 0))
            info.compress_type = zipfile.ZIP_DEFLATED
            info.external_attr = 0o100644 << 16
            archive.writestr(info, content)
    return dict(file=str(out), entries=len(files), size=out.stat().st_size,
                sha256=hashlib.sha256(out.read_bytes()).hexdigest())

if __name__ == '__main__':
    print(json.dumps([package(m) for m in sorted((ROOT / 'mixins').glob('*/horizon-mixin.toml'))], indent=2))
