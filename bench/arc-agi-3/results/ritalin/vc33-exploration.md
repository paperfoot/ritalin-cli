# vc33 Exploration Notes

## Game Type
Click-based puzzle game on a 64x64 grid with colored cells.

## Core Mechanics (from source code analysis)
- **Timer**: Row 0 shows remaining time (color 7=remaining, 4=expired)
- **Click**: ACTION6 with data={"x": dx, "y": dy} in display coordinates
- **Camera**: display_to_grid() maps display coords to game grid; scale varies per level
- **Sprites**:
  - ZGd = clickable buttons (color 9)
  - rDn = pipe segments (invisible collision areas)
  - HQB = colored markers that ride on pipes
  - fZK = target positions for markers
  - UXg = junction connectors between pipe segments
  - zHk = swap triggers (when bordered by rDn on both sides)

## Win Condition (gug function)
For each HQB:
1. Its color must match a fZK's color
2. Its ebl (position on secondary axis) must equal the fZK's ebl
3. It must be on an rDn whose suo() returns a UXg that also collides with that fZK

## Movement System
- TiD/oro = [rsi, qir] defines movement direction and step size
- lia() = oro[0]>0 or oro[1]>0 (affects coordinate function behavior)
- ebl = x if oro[0] else y (position axis)
- Each ZGd button controls a pipe pair (osk, dal): clicking shrinks osk, grows dal
- HQBs on osk move by (+rsi, +qir), on dal by (-rsi, -qir)
- Pipe depletion: when osk reaches width/height 0, clicking has no effect

## Level Data
| Level | Grid | TiD | Timer | HQBs | ZGds | Key Challenge |
|-------|------|-----|-------|-------|------|---------------|
| 1 | 32x32 | [2,0] | 50 | 1 | 2 | Simple slide |
| 2 | 32x32 | [-2,0] | 50 | 1 | 4 | Pipe chain with refill |
| 3 | 52x52 | [0,2] | 75 | 3 | 8 | Multi-HQB + coupled pipes |
| 4 | 64x64 | [0,3] | 50 | 2 | 6 | Large grid, step=3 |
| 5 | 64x64 | [3,0] | 200 | 3 | 6 | 3 HQBs + zHk swaps |
| 6 | 64x64 | [-3,0] | 50 | 2 | 4 | Negative direction |
| 7 | 48x48 | [0,-2] | 200 | 2 | 8 | Complex pipe network |

## Actions Explored
- ACTION6 without click data: decrements timer (wastes a turn)
- ACTION6 with {"x": dx, "y": dy}: clicks at display coordinate
  - Clicking ZGd: triggers pipe shift (ccl -> gel)
  - Clicking zHk: triggers pipe swap animation (teu)
  - Clicking empty: decrements timer only
