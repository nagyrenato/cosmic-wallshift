#!/usr/bin/env python3
import re
import json
import subprocess
import urllib.request
from datetime import datetime
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
LOCK = ROOT / "Cargo.lock"
TOML = ROOT / "Cargo.toml"

def parse_lock(lock_path):
    data = lock_path.read_text(encoding="utf-8")
    packages = {}
    entries = data.split('\n[[package]]')
    for entry in entries:
        name_m = re.search(r'name\s*=\s*"([^"]+)"', entry)
        ver_m = re.search(r'version\s*=\s*"([^"]+)"', entry)
        src_m = re.search(r'source\s*=\s*"([^"]+)"', entry)
        if name_m and ver_m and src_m:
            src = src_m.group(1)
            if src.startswith('registry+'):
                packages[name_m.group(1)] = ver_m.group(1)
    return packages

def get_latest_crate_version(crate):
    url = f'https://crates.io/api/v1/crates/{crate}'
    try:
        with urllib.request.urlopen(url, timeout=15) as r:
            j = json.load(r)
            return j.get('crate', {}).get('max_version')
    except Exception:
        return None

def parse_toml_deps(toml_path):
    text = toml_path.read_text(encoding='utf-8')
    deps = {}
    in_deps = False
    for line in text.splitlines():
        if line.strip().startswith('['):
            in_deps = line.strip() == '[dependencies]'
            continue
        if not in_deps:
            continue
        m_simple = re.match(r'\s*([A-Za-z0-9_-]+)\s*=\s*"([^"]+)"', line)
        if m_simple:
            deps[m_simple.group(1)] = m_simple.group(2)
            continue
        m_table = re.match(r'\s*([A-Za-z0-9_-]+)\s*=\s*\{([^}]*)\}', line)
        if m_table:
            name = m_table.group(1)
            body = m_table.group(2)
            if 'git' in body:
                continue
            ver_m = re.search(r'version\s*=\s*"([^"]+)"', body)
            if ver_m:
                deps[name] = ver_m.group(1)
    return deps

def update_toml_versions(toml_path, updates):
    text = toml_path.read_text(encoding='utf-8')
    for name, newver in updates.items():
        # replace simple form using a callable to avoid backreference parsing
        pattern_simple = re.compile(r'(\n\s*' + re.escape(name) + r'\s*=\s*")([^"]+)(")')
        text = pattern_simple.sub(lambda m, v=newver: m.group(1) + v + m.group(3), text)
        # replace table form version = using callable
        pattern_table = re.compile(r'(' + re.escape(name) + r'\s*=\s*\{[^}]*version\s*=\s*")([^"]+)(")')
        text = pattern_table.sub(lambda m, v=newver: m.group(1) + v + m.group(3), text)
    toml_path.write_text(text, encoding='utf-8')

def run(cmd, cwd=ROOT):
    print(f"$ {' '.join(cmd)}")
    p = subprocess.run(cmd, cwd=str(cwd), capture_output=True, text=True)
    print(p.stdout)
    if p.returncode != 0:
        print(p.stderr)
    return p.returncode == 0

def main():
    lock_pkgs = parse_lock(LOCK)
    toml_deps = parse_toml_deps(TOML)

    candidates = {}
    for name, curver in toml_deps.items():
        latest = get_latest_crate_version(name)
        if not latest:
            continue
        if latest != curver:
            candidates[name] = (curver, latest)

    if not candidates:
        print('No dependency updates found for top-level Cargo.toml.')
        return 0

    print('Found updates:')
    for k, (old, new) in candidates.items():
        print(f'- {k}: {old} -> {new}')

    # Prepare branch
    date = datetime.utcnow().strftime('%Y%m%d')
    branch = f'chore/deps-bump-{date}'
    # create or reset branch
    if not run(['git', 'checkout', '-B', branch]):
        print('Failed to create or switch to branch. Aborting.')
        return 2

    # Apply updates to top-level Cargo.toml
    updates = {k: v[1] for k, v in candidates.items()}
    update_toml_versions(TOML, updates)

    # Run cargo check (minimal verification)
    run(['cargo', 'check'])

    # Commit and push
    run(['git', 'add', str(TOML)])
    run(['git', 'commit', '-m', f'chore: bump dependencies ({date})'])
    if not run(['git', 'push', '-u', 'origin', branch]):
        print('Failed to push branch; push may require authentication.')
        # still leave changes on branch locally

    # Create a summary file
    summary = ROOT / 'DEPENDENCY_UPDATES.md'
    with summary.open('w', encoding='utf-8') as f:
        f.write('# Dependency updates\n\n')
        for k, (old, new) in candidates.items():
            f.write(f'- **{k}**: {old} → {new}\n')
    run(['git', 'add', str(summary)])
    run(['git', 'commit', '--allow-empty', '-m', f'Add dependency updates summary ({date})'])
    run(['git', 'push', 'origin', branch])

    # Print remote info for manual PR creation
    run(['git', 'remote', 'get-url', 'origin'])
    print('\nBranch created and pushed (if push succeeded).')
    print('Open a pull request from the pushed branch to main using your remote URL.')
    return 0

if __name__ == '__main__':
    raise SystemExit(main())
