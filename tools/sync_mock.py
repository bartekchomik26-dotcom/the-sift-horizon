#!/usr/bin/env python3
"""Fills the harness's renderer mock with the files as they ship: Horizon's environment.rs (its
code, without its tests), The Sift's end_pack.rs, and sky.rs's settings and uniform halves."""
from pathlib import Path
import sys
root = Path(__file__).resolve().parents[1]
hz = Path(sys.argv[1] if len(sys.argv) > 1 else '/home/claude/hz-src')
env = (hz / 'crates/horizon-client-render/src/environment.rs').read_text()
env = env[:env.index('#[cfg(test)]')]
(root / 'harness/render-mock/src/environment_real.rs').write_text(env)
sky = (root / 'mixins/the_sift_native/sky.rs').read_text()
top = sky[:sky.index('const SHADER')]
i = sky.index("/// The uniform's bytes"); j = sky.index('/// Once the world is drawn')
(root / 'harness/render-mock/src/sky_settings.rs').write_text(top + sky[i:j])
