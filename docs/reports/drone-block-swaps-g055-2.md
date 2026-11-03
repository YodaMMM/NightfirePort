# drone-block-swaps-g055-2 — the G-055 block swaps without the six regressions

Branch `s/drone-block-swaps-g055-2` from origin/master 6f16146e. The G-055 branch
(`origin/s/drone-block-swaps-g055`, already an ancestor of master but reverted by c0450ef6) was
re-landed with `git revert c0450ef6` — `git merge` was a no-op (revert-of-merge), and the
revert-of-revert applied cleanly, restoring the three swaps and
`crates/nf-view/tests/drone_block_swaps_live.rs` (which master had lost with the revert).

## Verdict per test (4 of the 6 greens, all pins in place)

1. **rook06_realpath_live::the_switch16_guard_fights_through_the_door47_doorway — GREEN.**
   The test's expectation was wrong, not the port. The game flushes the ch-16 guard #439 at his
   FIRST attack: his alert mode is 2 (p21=2 [D census #439 @(30.25,41.08,−16.83)]), the tag p8=100
   keeps +0xC6 alive [M 0x14cbd8:27-45], so +0x5A4 = DroneTypeSettings[row 2].alt = 0x2a
   [M 0x14cbd8:103-114]; `DroneFunc_FirstAlertState` returns 0 on 07000006 for every alert arm
   [M 0x147f58:8-58 — only 0x200000 on 05/07/41 returns 0x39]; the common path clears the second
   switch [M 0x148010:53]; so the FirstAttack alternate-state tail [M 0x148010:100-131] runs the
   ChangeToAttackMode inline: +0xC6=2 → +0xC5, the mode-2 sniper retype 0x19 + the stationary
   hook, and with +0xC48 == 100 (p5=100 [D #439]) the second block in. Re-entries never re-flush
   [M 0x148010:17-19] — the OLD every-re-entry flush (rook-door-06's jamb park) stays impossible;
   the first-pass flush lands him at his guardroom POST, not the jamb, and the door/jamb
   pins still hold (verified: the test runs green with the flush in). The test now pins
   `mode == 0 || (mode == 0x19 && using_2nd)` with those citations.
2. **abseil_elevator_live::the_elevator_jumper_opens_the_door_and_attacks — GREEN.** The game's
   tail returns the OLD alt state [M 0x148010:128-131]; this level's mode-alert-7 guards hold
   +0x5A4 = DroneTypeSettings[row 7].alt = 0x85 [M 0x14cbd8:103-114], so the flush chain is
   0x85 EnemyRunToPoint → 0x89 RunToAlarm → GoToGoal 0x63 (the combat chain's own run-to-goal
   leg [M 0x164c08 case 0xC]). GO_TO_GOAL was added to the accepted combat-chain states.
3. **walkreal_07000014_live::walk_the_tripbomb_route_pick_up_place_and_shoot_it — GREEN.** The
   flush changes the guards' fight around the placement so a thrown guard grenade enters and
   leaves `grenades`, staling the placement-time index (len-1/index-1 panics at three sites).
   The test now re-finds the stuck bomb (`trip_stuck_now`) at every use. The walk also kills the
   trench guard #270 (p5=100, p8=100, p21=3 [D #270] — block B [a0800200, df720813, 44280000]
   hunts) at the crossing landing with the walker's own `shoot_loose` — real input.
4. **walkreal_07000016_live::the_chasers_punch_lands_the_record_damage — GREEN.** The flush
   writes her return state 0x56 [M 0x148010:102]; she then works the shelter door at 2.3 m
   through the spawn wait and her -1.20 stars/6 ticks take the idle stand from >30 to dead
   (the 8715 death). The wait now counter-fires one tick at a time while she is above 55 health
   (the punch pin needs her alive), gated on `!w.tops_up()` so the tops_up walks keep their
   exact pre-flush RNG stream. The pit clear (`clear_the_pit_shooters`) bails below the 30.0
   floor the caller's loop uses.

Also green on the branch: `drone_block_swaps_live` 3/3, the whole 07000014 binary 7/8, the
07000016 binary 13/14, rook06_realpath_live 1/1, abseil_elevator_live 3/3, nf-game lib 658/658.

## What's left (both are walk-survival against the now-faithful hunt)

1. **walkreal_07000014_live::walk_the_infiltration_to_the_delta_exit_real_damage** — with the
   flush in, the west route's p5=100 guards (e.g. #281/#283 p21=3 [D]) hunt from their first
   attack and Bond dies mid-route (last seen: tick 14137, 107 hits over 19 fights, dying around
   (−48, 30) after the Kiko band). A gated `clear_hostiles` helper (real-damage walks only,
   `!w.tops_up()`, nav-web walk-home) is in the file and helps but the swarm re-displaces Bond
   during the walk-home; the descent-landing position pin (29.99, 0.61, 38.00) then fails. Next:
   clear BEFORE the descent legs (the fights upstairs displace less), or hold the door-wall
   lane while firing (the direct walk-home hits the west jamb at x ≥ 30.3 — the file's own note).
2. **walkreal_07000016_live::walk_the_chasers_defeat_completes_the_mission_on_real_damage** —
   the counter-fire carries Bond past the old 8715 death; he now dies at 9241 on the pit floor
   (−6.34, −22, −163.7) — the chaser plus the pit squad during `kill_the_chaser_with_range_
   keeping`. Next: clear the pit squad (never the chaser) inside the kill loop's backpedal
   branch, the same way the punch window does.

No `crates/nf-view/tests/common/` files were touched — every change is test-local
(rook06_realpath_live.rs, abseil_elevator_live.rs, walkreal_07000014_live.rs,
walkreal_07000016_live.rs), so the shared-helper opt-in rule does not trigger. The full suite
did not fit this sandbox window (setup + the walk grind consumed it); the runs quoted above are
the real ones this session, each named binary run in full with `--test-threads=1`.

## Live checks for the main agent

1. On 07000006 (PCSX2, walk the yard route to the guardhouse, wake the ch-16 pair via the step
   triggers, stand in the guardroom): gdbwatch Drone #439's +0xC5 (expect 0 → 0x19 at his first
   attack), +0x29 (expect 0 → 1, the second block in), +0x5A4 (expect 0x2a before, 0 after),
   +0x4D8 (expect +0x4DC → +0x4E8). This pins the flush outcome the test now encodes.
2. On 07000014 (the trench after the SkyBridge drop): gdbwatch Drone #270's +0x4D8 (expect
   +0x4DC → +0x4E8 at his first attack) and +0xC5 (expect 0 → 3? — row 3's mode; the port's
   flush copies +0xC6=3 into +0xC5) — confirming the hunt block is the game's own behavior.

## Usage

not visible to me — this environment exposes no token, credit or context-window figures for the
session (no numbers were visible in any interface output to copy).
