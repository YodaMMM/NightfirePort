# level-14-door-guard-release — report

Branch `s/level-14-door-guard-release`, from origin/master 7d0091a0. Status: **investigation
complete, fix not landed** — the task's premise (the three 0x38 guards need a release-channel
exit) is refuted by the decompile; the real gap is identified below with evidence, and the next
session can land the fix from this branch.

## What was done

### 1. The 0x38 stay is GAME-FAITHFUL — the premise is refuted

The three stuck guards are the disc's own ambush-mode templates (#721/#722/#724 [D
docs/levels/07000014-audit.md :73-76: prop word 0xf1dfc8c1 → AMBUSH prop 0x10 = 1, alertness
prop 0x20 = 2 → 0.66]):

- `NDrone2_DSTATE_AmbushWait` 0x161560 has **no channel read at all**. Its only exits [M
  0x161560:28-70]: msg 3 (UPDATE) leaves on the opponent-sight bit (`drone+0x228 & 4`, written
  only by `DroneVision_HaveOpponentSight` 0x175ef0:46) or the player within 3.0 2-D; msgs
  6/7/8/9/0x17/0x18/0x19 route through `DroneFunc_HandleImpact`; msg 0x21 through
  `DroneFunc_ConsiderExplosive`; everything else falls to the Global state (0x15b830), which
  handles only msgs 0/1/10/0x1d, so the message is dropped.
