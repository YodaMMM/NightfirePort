# Report: level-13-guards-start-alerted

Task: Phoenix Fire Part 4 (07000013) — the lobby guards start alerted at level start, before Bond has
been seen. Find each guard's start state on the game's path and port what the game does.

## What's done

**The roster, on the game's path** ([D] census via `crates/nf-view/tests/census_07000013.rs`, params
decoded with the game's own `bitDescs`/`bitMasks`/`DroneTypeSettings` tables from ACTION.ELF):

- 12 kind-0x0F placements. All carry skin p0 ∈ {0x05000004/06/08}, start script p1 = 0x6000000,
  difficulty gate p2 ∈ {1,2,3}, wait channel p3 (0x28 on seven of them = TouchOnce #58's out at the
  spawn [D], 0x2a on #326 — a dead channel [D]), spawner group p4 ∈ {0,1,2}, mode row p5 = 0x64 (=100
  → `DoModeSettingsNEW` [M 0x14b300:495-501]), behaviour type p15 = 3, block-1 count p16 ∈ {0,3},
  behaviour blocks p18-20/p23-25, stats p26-32 [D census dump].
- At Agent difficulty (1) exactly **two** guards create: #318/#319 [M Drone_Create 0x136be0:49 p2 gate].
  The six p4≠0 rows are spawner-group templates — `DroneSpawner_Init` 0x137aa0 copies their DIVars and
  deletes the objects [M 0x137aa0:56-63], and both spawners' start channels (0x29/0x2a) have no writer
  [D]. The port's roster matches (spawn_roles).

**The game's start-state rule, verified end to end**:

- Placement param 15 = the behaviour type, p16 = block-1 count; `GetDroneTypeAttackTypeFriend`
  0x14cbd8 computes +0xC5 = `GetDTYPENEW(type0, count1)` [M 0x14c600:77-80] — for p16 = 3 that is
  `GetDTYPENEW` case 3 → mode 7 unless behaviour prop 0x56 [M 0x14c938:111-117]. obj+0xF4 = 0 (Global)
  [M 0x14cbd8:62-64], the start state +0x5A2 = `DroneTypeSettings[+0xC5].start` = 0x85 for mode 7
  [L DroneTypeSettings row 7 from the ELF; L ee_start_05.bin drones 5/6 carry +0x5A2 = 0x85, and drones
  8/9 — mode 0 at full alertness — carry +0x5A2 = 0x56: the alertness fold is real in the game].
- The alertness float +0x50C = table[prop 0x20 of block 1] = `0x2f5cc8[prop19 & 3]` = **1.0 for ten of
  the twelve guards** (param 19 & 3 = 3) [L table {0,0,0.66,1.0} read from ee_start_05.bin AND
  ee_start_07000001.bin @0x2f5cc8]. The port's `ALERTNESS_LEVELS` was already exact.
- The intro ScriptPlayer #65 (script 0x060006D7, pack 07900013) **freezes the NPCs at its tick 0
  (stream-6 event 5) and re-enables at tick 349 (event 7)** [D stream decode]; `Script_EventHandler`
  case 5 → `Drone_EnableAll(0)` → `NPCGlobals = 0` [M 0x1d94c0; 0x1383e8], so the guards take no first
  ENTER and no sight pass until the freeze lifts. The port already modelled the event-5/7 pair
  (nis.rs 1003/905); the probe shows the freeze live (npc_enabled false t=0-210, true from ~230).

**The two port bugs found and fixed** (crates/nf-game/src/drone.rs):

1. **The sight pipeline ran without the vision bit.** `DroneVision_ProcessDroneSight` 0x176a28 computes
   the visibility value (+0x230) only for drones with `+0x4F8 & 0x10000` (set by the sight-capable
   states' ENTERs — Idle 0x15bf18:30, Alert 0x15c248:19, Patrol 0x15c498:19, GoToGoal 0x165640:22,
   EnemyRunToPoint 0x168910:18 — and cleared by RouteMsgDCV on every state change), and the third loop
   refuses without the same bit. There is **no alertness bypass**. The port's per-tick pass had
   `dr.vision_on || dr.alert >= 1.0` — a full-alertness placement (ten of the twelve lobby guards)
   therefore ran the whole sight chain from Global/WaitSwitch with zero vision — the "already alerted
   at level start" symptom. Removed the bypass ([M 0x176a28]; the bypass dated to the first AI
   milestone, no test pins it).
