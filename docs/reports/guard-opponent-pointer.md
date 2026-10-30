# guard-opponent-pointer — the drone opponent pointer (+0x170) set the way the game sets it

2026-10-30, branch `s/guard-opponent-pointer` (from origin/master 04820075). The port's
"guard currently sees Bond" stand-ins for drone +0x170 are gone: the pointer is now a field
(`Drone.opp_bond`) written by the game's own writer path, and every reader the task named —
NoOpponent 0x60's return to Attack, `NDrone2_DoorGuardHandler`'s zone gate, the Interogate
family, the abseil arms, `HasOpponent` — follows the pointer, not sight.

## The census: who writes +0x170 (the method)

An ELF-wide sweep of `ACTION.ELF` (capstone over the whole text segment, every `sw …,0x170($reg)`;
EE MIPS is little-endian, LQC2/SQC2 words skipped) finds 11 raw sites; 7 are `0x170($sp)` stack
locals of unrelated structs, one is the video-codec struct. Exactly three are Drone_tag stores:

1. **`NDrone2_SetOpponent` 0x1422d0:20** — THE writer (`sw $s1,0x170($s0)` at 0x142300). It also
   allocates the +0x174 target id, maintains the 0x299110 registry and the +0x1F0 aim pos.
2. **`Drone_Delete` 0x136ce8:53-54** — clears every drone's pointer TO the deleted object.
3. **`Drone_InitComms` 0x1399ec** — an MP bot-init struct block-copy (0x160..0x178), not a
   gameplay writer.

`NDrone2_SendAttackMessage` 0x14a410 and `NDrone2_ChangeToAttackMode` 0x14a648 — the two the task
named as candidates — are verified NON-writers by disassembly: the former only LOADS +0x170
(0x14a444/0x14a488) and alerts to it via `DroneAlertToObject` 0x14a538 (which writes the global
alert-target record 0x2990a0, no drone field); the latter never touches the offset. SetOpponent's
callers: `NDrone2_FindOpponent` 0x1424e0 (×5) — the only SP path — plus MP (InitComms, BOT_*,
BotGlobal/BotDead/BotIdle/BotDeathAnim/BotDeathByExplosion). FindOpponent runs every tick from
`NDrone2_ControlSTANDARD` 0x1490f0:167 and once at spawn from `NDrone2_PostLoad_Init`
0x14d890:14-17 (then copies +0x170 → +0x360).

**The team-1 fold** [M 0x1424e0:100-121]: for `+0x44 != 2`, `SetOpponent(player)` EVERY call —
zeroed only when `PlrStat_OkToUpdate` fails (:104-108), the player is class 3 with record health
≤ 0 (:111-114), or class 0x12 (:116-118). **No sight test anywhere on the path.**

## What the port change is (crates/nf-game/src/drone.rs)

- `Drone.opp_bond: bool` (new field, own line): the +0x170 pointer for team != 2. Written every
  tick by the fold `team != 2 && stat_ok && bond_alive` (the port's `stat_ok` = `PlrStat_OkToUpdate`
  [M 0x1993e0], `bond_alive` = the health fold covering both death arms), and at spawn in
  `type_setup` (the PostLoad_Init fold). Team 2 keeps the scan pick in `opp_drone` [M :101].
- `Drone::opp_ptr()` = `opp_bond || opp_drone.is_some()` — the pure +0x170 read.
- **`has_opponent` unified** [M `NDrone2_HasOpponent` 0x14ad58:8-15]: pointer OR the +0x238 grace,
  EVERY team. Master had the grace team-2-only (the combat-sight-loss-exit report's every-team
  claim did not survive the merge) and folded team-1 to bare `bond_alive`.
- **The +0x238 stamp now written for team 1/3 too** [M `DroneVision_HaveOpponentSight` 0x175ef0:33-36
  reaction gate, :66-68 stamp — reached from `ProcessDroneSight` 0x176a28:222-231 with NO team
  gate]. Master only stamped team 2. The visible branch also requires the pointer now
  [M 0x176a28:222-231 `+0x170 == 0` → the lost-sight arm], so the grace expires after death.
- **NoOpponent UPDATE/TIMER_C** [M 0x1650a0:27-47 cases 3/0xc]: `opp_ptr()` — the pointer alone,
  no grace. The stale "[S] which writer re-points +0x170" comment is replaced by the citation.
- **Interogate family** (ENTER/TIMER_C arms of 0x161a40/0x1616c0/0x161860 [M :26/:159/:19/:52])
  and the **abseil Slide/Hang arms** [M 0x169730:46-51, 0x169940:42-47]: `bond_alive` → `opp_ptr()`.
- **DoorGuardHandler** [M 0x1416d8:9-11]: the zone gate is `opp_ptr()` (was the `los` sight
  stand-in). The height gate |opp Y − my Y| ≤ 2.0 stays [M :22 + disasm 0x14170c-0x141738].