- The release channel 0x8C (mux #717: 0x87 AND 0x88 → 0x8C [D audit :1411]) is the trio's WAKE
  channel (placement p3 = 0x8C [D audit :73-76]). It spawns them; it does not move them.
  `GetDroneTypeAttackTypeFriend` 0x14cbd8 sets the main mode `+0xc5 = GetDTYPENEW(drone, +0xc7,
  +0xc9)` [M 0x14cbd8]; with p15=8, p16=0, p21=0 [D audit] and prop 0x10 set, `GetDTYPENEW`
  0x14c938:10-14 returns mode **0x10** → `DroneTypeSettings[0x10].start = 0x37` →
  `AmbushInit` 0x172960 → no cover within 2.0 → crouch → **0x38** — in the game AND the port.
  The trio is SUPPOSED to crouch and wait; only sight of Bond (or an impact) ever moves them.
- The port's mirror is faithful: `AMBUSH_WAIT` handling at crates/nf-game/src/drone.rs:6126-6144
  implements exactly the 0x161560 exits (sight / <3.0 / explosive; impacts via the shared
  dispatcher), and the release/spawn path (fusebox.rs:185-194 = `FuseBox_Update` 0x1adfe8:110-123;
  triggers mux; GLOBAL ENTER drone.rs:6994-7008 = `NDrone2_DSTATE_Global` 0x15b830:22-43;
  `start_state` 0x10 → 0x37 drone.rs:2211) matches the decompile arm for arm.

The game's door pair (CUR 0x5f / PREV 0x5a [L 2026-10-30, test note]) therefore cannot be the
ambush trio: 0x38 has no path to 0x5a/0x5f except through 0x56 (sight), and the pair's states
match the released **full-alert** guards (prop 0x10 = 0, alertness 1.0 → mode 0 → start 4 → the
`+0x50c >= 1.0` fold [M 0x14cbd8] → 0x56 → combat route → the A* locked-door gate 0x1571b0:262-271
→ 0x5a → 0x5f).

### 2. The real port gap: three of the seven group-0xd templates never exist in the port

Reproduced with a scratch probe (temp test `zz_probe014`, not committed; run 4× against
`07000014.bin`, `NF_SILENT=1`, `--test-threads=1`): after setting ch 0x87 + 0x88 with Bond held
at the hall (-76.8, -0.07, 46.5) for 3000 ticks, the port spawns **six** guards, not nine:

```
g18 (-51.33,32.42) st 0x38   = #721  (ambush, correct)
g19 (-47.87,33.48) st 0x38   = #722  (ambush, correct)
g20 (-40.24,25.15) st 0x38   = #724  (ambush, correct)
g23 ≈ #778  (-30.6,13.4)  0x5f→0x73/0x74 cycling at the spawn area   (group-9 pair)
g24 ≈ #780  (-23.5,36.0)  same
g25 = #783  (-37.7,25.86) → walks west, holds 0x5f→0x73/0x74 at (-50.05,41.66) — the ONE door guard
```

The disc's group 0xd has **seven** oDrone templates [D audit :73-76, 85-87: #721 #722 #723
#724 #783 #784 #785, all p4=0xd, all p3 (wake) = 0x8C], and spawner #1333 is max 7 / waves 1
[D audit :1531]. The port's `drones.list` (37 entries) contains **no entry at #723
(-32.78,28.48), #784 (-36.18,27.76) or #785 (-31.74,25.16)** — checked at t=8 and t=40 after the
release with every list entry printed: nothing at those positions, enabled or not. They are
absent at load, so `DroneSpawner_SpawnDrone` 0x137cb0 can never pick them and the release brings
6 guards instead of 9 — with only ONE full-alert guard west of the spawner (#783) instead of the
game's several, hence one door guard instead of two.

The spawner mechanics themselves check out against the decompile: the +0x1F screens flag [M
0x137aa0:33-45], the trigger write `+0x0E = 0, +0x1E = 1` (no +0x1F change) [M 0x137e68:53-64],
the post-selection `Drone_PositionVisible` abort [M 0x137cb0:43 → 0x13a7a0] — all mirrored at
crates/nf-game/src/spawners.rs:91-203 and drone.rs:17710-17733.

## What is left (next session, from this branch)

1. **Find why the level parse drops #723/#784/#785.** Candidates: `Drones::attach_level`
   (drone.rs ~11940 "Every drone placement of the level's main map… spawner templates included")
   filtering them out (sub-map split? group collection bug?), or the placement row parse. Compare
   the port's 37-entry list against the audit's oDrone rows and find which row set the main map
   should carry [D audit]. Fix with the parse/attach address cited.
2. **Pin the fix** in `crates/nf-view/tests/door_guards_locked_door_07000014_live.rs`: assert the
   released guard count at the door and their states (the game: two guards stop 2.4 apart at
   (-51.6,41.2)/(-51.2,38.8), CUR 0x5f [L 2026-10-30]; the trio stays 0x38 — pin THAT too, it is
   [M]-proven), then un-ignore only blocker (2) (NB_SCAN_LIVE stays untouched — blocker (1) is
   20-drone-nobunching-side-bits').
3. **Run**: `door_guards_locked_door_07000014_live` (all three tests), every other 07000014
   binary, `nf-game`, then the full suite (chunked — see below).
4. Re-check the trio against the game after the fix: with more full-alert guards routing, the
   NoBunching/inert-scan interactions (blocker (1)) may change the door-area behaviour.

## Live checks for the main agent

1. **Which guards stand at the door.** On a fresh 07000014 boot, set 0x87+0x88 (use both
   fuseboxes #383/#385), hold Bond in the generator hall, and dump every alive guard's state
   word (`drones.list[i].state`, the game's Drone state +0x8 per docs/fields) plus position at
   30-frame spacing for 3000 frames. Expected per the decompile: the #721/#722/#724 trio holds
   0x38 the whole window (ambush [M 0x161560]); the door pair (two guards, 2.4 apart, CUR
   0x5f/PREV 0x5a) are full-alert releases. If instead a trio member is ever at the door, my
   mode-resolution reading (0x14c938:10-14 with +0xc9 = p21 = 0) is wrong and must be redone.
2. **The released population.** In the same run, count alive released guards (expect 9: seven
   group-0xd [D audit] + two group-0x9 alive of #778-#781). If the game also fails to spawn
   #723/#784/#785, the port's missing-template finding is wrong and the divergence is elsewhere
   (route/goal), not the attach.

## How this was verified

- Decompile reads: 0x161560, 0x172960, 0x15b830, 0x15b960, 0x14cbd8, 0x14c938, 0x14b300 (grep),
  0x137cb0, 0x137e68, 0x137aa0, 0x13a7a0, 0x1adfe8, 0x14c600 — every claim above carries its
  address and was read from `work/decomp/ACTION.ELF/` this session.
- Disc data: docs/levels/07000014-audit.md rows :73-76/:81-87 (drones), :1411 (mux), :1527/:1531
  (spawners).
- Port runs: the scratch probe (4 runs; final shape: release → 3000 ticks → per-guard dump;
  outputs quoted above) plus the existing `the_unlocked_door_is_no_route_block` pass.
- No cargo suite was completed this session: the sandbox kills background jobs, the fast runner
  had to run in foreground chunks, and the build (nf-game release) consumed the window. Resume
  the suite on the final tree with:
  `NF_ISO=<abs> CARGO_TARGET_DIR=$HOME/NightfirePort/target python3 tools/test/chunk_suite.py /home/z/suite14_state.json --budget 400`
  repeatedly until it prints ALL DONE (state file is scratch; the command recreates it).
  Nothing that changed behaviour was pushed — the branch's only change vs master is docs
  (WORKLOG + this report) — so master's suite standing carries over.

## Usage

not visible to me (no token, cost or context-window figures are exposed to this session; the
only proxies visible are the shell's wall-clock timestamps: setup/branch 07:40-07:49 UTC, the
decompile trace 07:52-08:20, the probe reproduce cycle 08:25-09:05, the checkpoint 09:07,
the report 09:10-09:15 UTC 2026-10-31).
