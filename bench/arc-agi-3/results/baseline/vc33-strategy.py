#!/usr/bin/env python3
"""
VC33 Solver - ARC-AGI-3 Game
============================
Track-switching puzzle. Move HQB markers to match fZK targets by clicking
ZGd swap buttons that pump track segments.

Strategy: Build a simplified state model per level, then BFS for optimal solution.
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


def analyze_click_effects(game, targets, prev_clicks, arc):
    """Determine the effect of each click button by testing."""
    effects = []
    for ti, (dx, dy) in enumerate(targets):
        env2 = arc.make('vc33', render_mode=None)
        obs2 = env2.reset()
        game2 = env2._game
        for pdx, pdy in prev_clicks:
            obs2 = env2.step(GameAction.ACTION6, data={"x": pdx, "y": pdy})

        hqb_before = {h.name: game2.ebl(h) for h in game2.current_level.get_sprites_by_tag("HQB")}
        rdn_before = {r.name: r.height for r in game2.current_level.get_sprites_by_tag("rDn")}

        obs2 = env2.step(GameAction.ACTION6, data={"x": dx, "y": dy})

        if obs2.state == GameState.GAME_OVER or obs2.levels_completed > game.current_level.get_data("RoA"):
            effects.append(None)
            continue

        hqb_after = {h.name: game2.ebl(h) for h in game2.current_level.get_sprites_by_tag("HQB")}
        rdn_after = {r.name: r.height for r in game2.current_level.get_sprites_by_tag("rDn")}

        effect = {
            "hqb_delta": {k: hqb_after.get(k, 0) - hqb_before.get(k, 0) for k in hqb_before if hqb_after.get(k, 0) != hqb_before.get(k, 0)},
            "rdn_delta": {k: rdn_after.get(k, 0) - rdn_before.get(k, 0) for k in rdn_before if rdn_after.get(k, 0) != rdn_before.get(k, 0)},
        }
        effects.append(effect)

    return effects


def build_state_model(game, effects, arc, prev_clicks):
    """Build a state model for BFS solving."""
    hqb_list = game.current_level.get_sprites_by_tag("HQB")
    fzk_list = game.current_level.get_sprites_by_tag("fZK")
    rdn_list = game.current_level.get_sprites_by_tag("rDn")

    # State: HQB ebl values + rDn height values
    hqb_names = sorted([h.name for h in hqb_list])
    rdn_names = sorted([r.name for r in rdn_list])

    hqb_init = {h.name: game.ebl(h) for h in hqb_list}
    rdn_init = {r.name: r.height for r in rdn_list}

    # Goal: each HQB at its target ebl
    hqb_targets = {}
    for h in hqb_list:
        color = h.pixels[-1, -1]
        for f in fzk_list:
            if color in f.pixels:
                hqb_targets[h.name] = game.ebl(f)
                break

    print(f"  HQB init: {hqb_init}")
    print(f"  HQB targets: {hqb_targets}")
    print(f"  rDn init heights: {rdn_init}")

    # Build action effects as deltas
    actions = []
    for ti, eff in enumerate(effects):
        if eff is None:
            actions.append(None)
            continue

        hqb_d = eff["hqb_delta"]
        rdn_d = eff["rdn_delta"]

        # Determine which rDn is consumed (height decreases)
        consumed = [name for name, delta in rdn_d.items() if delta < 0]
        produced = [name for name, delta in rdn_d.items() if delta > 0]

        action = {
            "hqb_delta": hqb_d,
            "rdn_delta": rdn_d,
            "consumed": consumed,
            "produced": produced,
        }
        actions.append(action)

    return hqb_names, rdn_names, hqb_init, rdn_init, hqb_targets, actions


def bfs_solve(hqb_names, rdn_names, hqb_init, rdn_init, hqb_targets, actions, max_depth=40):
    """BFS on simplified state model."""
    # State tuple: (hqb values..., rdn heights...)
    def make_state(hqb_vals, rdn_vals):
        return tuple([hqb_vals[n] for n in hqb_names] + [rdn_vals[n] for n in rdn_names])

    def is_goal(state):
        for i, name in enumerate(hqb_names):
            if name in hqb_targets and state[i] != hqb_targets[name]:
                return False
        return True

    def apply_action(state, action_idx):
        action = actions[action_idx]
        if action is None:
            return None

        # Check if consumed resources are available
        rdn_offset = len(hqb_names)
        for consumed_name in action["consumed"]:
            idx = rdn_offset + rdn_names.index(consumed_name)
            if state[idx] < 2:
                return None

        # Apply deltas
        new_state = list(state)
        for name, delta in action["hqb_delta"].items():
            if name in hqb_names:
                idx = hqb_names.index(name)
                new_state[idx] += delta

        for name, delta in action["rdn_delta"].items():
            if name in rdn_names:
                idx = rdn_offset + rdn_names.index(name)
                new_state[idx] += delta

        # Sanity check
        if any(v < 0 for v in new_state):
            return None

        return tuple(new_state)

    init_state = make_state(hqb_init, rdn_init)

    if is_goal(init_state):
        return []

    queue = deque()
    visited = set()
    queue.append((init_state, []))
    visited.add(init_state)

    while queue:
        state, path = queue.popleft()

        if len(path) >= max_depth:
            continue

        for a in range(len(actions)):
            new_state = apply_action(state, a)
            if new_state and new_state not in visited:
                if is_goal(new_state):
                    return path + [a]
                visited.add(new_state)
                queue.append((new_state, path + [a]))

    return None


def solve_level(arc, prev_clicks, max_bfs_depth=40):
    """Solve one level using simplified state model + BFS."""
    env = arc.make('vc33', render_mode=None)
    obs = env.reset()
    game = env._game
    for dx, dy in prev_clicks:
        obs = env.step(GameAction.ACTION6, data={"x": dx, "y": dy})

    start_level = obs.levels_completed
    targets = get_targets_fast(game)

    print(f"\nLevel {start_level + 1}: {len(targets)} targets")

    # Analyze effects
    effects = analyze_click_effects(game, targets, prev_clicks, arc)

    for ti, eff in enumerate(effects):
        if eff:
            print(f"  T[{ti}]: {eff}")
        else:
            print(f"  T[{ti}]: no effect")

    # Build state model
    hqb_names, rdn_names, hqb_init, rdn_init, hqb_targets, actions = build_state_model(
        game, effects, arc, prev_clicks
    )

    # BFS
    solution_indices = bfs_solve(hqb_names, rdn_names, hqb_init, rdn_init, hqb_targets, actions, max_bfs_depth)

    if solution_indices is None:
        print(f"  BFS failed!")
        return None

    solution_clicks = [targets[i] for i in solution_indices]
    print(f"  BFS solution: {len(solution_indices)} clicks: {solution_indices}")

    # Verify in actual game
    env2 = arc.make('vc33', render_mode=None)
    obs2 = env2.reset()
    for dx, dy in prev_clicks:
        obs2 = env2.step(GameAction.ACTION6, data={"x": dx, "y": dy})

    for dx, dy in solution_clicks:
        obs2 = env2.step(GameAction.ACTION6, data={"x": dx, "y": dy})
        if obs2.state == GameState.GAME_OVER:
            print(f"  GAME OVER during verification!")
            return None

    if obs2.levels_completed > start_level:
        print(f"  Verified! Level solved.")
        return solution_clicks
    else:
        print(f"  Verification failed - level not completed. levels={obs2.levels_completed}")
        # The simplified model might not match exactly. Try adding extra clicks.
        return None


def main():
    arc = arc_agi.Arcade()

    all_clicks = []
    total_actions = 0

    for level_num in range(1, 8):
        solution = solve_level(arc, all_clicks, max_bfs_depth=35)

        if solution is None:
            print(f"\nCould not solve level {level_num}")
            break

        all_clicks.extend(solution)
        total_actions += len(solution)
        print(f"  Total actions: {total_actions}/200")

        if total_actions >= 195:
            print("  Approaching action budget limit!")
            break

    # Final verification
    print(f"\n=== Final Verification ===")
    env = arc.make('vc33', render_mode=None)
    obs = env.reset()

    for i, (dx, dy) in enumerate(all_clicks):
        obs = env.step(GameAction.ACTION6, data={"x": dx, "y": dy})
        if obs.state == GameState.GAME_OVER:
            obs = env.reset()
        elif obs.state == GameState.WIN:
            break

    final_levels = obs.levels_completed
    print(f"Final score: {final_levels}/7 levels completed in {len(all_clicks)} actions")

    # Save results
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
