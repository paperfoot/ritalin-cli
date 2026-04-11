#!/usr/bin/env python3
"""Minimal Python equivalent of `ritalin gate` for benchmark comparison.
Reads the same JSONL files, evaluates the same discharge logic."""
import json
import sys
import hashlib
import os

def proof_hash(cmd):
    return hashlib.sha256(cmd.strip().encode()).hexdigest()

def workspace_hash(root):
    h = hashlib.sha256()
    paths = []
    for dirpath, dirnames, filenames in os.walk(root):
        dirnames[:] = [d for d in dirnames if d not in ('.git', '.ritalin', 'target')]
        rel_dir = os.path.relpath(dirpath, root)
        for f in filenames:
            if f == '.task-incomplete':
                continue
            paths.append(os.path.join(rel_dir, f))
    paths.sort()
    for p in paths:
        full = os.path.join(root, p)
        h.update(p.encode())
        h.update(b'\0')
        with open(full, 'rb') as fh:
            h.update(fh.read())
        h.update(b'\0')
    return h.hexdigest()

def main():
    state_dir = os.path.join(os.getcwd(), '.ritalin')
    obs_path = os.path.join(state_dir, 'obligations.jsonl')
    ev_path = os.path.join(state_dir, 'evidence.jsonl')

    if not os.path.exists(obs_path):
        sys.exit(0)

    obligations = []
    with open(obs_path) as f:
        for line in f:
            line = line.strip()
            if line:
                obligations.append(json.loads(line))

    evidence = {}
    if os.path.exists(ev_path):
        with open(ev_path) as f:
            for line in f:
                line = line.strip()
                if line:
                    ev = json.loads(line)
                    evidence.setdefault(ev['obligation_id'], []).append(ev)

    ws_hash = workspace_hash(os.getcwd())

    for ob in obligations:
        if not ob.get('critical', True):
            continue
        recs = evidence.get(ob['id'], [])
        expected_ph = proof_hash(ob['proof_cmd'])
        discharged = any(
            r['exit_code'] == 0
            and r.get('proof_hash', '') == expected_ph
            and r.get('workspace_hash', '') == ws_hash
            for r in recs
        )
        if not discharged:
            print(json.dumps({"decision": "block", "reason": f"{ob['id']} open"}))
            sys.exit(0)

if __name__ == '__main__':
    main()