2. **`EnemyRunToPoint`'s Tower resume never kept Attack.** [M 0x168910:22-36]: the resume +0x5A2 =
   0x56 (Attack) by default; the 0x41/0x11/0x12/0x13 clause — gated on `+0x4F8 & 0x4000000 == 0` (the
   port's `sighted` latch, not `ever_seen`) — only LOWERS it: Idle under 0.66, Alert under 1.0, and at
   +0x50c ≥ 1.0 the Attack resume stands. The port gated on the wrong latch and folded every resume to
   Idle/Alert, so a waking guard landed in 0x05 Alert instead of the game's 0x56 chain. Fixed both the
   gate and the value.

**Verification**: `crates/nf-view/tests/guards13_start_live.rs` (new, green) — fresh 07000013 load on
real input: the two-guard roster and their mode/wait-channel [M/D]; at ticks 30 and 200 every guard is
in Global with `sighted == false` and `visibility == 0.0` (the freeze + the vision-bit gate); ch 0x28
fires at the spawn [D]; after the wake no guard sits in WaitSwitch (the channel is already on [M
0x15b830]) or in 0x05 Alert (the resume keeps Attack at full alertness [M 0x168910:27-30]).
`guards13_start_probe.rs` (assertion-free probe) recorded the before/after trajectories. The
`census_07000013.rs` dump and an ELF-table walk produced the [D]/[L] numbers above.

**Full suite** (slim ISO; the sandbox killed every detached `tools/test/suite.py` run and one
sequential run did not fit a tool window, so the same set ran per crate with `--test-threads=1`,
the suite's own flags): nf-formats + nf-audio **133 passed, 3 failed** — the 3 are exactly the slim
ISO's allowed `disc_golden` failures (bigf_header_fields, bigf_tables, directory_tree golden
manifests); nf-game **933 passed, 0 failed, 8 ignored**; nf-view **178 passed, 0 failed, 4 ignored**
when the window cut mid-crate (the run was alphabetical, last completed binary
`campaign_flow_live`; the remaining ~30 nf-view binaries were not re-run in this sandbox — the
touched nf-view tests, `guards13_start_live`/`guards13_start_probe`, ran green standalone above).
Total: **1044 passed, 3 failed (the allowed disc_golden set), 12 ignored** + the un-run nf-view tail.

## What's left

- The end-to-end post-wake outcome (whether the real game's lobby guards, once re-enabled at the
  intro's tick 349 with ch 0x28 already on, actually SEE Bond at the spawn nook) depends on the
  game's AI-cel line-of-sight ray, which the port stands in for with its collision-world ray
  ([S] documented at the sight pass). The port's guards do re-acquire him within ~2 s of the wake
  (in-cone, vis 1.12 with the ×3 alertness multiplier [M 0x175af0]). If the real game keeps them
  passive longer, the ray or the cone source (+0x380 head yaw vs body yaw) is the next suspect.
- The port's intro ScriptPlayer ends at ~tick 230 of the script's 350 [D len] (the freeze + Bond
  release land together there); the game's event-7 re-enable sits at tick 349. The wake chain is the
  same either way, but the 120-tick early release is unexplained and worth its own task.

## Live checks for the main agent

1. Fresh 07000013 boot (Agent), gdbwatch on the first difficulty-1 guard's sight flags: the guard
   objects are the 2nd/3rd drone on the 0x298E14 list; watch `+0x4F8` (obj+0x608) around the intro's
   end — expect the 0x10000 bit to rise only when the wake chain starts (0x85 ENTER), not before;
   then `+0x228` (obj+0x338) bit 2 (the SeekOpponentLOS ray result [M 0x175cb8:104]) — does it set
   while Bond stands in the spawn nook (dist ≈ 17.8, in-cone, placement yaw −2.85 vs toBond −2.86)?
   Expected if the game keeps them unaware: bit 2 stays clear until Bond steps out (i.e. the game's
   ray is blocked where the port's collision ray is not).
2. Same boot, `switch_channels[0x28]` (0x26fc92 + 0x28): the tick it rises relative to the intro's
   event-5 (tick 0) and event-7 (tick 349) — the port fires it at ~tick 230 of 350; the game's touch
   scan may or may not run under `NPCGlobals = 0`.
3. The intro length: watch the intro SP's free (script 0x060006D7) — the game runs 350 ticks; the
   port's ends at ~230.

## Usage

not visible to me (no token/credit/context-window figures are shown in this environment).
