#!/usr/bin/env python3
"""ARC-AGI-3 game driver.

Usage:
    python3 play.py --list                          # List available games
    python3 play.py --game ls20 --interactive       # Interactive mode (agent types actions)
    python3 play.py --game ls20 --random            # Random baseline
    python3 play.py --game ls20 --strategy strat.py # Run a strategy file

In interactive mode, the script prints observations and reads action numbers
from stdin. The Claude Code agent runs this in a Bash call and types actions.

Log files are written to --log-dir (default: results/).
"""

import argparse
import json
import logging
import os
import random
import sys
import time
from pathlib import Path

# Suppress arc-agi info logging
logging.getLogger().setLevel(logging.ERROR)
os.environ.setdefault("ARCENGINE_LOG_LEVEL", "ERROR")


def render_grid(frame_data) -> str:
    """Convert frame data to compact text grid for LLM reasoning."""
    if not frame_data or not frame_data.frame:
        return "[no frame data]"

    import numpy as np
    grid = frame_data.frame[0]
    if not isinstance(grid, np.ndarray):
        grid = np.array(grid)

    nonzero = np.argwhere(grid != 0)
    if len(nonzero) == 0:
        return "[empty grid]"

    r_min, c_min = nonzero.min(axis=0)
    r_max, c_max = nonzero.max(axis=0)
    r_min = max(0, r_min - 1)
    c_min = max(0, c_min - 1)
    r_max = min(grid.shape[0] - 1, r_max + 1)
    c_max = min(grid.shape[1] - 1, c_max + 1)
    cropped = grid[r_min:r_max + 1, c_min:c_max + 1]

    char_map = {0: '.', 1: '#', 2: '@', 3: '*', 4: '+', 5: 'X',
                6: 'O', 7: '~', 8: '^', 9: '!'}
    lines = []
    for row in cropped:
        lines.append(''.join(char_map.get(int(v), str(int(v))) for v in row))
    return '\n'.join(lines)


def play_interactive(game_id, max_actions=300, log_dir=None):
    """Interactive mode: print state, read action from stdin, repeat."""
    import arc_agi
    from arcengine import GameAction, GameState

    arc = arc_agi.Arcade()
    env = arc.make(game_id, render_mode=None)
    if env is None:
        print(f"ERROR: Failed to create environment for {game_id}", file=sys.stderr)
        sys.exit(1)

    obs = env.reset()
    step = 0
    log_entries = []

    print(f"=== ARC-AGI-3: {game_id} ===")
    print(f"Levels to win: {obs.win_levels}")
    print(f"Available actions: {[a.value for a in env.action_space]}")
    print(f"Max actions: {max_actions}")
    print()
    print("GRID:")
    print(render_grid(obs))
    print()
    print(f"STATE: {obs.state.name} | Levels: {obs.levels_completed}/{obs.win_levels}")
    print(f"Available actions: {[a.value for a in env.action_space]}")
    print("Enter action number (or 'q' to quit):")
    sys.stdout.flush()

    while step < max_actions:
        try:
            line = input().strip()
        except EOFError:
            break

        if line.lower() == 'q':
            break

        try:
            action_num = int(line)
        except ValueError:
            print(f"Invalid input: '{line}'. Enter a number from {[a.value for a in env.action_space]}")
            sys.stdout.flush()
            continue

        try:
            action = GameAction(action_num)
        except ValueError:
            print(f"Invalid action: {action_num}. Available: {[a.value for a in env.action_space]}")
            sys.stdout.flush()
            continue

        obs = env.step(action)
        step += 1

        entry = {
            "step": step,
            "action": action_num,
            "state": obs.state.name if obs else "UNKNOWN",
            "levels_completed": obs.levels_completed if obs else 0,
        }
        log_entries.append(entry)

        print(f"\n--- Step {step} | Action: {action_num} ---")
        print("GRID:")
        print(render_grid(obs))
        print(f"STATE: {obs.state.name} | Levels: {obs.levels_completed}/{obs.win_levels}")
        print(f"Available actions: {[a.value for a in env.action_space]}")

        if obs and obs.state == GameState.GAME_OVER:
            print("GAME OVER on this level. Resetting level...")
            obs = env.reset()
            print("GRID (after reset):")
            print(render_grid(obs))
            print(f"STATE: {obs.state.name} | Levels: {obs.levels_completed}/{obs.win_levels}")
            print(f"Available actions: {[a.value for a in env.action_space]}")

        elif obs and obs.state == GameState.WIN:
            print(f"\n*** GAME WON after {step} actions! ***")
            try:
                sc = arc.get_scorecard()
                if sc:
                    print(f"Score: {sc.score}")
            except Exception:
                pass
            break

        print("Enter action number (or 'q' to quit):")
        sys.stdout.flush()

    # Save logs
    if log_dir:
        log_path = Path(log_dir)
        log_path.mkdir(parents=True, exist_ok=True)

        short_id = game_id.split('-')[0]
        with open(log_path / f"{short_id}-actions.jsonl", 'w') as f:
            for entry in log_entries:
                f.write(json.dumps(entry) + '\n')

        summary = {
            "game_id": game_id,
            "total_steps": step,
            "final_state": obs.state.name if obs else "UNKNOWN",
            "levels_completed": obs.levels_completed if obs else 0,
            "win_levels": obs.win_levels if obs else 0,
        }
        with open(log_path / f"{short_id}-score.json", 'w') as f:
            json.dump(summary, f, indent=2)

        print(f"\nLogs saved to {log_path}/")

    print(f"\nFinal: {step} actions, {obs.levels_completed if obs else 0}/{obs.win_levels if obs else '?'} levels")


