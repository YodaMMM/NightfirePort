# level-13-warm-chain-margin — the bleed bisected; the margin work is mid-flight

**Branch** `s/level-13-warm-chain-margin` (also pushed as `s/10-level-13-warm-chain-margin`, same
commits — the brief names both). Cut from origin/master ef7822e8b.

## The bisect (the task's first question — DONE)

`git bisect start origin/master d09f8b2b3`, `git bisect run` with the walk itself as the probe
(`scripts/bisect13.sh`: build + run
`walkreal_07000013_live walk_the_warm_chain_to_the_front_entrance_and_out_real_damage
--include-ignored`, exit 0 only on "test result: ok"). Every verdict was a real test failure
(FAILED lines in the /tmp/bisect13_*.log runs), not a build error.

**First bad commit: 7876b7feb** ("Ally deaths blame the right killer; level music machines;
turrets take rounds; laser cuts bolts only") — the commit immediately after the good d09f8b2b3.
At 7876b7fe the death is byte-identical to the ledger: `Bond died at tick 2448 (frame 2448) at
(4.03,-368.39,33.35)` [L the run log]. At d09f8b2b3 the test passes on my own run (2 passed:
the d3 real-damage walk and the agent sister) [L this session's run].

**The mechanism**: 7876b7fe replaces 12's keypad laser burn with the typed code
(`tower2.rs`, the `0x1ae3d8` keypad path [M lock.rs's key-ring arms]). The 12 leg
re-times, and 13's lobby then meets a different spawner-wave state: the wave lands ON the east
lane while Bond stands in it, and a shotgun guard (skin 0x5000008, weapon 0x1c, at
(5.0,−368.4,27.6)) unloads **7 pellets, −0.5 + 6 × −6.0 = −36.5 in one tick** at d 2.6 [L the
NF_DMG_TRACE run at 7876b7fe, frame 1647].

## Is the new damage game-correct? [M] (the task's second question — DONE: yes)

The per-pellet math is the game's own pipeline, every factor cites code:
- the pellet damage × count from the weapon record's fire script (the 12-spawn ledger measured the
  same gun at 8 × −3.0 at d 3.5 [L the civ-unignore trace]);
- the Agent-difficulty pain scale — `Player_HandlePain` 0x1902c8 scales by the difficulty's
  Plr_DMod_* row (2.0 at Agent here: 3.0 → 6.0 per pellet) [M];
- the point-blank bonus — the port's `player_hit_damage` (drone.rs, mirroring the
  `DroneWeap_DoBulletAccuracy` 0x179740 family): `shooter_dist < DroneFiring_TooClose_Distance`
  multiplies by `DroneFiring_TooClose_Damage`; d 2.6 is inside the band, so 3.0 × 2 × 2 = 6.0 per
  pellet [M].
So the VOLLEY is game-correct; what is not game-fair is the walk's exposure — a player does not
stand at d 2.6 of a posted shotgun guard. The route/stand work below targets exactly that; no
damage or health number was touched anywhere.

## The re-tune shipped on the branch (real input only)

- `tower2::PRE_CLEAR_NORTH_LOBBY` (new opt-in atomic, the `PRE_CLEAR_SOUTH_CORRIDOR` precedent):
  at "the north lobby 2" leg end, stand-and-fire the alive team-1 guards within 12 u that have an
  exposed part, nearest first, up to three. At master the fight's own cap left #6/#7 alive and the
  leg resumed into their pursuit — #6 walked Bond down and unloaded 4 × −14.0 at d 2.5–2.9 (the
  TooClose bonus) for the tick-5914 death on 12 [L the master run's trace]. With the flag the 12
  leg survives every run [L]. Flag default false: every other caller's legs are byte-identical.
- 13: a `clear_the_visible` sweep at the east-lane leg end — takes the wave guard before the
  re-anchor (the tick-1647 volley's killer died to it, killed true at tick 724 [L]).
- 13: the west-mid pre-clear stand REMOVED from that leg end and re-hung at the north-west end —
  the stands themselves were the exposure window (the tick-1995 death: #10's 0x19 at d 13.2 and
  mezzanine #4's 0x18 at d 22.2 bled 31.5 during four stands whose own boxes never opened to the
  walker's gun rays) [L].
- The sweep's `aimed_stand` cap 2400 → 700 (the kills land in ~50–350 ticks; a hopeless stand
  statue-stills Bond through the 1500-tick no-hit give-up) [L].
- `precise_fights` left OFF for this walk (the 0.01-rad tap gate was 11's 7-round-budget lever;
  here it changed nothing at the death sites — two runs died tick-identical with it on and off
  [L]) — the ammo it would spray comes back through `lane_floor_drops`.

**State**: the walk now crosses 12 and reaches 13's west legs, then dies at tick 2242 at
(−12.59,−368.39,39.91) inside the north-west sweep's failing #9 stand (24 hits for 82.5) [L].
**The un-ignore is NOT shipped** — the test still fails, and a red test does not un-ignore.

## What is left (the next session's levers)

1. The north-west sweep's #9 stand fails (killed false) while the unseen guards bleed Bond out —
   the same exposure asymmetry: guards shoot from their eye cone, the walker's gun-ray exposure
   needs the eye AND both gun-offset rays. Levers: sweep from the "past the glass band" end
   instead (the walls differ), or give the sweep's failing stands an immediate reposition (walk
   the next leg on give-up instead of standing again).
2. The margin math: Bond enters 13 at ~82.5 and the lobby fights take ~80. The 12 leg's "other
   17.5" (the spawn-lobby trades) is the cheapest margin to recover — the resupply fights on 12
   (the north lobby pre-clear already took #7 at d 5.6).
3. If the fights' kill rate is the wall, the Spas12 pickup the level grants [D §5] is the
   in-route answer (a real player takes it); the take machinery exists (the Pickup_Handler path).

## Runs this session (every binary whose code changed)

- `walkreal_07000013_live` (the d3 real-damage walk + the agent + operative sisters): ran ~10
  times across the bisect and the re-tune iterations; sisters 2 passed on every run; the d3 walk
  red on master and still red on the re-tuned tree (death 2448@7876b7fe → 12-death on master →
  2242 on this tree) [L all run logs].
- `git bisect run` covered d09f8b2b3, 7876b7fe, 63b05cca, 2998b6a7, 6186d853, 290dc4b0, and the
  branch tip.
- NOT run (no time): `interactables_07000013_live`, the other `walkreal_*_live` binaries, the full
  suite. The shared-helper change (tower2.rs) is opt-in-gated, and the d1/d2 sisters that share
  the file ran green every run — but the battery is owed before any merge.

## Live checks for the main agent

1. Fresh boot of 07000013 at Agent: stand Bond where the walk stands at the west-mid
   (−4.95,−368.39,38.96) and let the lobby wave engage — do the desk-block guards (#10-class,
   weapon 0x19) hit from d 13+ in the real game, or does the port's eye-vs-gun-ray exposure
   asymmetry (they shoot, his gun rays find no box) let them? `gdbwatch.py` on the guard's fire
   script vs `gun_through_wall` 0x178520's gate.
2. Fresh boot: the spawner wave (start ch 0x29 = the lobby guards' death channels [D §4]) —
   record where the wave lands (x/z) and how long after the first kill (the walk's wave guard
   stood at (5.0,−368.4,27.6), ~500 ticks after the pocket kills [L the trace]).

## Usage

not visible to me (no token/credit/context figures are exposed in this environment).
