# VC33 Game Analysis Log

## Game Overview
VC33 is a track-switching puzzle game with 7 levels. The game mechanics involve:
- **HQB markers** that need to be moved to match **fZK target positions**
- **rDn track segments** that carry the markers
- **ZGd swap buttons** that pump track segments (shrink one, grow another)
- **zHk switch buttons** that teleport markers between tracks when activated
- A **timer bar** (row 0) that depletes with each action

## Core Mechanics
- `ebl(sprite)` returns the position coordinate (x or y depending on level orientation)
- `jqo(sprite)` returns the "fuel" dimension (track length available for pumping)
- `gel()` transfers fuel between two tracks, moving any HQB markers on them
- `krt()` checks if a zHk switch can be activated (needs tracks on both sides)
- `gug()` checks win condition: all HQB markers at their fZK target positions
- `oro` defines step size and direction per level (e.g., [2,0] or [0,3])

## Display Coordinate System
- Camera has position, width, height
- Display is 64x64
- Scale = min(64/cam_width, 64/cam_height)
- Padding = (64 - cam_dim * scale) / 2
- Grid coord = (display - padding) / scale + cam_offset

## Level Solutions

### Level 1 (3 actions)
- Grid: 32x32, Scale: 2, TiD: [2,0]
- 1 HQB (ChX): ebl 23 -> 17 (delta -6, 3 clicks of -2)
- Solution: 3x T[1] at display (60,32)

### Level 2 (7 actions)  
- Grid: 32x32, Scale: 2, TiD: [-2,0]
- 1 HQB (PPS): ebl 4 -> 14 (delta +10, 5 pump + 2 refill)
- Solution: T[1,1,3,3,3,3,3] at display coords
- Pattern: refill fuel track (T[1] grows dkk), then pump HQB (T[3] moves PPS)

### Level 3 (23 actions)
- Grid: 52x52, Scale: 1 (padding: 6), TiD: [0,2]
- 3 HQBs: ChX 21->39, PPS 45->33, VAJ 43->31
- Solution: [0,3,0,3,0,5,3,0,5,3,0,5,3,0,7,7,7,7,7,7,7,7,7]
- Strategy: First pump PPS down (T[0]) with sro refills (T[3]) and TKb refills (T[5]),
  then pump ChX up AND VAJ down together (T[7] does both)

### Level 4 (unsolved)
- Grid: 64x64, Scale: 1, TiD: [0,3]
- 1 HQB (Ubu): ebl 43 -> 25
- T[0] and T[2] are opposites (cancel each other for HQB movement)
- Requires zHk switch activation: need to position tracks adjacent to zHk
- zHk at (27,34) needs: Oqo.ebl=46 AND BfR.ebl=46 for krt=True
- But activating zHk after 5x T[0] puts Ubu at ebl=58, swap doesn't reach target 25
- The zHk teleport mechanic (teu) needs deeper analysis

## Results
- **Score: 3/7 levels completed in 33 actions**
- Remaining action budget: 167/200

## What Worked
- Simplified state model (ebl + jqo per track) for BFS
- Analyzing click effects empirically by testing each button
- Understanding pump-and-refill pattern

## What Didn't Work
- Greedy solver (can't see through refill steps)
- Pure ZGd-only approach for L4 (T[0]/T[2] cancel)
- Full simulation BFS (too slow - 1 env creation per node)

## What's Needed for L4+
- Deeper understanding of zHk teu() swap: how it repositions HQB
- Need to track both ebl (position) and urh (cross-position) in state model
- May need multi-phase strategies: pump tracks to activate zHk, then use zHk to teleport
