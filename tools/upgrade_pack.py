"""Upgrade the generated 26.2 data pack to 26.3 with Horizon's own upgrader
(horizon-port-pack's upgrade module, built from tools/hzupgrade), then check
every carver has the fields Horizon's carver loader requires."""
import json, shutil, subprocess, sys, tempfile
from pathlib import Path
TOOLS = Path(__file__).parent
pack = Path(sys.argv[1])
crate = TOOLS / 'hzupgrade'
subprocess.run(['cargo', 'build', '--offline', '--release', '--quiet'], cwd=crate, check=True)
exe = crate / 'target' / 'release' / 'hzupgrade'
with tempfile.TemporaryDirectory() as t:
    src, out = Path(t) / 'in', Path(t) / 'out'
    shutil.copytree(pack, src)
    w = src / 'data' / 'the_sift' / 'worldgen'
    for new, old in (('carver', 'configured_carver'), ('feature', 'configured_feature')):
        if (w / new).exists():
            (w / new).rename(w / old)
    print(subprocess.run([exe, src, out], check=True, capture_output=True, text=True).stdout, end='')
    shutil.rmtree(pack)
    shutil.copytree(out, pack)
need = {'minecraft:cave': ['probability', 'y', 'count', 'thickness', 'floor_level', 'room_vertical_radius_multiplier',
                           'horizontal_radius_multiplier', 'vertical_radius_multiplier'],
        'minecraft:canyon': ['probability', 'y', 'vertical_rotation', 'shape']}
for f in sorted((pack / 'data' / 'the_sift' / 'worldgen' / 'carver').glob('*.json')):
    j = json.loads(f.read_text())
    miss = [k for k in need[j['type']] if k not in j]
    assert not miss and 'config' not in j, f'{f.name}: missing {miss}'
    assert 0 <= j['probability'] <= 1
print('carvers ok')
