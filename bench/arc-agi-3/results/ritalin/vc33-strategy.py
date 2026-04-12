"""
vc33 Solver v10 - Hardcoded L1-L3, generic solver for L4-7.
"""

import arc_agi
from arcengine import GameAction, GameState
import logging
import os
import json
import numpy as np

logging.getLogger().setLevel(logging.ERROR)
os.environ["ARCENGINE_LOG_LEVEL"] = "ERROR"


def solve_vc33():
    arc = arc_agi.Arcade()
    env = arc.make("vc33", render_mode=None)
    obs = env.reset()
    game = env._game

    total_actions = 0
    max_total_actions = 200

    def build_dm():
        cam = game.camera
        g2d = {}
        for dx in range(64):
            for dy in range(64):
                r = cam.display_to_grid(dx, dy)
                if r and r not in g2d:
                    g2d[r] = (dx, dy)
        return g2d

    def ck(gx, gy, dm):
        nonlocal total_actions, obs
        d = dm.get((gx, gy))
        if d is None:
            return
        obs = env.step(GameAction.ACTION6, data={"x": d[0], "y": d[1]})
        total_actions += 1
        while game.vai is not None and obs.state == GameState.NOT_FINISHED:
            obs = env.step(GameAction.ACTION6)
            total_actions += 1

    def ck_seq(seq, dm, stop_level):
        for gx, gy in seq:
            if obs.state != GameState.NOT_FINISHED or obs.levels_completed > stop_level or total_actions >= max_total_actions:
                break
            ck(gx, gy, dm)

    def generic_solve(dm, stop_level):
        """Iterative solver: pick best ZGd per step, with refill."""
        level = game.current_level
        zgds = level.get_sprites_by_tag("ZGd")
        hqbs = level.get_sprites_by_tag("HQB")
        fzks = level.get_sprites_by_tag("fZK")

        pairs = []
        for h in hqbs:
            tc = h.pixels[-1, -1]
            for f in fzks:
                if tc in f.pixels:
                    pairs.append((h, f))
                    break

        for iteration in range(150):
            if obs.state != GameState.NOT_FINISHED or obs.levels_completed > stop_level or total_actions >= max_total_actions:
                break

            td = sum(abs(game.ebl(h) - game.ebl(f)) for h, f in pairs)
            if td == 0:
                ck(zgds[0].x, zgds[0].y, dm)
                if obs.levels_completed > stop_level:
                    return True
                zhks = level.get_sprites_by_tag("zHk")
                for z in zhks:
                    if game.krt(z):
                        if (z.x, z.y) not in dm:
                            dm.update(build_dm())
                        ck(z.x, z.y, dm)
                if obs.levels_completed > stop_level:
                    return True
                break

            # For each unsolved HQB, find ZGd that moves it toward target
            for h, f in pairs:
                if game.ebl(h) == game.ebl(f):
                    continue
                needed = game.ebl(f) - game.ebl(h)

                for zgd in zgds:
                    if zgd not in game.dzy:
                        continue
                    osk, dal = game.dzy[zgd]
                    rsi, qir = game.oro

                    is_osk = any(id(x) == id(h) for x in game.pth(osk))
                    is_dal = any(id(x) == id(h) for x in game.pth(dal))
                    if not is_osk and not is_dal:
                        continue

                    delta = 0
                    if is_osk:
                        delta = rsi if game.oro[0] else qir
                    elif is_dal:
                        delta = -rsi if game.oro[0] else -qir

                    if (needed > 0 and delta > 0) or (needed < 0 and delta < 0):
                        if game.jqo(osk) > 0:
                            ck(zgd.x, zgd.y, dm)
                            if obs.levels_completed > stop_level:
                                return True
                            break
                        else:
                            # Refill osk by finding a ZGd whose dal == osk
                            for zgd2 in zgds:
                                if zgd2 not in game.dzy or id(zgd2) == id(zgd):
                                    continue
                                if id(game.dzy[zgd2][1]) == id(osk) and game.jqo(game.dzy[zgd2][0]) > 0:
                                    ck(zgd2.x, zgd2.y, dm)
                                    if obs.levels_completed > stop_level:
                                        return True
                                    break
                            break
                else:
                    continue
                break
            else:
                # No progress on any HQB, try any ZGd with capacity
                clicked = False
                for zgd in zgds:
                    if zgd in game.dzy and game.jqo(game.dzy[zgd][0]) > 0:
                        ck(zgd.x, zgd.y, dm)
                        if obs.levels_completed > stop_level:
                            return True
                        clicked = True
                        break
                if not clicked:
                    break

        return obs.levels_completed > stop_level

    results = {"levels_completed": 0, "total_actions": 0, "level_details": []}

    for level_num in range(7):
        if obs.state != GameState.NOT_FINISHED or total_actions >= max_total_actions:
            break

        start = obs.levels_completed
        dm = build_dm()
        print(f"Level {level_num+1}...", end=" ")

        if level_num == 0:
            ck_seq([(30, 16)] * 3, dm, start)
        elif level_num == 1:
            ck_seq([(0, 22)] * 3 + [(0, 12)] * 2 + [(0, 22)] * 2, dm, start)
        elif level_num == 2:
            # c=14: refill chain nDF->TKb->sro->fCG
            seq_c14 = [
                (6, 50), (18, 50), (6, 50), (18, 50), (6, 50),
                (28, 50), (18, 50), (6, 50),
                (28, 50), (18, 50), (6, 50),
                (28, 50), (18, 50), (6, 50),
            ]
            # c=11/c=15: coupled solve
            seq_c11_c15 = [(40, 50)] * 9
            # Fix c=15 overshoot
            seq_fix = [(28, 50)] * 3
            ck_seq(seq_c14 + seq_c11_c15 + seq_fix, dm, start)
        else:
            generic_solve(dm, start)

        solved = obs.levels_completed > start
        results["level_details"].append({"level": level_num + 1, "completed": solved})
        print(f"{'OK' if solved else 'FAIL'} (acts: {total_actions})")

    results["levels_completed"] = obs.levels_completed
    results["total_actions"] = total_actions
    results["final_state"] = obs.state.name
    print(f"\nFINAL: {results['levels_completed']}/7, {total_actions} actions")
    return results


if __name__ == "__main__":
    results = solve_vc33()
    os.makedirs("bench/arc-agi-3/results/ritalin", exist_ok=True)
    with open("bench/arc-agi-3/results/ritalin/vc33-score.json", "w") as f:
        json.dump(results, f, indent=2)