- The router head keeps `has_opponent` (= the game's pointer-or-stamp gate [M 0x148338:19-31]).

## What of combat-sight-loss-exit is on master

All of its surviving semantics: the `if false` sight-based router head deleted (its first gate IS
the pointer/stamp check), `has_opponent` = pointer-or-30-tick-grace, the team-2 re-pick fold, and
the `bond_alive` team-1 fold at drone.rs's tick. Two things from its report were NOT on master and
this task restores exactly: the every-team grace in `has_opponent`, and the team-1/3 +0x238 stamp
(its "opp_pointer()" helper name never landed on master — the NoOpponent exits read `has_opponent`
with the [S] comment my task was assigned to replace).

## Tests (all green, NF_ISO set, --test-threads=1)

- `door_guard_live` (07000007, the only door-guard level [D census]): **5 passed** — the three
  pre-existing pins stay green with the pointer gate, plus two new:
  `the_zone_dies_with_bond_even_seen` — a ray-visible Bond in the challenge band reads zone 2,
  his health 0 zeroes the fold [M 0x1424e0:111-114] and the zone drops to 0 within 8 ticks
  [M 0x1416d8:9-11] while the sight is still present (the gate is the POINTER; this bites the old
  stand-in, which kept a seen corpse's zone flowing); `the_zone_bands_read_a_bond_below_the_floor_
  within_the_height_gate` — the bands' planes are horizontal (b = 0 [M 0x141360:115-174 +
  0x1e4498]), so Bond 1.9 below the floor stays in-band and inside the [M 0x1416d8:22] gate and
  still challenges.
- `ai_missing_states_live` (07000005): **9 passed** — the seven pre-existing pins stay green,
  plus `a_sight_lost_guard_stays_out_of_no_opponent_while_bond_lives` (150 ticks of lost sight
  over a LIVING Bond never reaches 0x60 — the head is the pointer/stamp gate [M 0x148338:19-31])
  and `the_router_holds_through_the_stamp_grace_after_bonds_death` (0x60 does not land before the
  30-tick stamp grace [M 0x14ad58:11-14] could hold).
- `find_opponent_live` 3, `vision_bit_census` 1, `walkreal_07000007_live` **2 passed** — the
  door-guard level's real-input walk (party start → Drake → the exit) is green with the port.

Touched-binary total via the fast runner: **20 passed, 0 failed, 0 ignored**
(18 from the four-binary chunk + 2 walkreal; `suite.py --only` totals quoted verbatim).

## The full suite: honest status

NOT completed this round. This sandbox kills EVERY background process between tool calls
(verified with a `setsid nohup sleep 120` probe — dead before the next call), so the fast runner
only survives inside ≤10-minute foreground calls. The four-crate build is cached ("build ok: 934
test binaries in 0 s"), but a full 934-binary run does not fit the 2-hour window alongside the
port work, and it was not running when the stop-new-work line arrived. What ran instead is the
guard-behaviour-critical set above (20/0). The last full-suite record on a merged tree remains
the level-13 round of 09:15 UTC today: 1510 passed, 3 failed (2 no-GPU pylon env failures +
net_internet_live), 173 ignored across 664/915 binaries.

## What's left

- The full-suite run on the final tree (the next session: `python3 tools/test/suite.py` chunks in
  the foreground, or a sandbox that lets background processes live).
- The lost-sight ZONE case: on 07000007 every in-band spot has a clear eye ray (the pair stands
  on open floor; floors cull their back face upward [M 0x10a8c8:168]; doors are not ray-geo in
  the port), so the old `los` stand-in and the pointer agree on every reachable spot while Bond
  lives — the pointer's win there is the death arm (pinned) and the code citation. A live probe
  (below) can pin the sight-independence directly.
- `Drone_Delete` 0x136ce8:53-54's pointer clear is not modelled (a deleted opponent's pointer
  clears in other drones' +0x170) — unobservable in SP, where the only shared opponents are
  team-2 drone picks that the scan re-validates [M 0x1424e0:157-166].
- A team-2 door guard would band its drone opponent's position; the port bands the player
  ([S] no team-2 door guard exists — the 0x31 mode is guard-only; marked in the code).

## Live checks for the main agent

- [S] On 07000007 at the 441/442 door-guard pair (Drake's floor): hide Bond behind the geometry
  just outside the challenge band's sightline (a pillar/plant the port's ray world lacks), close
  enough that his xz stays inside the s0 ∈ [0,2) slab. gdbwatch/fieldprobe a guard's +0x2CC:
  expect the zone value to keep flowing (1/2/3) while unseen — and to zero the tick Bond dies.
- [S] gdbwatch a hall guard's +0x238 on 07000001 after Bond's death: the stamp holds its last
  value, +0x170 zeroes that tick, and the guard leaves combat for 0x60 only ~30 frames later
  [M 0x14ad58:11-14 + 0x148338:19-31] — the port now models exactly this.

## Usage

Token, credit and cost figures: not visible to me (this worker's CLI exposes no usage dashboard;
the IM gateway metadata carries only session/chat ids, no numbers). Context-window figures: not
visible to me. The only usage-shaped numbers this environment exposes are filesystem/process ones
(rootfs 9.9G, 1.6G free at report time; 3 GB RAM, the test build cached in /home/z/target-cache).