def play_random(game_id, max_actions=300, log_dir=None):
    """Random baseline for comparison."""
    import arc_agi
    from arcengine import GameAction, GameState

    arc = arc_agi.Arcade()
    env = arc.make(game_id, render_mode=None)
    obs = env.reset()
    step = 0

    while step < max_actions:
        actions = [a for a in env.action_space]
        if not actions:
            break
        action = random.choice(actions)
        obs = env.step(action)
        step += 1

        if obs and obs.state == GameState.GAME_OVER:
            obs = env.reset()
        elif obs and obs.state == GameState.WIN:
            print(f"Random agent WON {game_id} in {step} actions!")
            break

    print(f"Random: {game_id} | {step} actions | levels: {obs.levels_completed}/{obs.win_levels}")


def main():
    parser = argparse.ArgumentParser(description="ARC-AGI-3 game driver")
    parser.add_argument("--list", action="store_true")
    parser.add_argument("--game", type=str)
    parser.add_argument("--interactive", action="store_true")
    parser.add_argument("--random", action="store_true")
    parser.add_argument("--max-actions", type=int, default=300)
    parser.add_argument("--log-dir", type=str, default=None)
    args = parser.parse_args()

    if args.list:
        import arc_agi
        arc = arc_agi.Arcade()
        envs = arc.get_environments()
        for e in envs:
            print(f"{e.game_id}: {e.title} (tags: {e.tags})")
        return

    if not args.game:
        parser.print_help()
        return

    # Resolve short game ID
    import arc_agi
    arc = arc_agi.Arcade()
    envs = arc.get_environments()
    game_id = None
    for e in envs:
        if e.game_id.startswith(args.game) or args.game in e.game_id:
            game_id = e.game_id
            break

    if not game_id:
        print(f"Game '{args.game}' not found", file=sys.stderr)
        sys.exit(1)

    if args.random:
        play_random(game_id, args.max_actions, args.log_dir)
    elif args.interactive:
        play_interactive(game_id, args.max_actions, args.log_dir)
    else:
        play_interactive(game_id, args.max_actions, args.log_dir)


if __name__ == "__main__":
    main()
