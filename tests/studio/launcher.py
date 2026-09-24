"""Qualified Linux launcher: symlink, spaces, exact argv and rebuilt binary."""
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[2]
with tempfile.TemporaryDirectory(prefix='yai studio launcher ') as tmp:
    root = Path(tmp) / 'source checkout'
    scripts = root / 'tools/shell'
    scripts.mkdir(parents=True)
    for name in ('yai-studio.sh', 'install-studio.sh'):
        shutil.copy2(ROOT / 'tools/shell' / name, scripts / name)
    binary = root / 'studio/src-tauri/target/release/yai-studio'
    binary.parent.mkdir(parents=True)
    prefix = Path(tmp) / 'local install'
    env = dict(os.environ, PREFIX=str(prefix), HOME=tmp, DISPLAY=':99')
    for name in ('YAI_HOME', 'GDK_BACKEND', 'WEBKIT_DISABLE_DMABUF_RENDERER'):
        env.pop(name, None)
    refused = subprocess.run(['sh', str(scripts / 'install-studio.sh')], env=env, capture_output=True)
    assert refused.returncode == 127
    def build(version):
        binary.write_text('#!/usr/bin/env python3\nimport os,json,sys\nprint(json.dumps([%r,sys.argv[1:],os.environ.get("YAI_HOME"),os.environ.get("GDK_BACKEND"),os.environ.get("WEBKIT_DISABLE_DMABUF_RENDERER")]))\n' % version)
        binary.chmod(0o755)
    build('first')
    subprocess.run(['sh', str(scripts / 'install-studio.sh')], env=env, check=True)
    launcher = prefix / 'bin/yai-studio'
    def run(overrides=None):
        return json.loads(subprocess.check_output([str(launcher), 'a b', '$(literal)'], env=dict(env, **(overrides or {})), cwd='/'))
    assert run() == ['first', ['a b', '$(literal)'], str(Path(tmp)/'.yai'), 'x11', '1']
    build('rebuilt')
    assert run()[0] == 'rebuilt'
    assert run(dict(YAI_HOME='/explicit profile', GDK_BACKEND='wayland', WEBKIT_DISABLE_DMABUF_RENDERER='0'))[2:] == ['/explicit profile','wayland','0']
    assert run(dict(DISPLAY=''))[3] is None
    binary.unlink()
    assert subprocess.run([str(launcher)], env=env, capture_output=True).returncode == 127
print('PASS: installed symlink, rebuilt binary, spaces/argv, defaults, overrides, missing build')
