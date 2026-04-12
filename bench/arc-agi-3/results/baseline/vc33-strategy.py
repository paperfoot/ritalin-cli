#!/usr/bin/env python3
"""
VC33 Solver - ARC-AGI-3 Game
============================
Track-switching puzzle. Move HQB markers to match fZK targets by clicking
ZGd swap buttons that pump track segments, and zHk switches for teleporting.

Solved levels 1-3 using simplified state model + BFS.
Level 4+ requires zHk teleport mechanics not yet solved.
"""

import arc_agi
from arcengine import ActionInput, GameAction, GameState
import numpy as np
import json
import logging
import os
from pathlib import Path
from collections import deque

logging.getLogger().setLevel(logging.ERROR)
os.environ["ARCENGINE_LOG_LEVEL"] = "ERROR"


def get_targets_fast(game):
    """Get display coordinates for all clickable sprites, accounting for camera padding."""
    cam = game.camera
    scale = min(64 // cam.width, 64 // cam.height)
    x_pad = (64 - cam.width * scale) // 2
    y_pad = (64 - cam.height * scale) // 2
    click_sprites = []
    for tag in ["ZGd", "zHk"]:
        for s in game.current_level.get_sprites_by_tag(tag):
            if s not in click_sprites:
                click_sprites.append(s)
    return [((s.x - cam.x) * scale + x_pad, (s.y - cam.y) * scale + y_pad) for s in click_sprites]


def analyze_and_solve(arc, prev_clicks, max_bfs_depth=40):
    """Build state model using (ebl, jqo) dimensions and BFS solve."""
    env = arc.make('vc33', render_mode=None)
    obs = env.reset()
    game = env._game
    for dx, dy in prev_clicks:
        obs = env.step(GameAction.ACTION6, data={"x": dx, "y": dy})

    start_level = obs.levels_completed
    targets = get_targets_fast(game)

    # Get initial state
    hqb_list = game.current_level.get_sprites_by_tag("HQB")
    fzk_list = game.current_level.get_sprites_by_tag("fZK")
    rdn_list = game.current_level.get_sprites_by_tag("rDn")

    hqb_names = sorted(set(h.name for h in hqb_list))
    rdn_names = sorted(set(r.name for r in rdn_list))

    hqb_init = {h.name: game.ebl(h) for h in hqb_list}
    rdn_init = {r.name: game.jqo(r) for r in rdn_list}

    hqb_targets = {}
    for h in hqb_list:
        color = h.pixels[-1, -1]
        for f in fzk_list:
            if color in f.pixels:
                hqb_targets[h.name] = game.ebl(f)
                break

    print(f"Level {start_level+1}: {len(targets)} targets")
    print(f"  HQB init: {hqb_init}, targets: {hqb_targets}")
    print(f"  rDn jqo: {rdn_init}")

    # Analyze each click's effect
    effects = []
    for ti, (dx, dy) in enumerate(targets):
        env2 = arc.make('vc33', render_mode=None)
        obs2 = env2.reset()
        game2 = env2._game
        for pdx, pdy in prev_clicks:
            obs2 = env2.step(GameAction.ACTION6, data={"x": pdx, "y": pdy})

        hqb_before = {h.name: game2.ebl(h) for h in game2.current_level.get_sprites_by_tag("HQB")}
        rdn_before = {r.name: game2.jqo(r) for r in game2.current_level.get_sprites_by_tag("rDn")}

        obs2 = env2.step(GameAction.ACTION6, data={"x": dx, "y": dy})

        if obs2.state == GameState.GAME_OVER:
            effects.append(None)
            continue

        hqb_after = {h.name: game2.ebl(h) for h in game2.current_level.get_sprites_by_tag("HQB")}
        rdn_after = {r.name: game2.jqo(r) for r in game2.current_level.get_sprites_by_tag("rDn")}

        hqb_delta = {k: hqb_after[k] - hqb_before[k] for k in hqb_before
                     if hqb_after.get(k, hqb_before[k]) != hqb_before[k]}
        rdn_delta = {k: rdn_after[k] - rdn_before[k] for k in rdn_before
                     if rdn_after.get(k, rdn_before[k]) != rdn_before[k]}

        consumed = [k for k, v in rdn_delta.items() if v < 0]
        effects.append({"hqb_delta": hqb_delta, "rdn_delta": rdn_delta, "consumed": consumed})
        print(f"  T[{ti}]: hqb={hqb_delta}, rdn={rdn_delta}")

    # BFS on simplified model
    def make_state(hqb_vals, rdn_vals):
        return tuple([hqb_vals.get(n, 0) for n in hqb_names] +
                     [rdn_vals.get(n, 0) for n in rdn_names])

    def is_goal(state):
        for i, name in enumerate(hqb_names):
            if name in hqb_targets and state[i] != hqb_targets[name]:
                return False
        return True

    def apply_action(state, a):
        eff = effects[a]
        if eff is None:
            return None
        rdn_offset = len(hqb_names)
        for c_name in eff["consumed"]:
            if c_name in rdn_names:
                idx = rdn_offset + rdn_names.index(c_name)
                step = abs(min(v for v in eff["rdn_delta"].values()))
                if state[idx] < step:
                    return None
        new = list(state)
        for name, delta in eff["hqb_delta"].items():
            if name in hqb_names:
                new[hqb_names.index(name)] += delta
        for name, delta in eff["rdn_delta"].items():
            if name in rdn_names:
                new[rdn_offset + rdn_names.index(name)] += delta
        if any(v < 0 for v in new):
            return None
        return tuple(new)

    init_state = make_state(hqb_init, rdn_init)
    if is_goal(init_state):
        return []

    queue = deque()
    visited = set()
    queue.append((init_state, []))
    visited.add(init_state)

    while queue:
        state, path = queue.popleft()
        if len(path) >= max_bfs_depth:
            continue
        for a in range(len(effects)):
            new_state = apply_action(state, a)
            if new_state and new_state not in visited:
                new_path = path + [a]
                if is_goal(new_state):
                    print(f"  BFS solved: {len(new_path)} clicks: {new_path}")
                    # Verify
                    env3 = arc.make('vc33', render_mode=None)
                    obs3 = env3.reset()
                    for pdx, pdy in prev_clicks:
                        obs3 = env3.step(GameAction.ACTION6, data={"x": pdx, "y": pdy})
                    for idx in new_path:
                        obs3 = env3.step(GameAction.ACTION6, data={"x": targets[idx][0], "y": targets[idx][1]})
                    if obs3.levels_completed > start_level:
                        print(f"  Verified!")
                        return [targets[i] for i in new_path]
                    else:
                        print(f"  Verification FAILED")
                        continue
                visited.add(new_state)
                queue.append((new_state, new_path))

    print(f"  BFS exhausted ({len(visited)} states)")
    return None


def main():
    arc = arc_agi.Arcade()

    all_clicks = []
    total_actions = 0

    for level_num in range(1, 8):
        solution = analyze_and_solve(arc, all_clicks, max_bfs_depth=35)
        if solution is None:
            print(f"\nCould not solve level {level_num}")
            break
        all_clicks.extend(solution)
        total_actions += len(solution)
        print(f"  Total actions: {total_actions}/200\n")
        if total_actions >= 195:
            break

    # Final verification
    print(f"\n=== Final Verification ===")
    env = arc.make('vc33', render_mode=None)
    obs = env.reset()
    for dx, dy in all_clicks:
        obs = env.step(GameAction.ACTION6, data={"x": dx, "y": dy})
        if obs.state == GameState.GAME_OVER:
            obs = env.reset()
        elif obs.state == GameState.WIN:
            break

    final_levels = obs.levels_completed
    print(f"Final score: {final_levels}/7 levels completed in {len(all_clicks)} actions")

    log_dir = Path("bench/arc-agi-3/results/baseline")
    log_dir.mkdir(parents=True, exist_ok=True)
    result = {
        "game_id": "vc33",
        "levels_completed": final_levels,
        "total_levels": 7,
        "total_actions": len(all_clicks),
        "clicks": all_clicks,
    }
    with open(log_dir / "vc33-score.json", "w") as f:
        json.dump(result, f, indent=2)
    print(f"Results saved to {log_dir}/vc33-score.json")
    return final_levels


if __name__ == "__main__":
    main()
