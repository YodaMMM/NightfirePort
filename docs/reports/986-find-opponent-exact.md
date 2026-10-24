# 986-find-opponent-exact

2026-10-24, branch `s/986-find-opponent-exact`. NDrone2_FindOpponent's team-2 scan: the exact
alive filter and the shared per-call LOS budget, replacing the port's `present && enabled &&
!corpse` fold and its one-LOS-per-drone approximation.

## 1. What DAT_00299092 is

**The shared LOS-probe counter** (u16 at 0x299092):

- **Incremented** when a drone-vision probe reaches the expensive `Collide_LineOfSight` stage —
  `DroneVision_CanSeeObjectFrom` 0x176f18:68 and :94 [M] (the increment sits between
  `AINetwork_TestRayCels` and `Collide_LineOfSight`: a cel-rejected probe consumes nothing),
  `DroneVision_LineOfSightToObject` 0x178390:19 [M], `DroneVision_CanSeePosition` 0x178438:69 [M].
- **Reset** only by `Drone_LevelReset`'s `memset(0x298e10, 0, 0x1d10)` [M 0x13a5bc-0x13a5c8, the
  disassembly of 0x13a5a8 — the range covers 0x299092]. **No per-frame reset exists** in the
  decompile — the counter is monotonic within a level (the task's "per-frame counter" premise is
  corrected here).
- **Consumed** by the looping consumers, which snapshot it at entry and stop when the live counter
  exceeds the snapshot — one budget-consuming probe per call:
  - `NDrone2_FindOpponent` 0x1424e0: the entry snapshot at 0x142554 [M disasm, `lhu s7, 0x282(a1)`
    with a1 = 0x298e10], the scan's bail `if (uVar1 < DAT_00299092)` at :239-242 (disasm
    0x143144-0x14314c, `lhu` + `sltu`), the cursor saved on the just-probed candidate;
  - `DroneVision_ProcessDroneSight` 0x176a28:71/:177 [M] — one snapshot for the whole per-frame
    drone loop (this is where the sharing bites across drones);
  - the snapshot-compare caches `Drone_IsCoverNodeUsable` 0x1386d8:175/:190 and
    `DroneSpawner_Control` 0x137e68:216/:224 [M].

Per FindOpponent call the budget is therefore: the held-opponent's `CanSeeObject` probe [M :152 —
it runs BEFORE the scan and after the same snapshot] plus at most ONE scan probe [M :231-242]. A
call with no held opponent spends ≤ 1; with one, ≤ 2. N allies scanning in one frame spend ≤ N (+N
held probes) because each call re-snapshots at its own entry.

## 2. The exact alive filter

The scan's candidate gate [M 0x1424e0 disasm 0x143098-0x1430f8] — the decompile at :220-226 is
misleading (the `(fe & 1) != 0` with the goto reads inverted; the delay slots settle it):

- the record's (+0xE0) +0x4F8 flags: **0x600 CLEAR** — `NDrone2_SetAsDead`'s 0x200 [M 0x14a2f0:9]
  and `NDrone2_PostDroneControl`'s health-dead 0x400 [M 0x14aff8:12: 0x100 set ∧ obj+0xFA |= 0x20 ∧
  health ≤ 0 ∧ 0x200 clear];
- **0x100 SET** — the AI-active bit (the port's `enabled`, set at the same activation — the port
  comment at the `enabled = true` site; `NDrone2_Enable(0)` clears it via the 0xffe7fe5f mask
  [M 0x149dd0]);
- the record's **+0xAC health > 0.0** [M 0x1430c4-0x1430cc];
- the obj's **class +0xFF != 0x11** [M 0x1430d8-0x1430e0] — vacuous in the port (the port's list
  holds only class-2 NDrone2 records; noted in the helper's doc);
- **obj+0xFE bit 0 CLEAR** [M 0x1430e4-0x1430f0 — the delete-me flag, docs/fields/obj-tag.md:44].

The held-opponent re-check [M :161-172] is the same test without the class arm, and it runs AFTER
the probe [M :152] — the port had the order reversed (`alive && los`), so a dead held opponent
consumed no budget where the game spends one. Fixed.

The port's `Drone::find_opponent_alive()` = `enabled && !corpse && health > 0.0 && !delete_me`
with the fold documented: `!corpse` covers the 0x200 arm (the port's death paths set both),
`health > 0` covers the 0x400 arm (the game's PostDroneControl flags health-dead every control
pass; the port reads the health directly). New `Drone.delete_me` field (own line), set at the
three sites where the port's comments cite the game's `obj+0xFE |= 1`: the DEAD-grace expiry
(0x163fe8 case 3), `DeleteMe` 0xb4 (0x173f00 / `Drone_FellOutMap` 0x13a878), the 0700004a abseil
soft-delete — in the port they coincide with `present = false` today, so the flag is exact-but-
redundant until a one-tick delete window is modelled.

## 3. The port changes (drone.rs)

- `Drone::find_opponent_alive` (pub, for the tests) — the exact gate above.
- `Drone.delete_me` + the three writers + both constructor inits.
- `Drones::los_probes` (pub u16, own line) — DAT_00299092; zeroed in `Drones::new` (the port's
  level reset — the constructor comment already cites `Drone_LevelReset` 0x13a5a8).
- `team2_opponent_picks` (`&mut self`, pub for the tests): the held probe first (consuming), the
  alive filter second; the scan's cheap gates use `find_opponent_alive`; each probe increments
  `los_probes`; the bail is `snapshot < los_probes` with the cursor on the just-probed candidate.
  The port's LOS has no cel pre-stage, so every port probe consumes budget where the game's
  cel-rejected probes do not (the documented stand-in, noted in the function doc).

## 4. Tests (crates/nf-game/tests/find_opponent_live.rs, 3 green)

- `scan_skips_dead_candidates_and_picks_the_live_one` — the gate arms pinned directly on
  `find_opponent_alive` (health 0, SetAsDead corpse, delete-me), then through the scan with the
  cursor parked on the guard: the healthy guard is picked with exactly one probe; each dead arm
  leaves him unpicked (a cheap-but-dead guard would have been probed and picked the same way —
  the pick difference IS the gate).
- `los_budget_matches_the_counter_rule` — five scenarios, each cursor-pinned: (1) one scan probe,
  the live guard wins; (2) holding the guard himself: one re-validation probe only [M :152 + the
  :223-226 skip]; (3) held + a second candidate: two probes; (4) a buried first candidate: one
  failed probe, no pick, the cursor saved [M :240], the resumed call spends one probe and picks a
  live guard; (5) N allies: the counter grows by N — each call re-snapshots [M 0x142554].
- `ally_picks_the_guard_through_the_tick` — through the real tick, the ally's opponent becomes
  the live guard.

## 5. What ran (all green unless noted)

`nf-game --lib` 546; `find_opponent_live` 3; `ai_live` 36; nf-view `walkreal_07000008_live` 4
(the ally tests stay green), `ally_stuck_sweep_live` 1, `ai_traces_live` 4; nf-formats + nf-audio
120 ok with the 2 allowed `disc_golden` slim-ISO failures (the task book's exception).

## 6. What's left

- The other counter consumers are not instrumented: `ProcessDroneSight`'s per-frame pass
  (0x176a28 — the port's sight round-robin at drone.rs's `dr.los` site), `SeekOpponentLOS`
  0x175cb8, `LineOfSightToObject`/`CanSeePosition` (the cover/spawner paths). The counter is in
  place; each site needs its increment + its own snapshot/bail semantics pinned.
- The cel pre-stage (`AINetwork_TestRayCels`) is not modelled: the game's cel-rejected probes are
  budget-free, the port's are not ([S] documented stand-in).
- The class-0x3b control-object arm of FindOpponent [M :124-146] and the MP arm [M :254-330]
  remain unported (unchanged scope).

## Live checks for the main agent

1. gdbwatch `0x299092` (u16) across a 07000005 session with allies: expect it to grow
   monotonically (no per-frame reset) and to jump by 1 per drone-vision LOS that reaches
   `Collide_LineOfSight` — the increment sites 0x176f18:68/:94, 0x178390:19, 0x178438:69.
2. A team-2 ally (dtype 0xC) holding an opponent: watch `Drone+0x2C0` (the scan cursor) on a tick
   where the held opponent is visible — the cursor should NOT advance past a probed candidate
   unless the counter grew (the bail) or a candidate won.
3. Poke a guard's +0xAC (health) to 0 while an ally scans him: the ally must not pick him on the
   next FindOpponent (the exact filter), and `0x299092` must not increment for him.

## Usage

not visible to me (this session exposes no token, cost or context-window figures to copy).
