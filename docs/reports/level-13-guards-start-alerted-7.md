# level-13-guards-start-alerted-7 — the vision-bit census: the escort flee and Rook's death channel

Branch `s/level-13-guards-start-alerted-7` = `origin/s/level-13-guards-start-alerted-6` (011027d0)
merged with `origin/master` (17aa36ab — carries the un-ignored 07000014 delta-exit walk), plus the
two port fixes below (2208737e). Both merged-tree reds' port sides are fixed; one walk leg remains.

## Red 1 — escort_follow_07000001_live: "the escort left the ally machine: state 0x86"

**Root cause (port), two pieces, both [M]:**

1. The ally-lead states' case-0x21 (explosive-nearby) resumes were missing: the port's
   msg::EXPLOSIVE fallback handed ALLY_LEAD resume ATTACK, so a survived grenade returned the escort
   into combat instead of the ally machine. The game's ally states call
   `DroneFunc_ConsiderExplosive(dcv, obj, own-state)` [M 0x15e698:131, 0x15eda8:90, 0x15ec08:64,
   0x15efc8:56, 0x15ea28:86, 0x15f120:71, 0x15f2e8:38] — ALLY_LEAD_INIT passes 0 [M 0x15e4f0:64] and
   the ally cover run 0x28 [M 0x15f938:61]. The trigger itself is game-true:
   `DroneFunc_HandleExplosives` 0x1435f0:30-51 runs for allies (ControlSTANDARD 0x1490f0:246), sends
   msg 0x21, and `ConsiderExplosive` sets state 0x86 [M 0x1431c0:194-195]. The -6 corpse fix shifted
   the shared-RNG stream so the guard-15 fight's grenade now lands inside the escort's scan — as the
   -6 report predicted.
2. `RECOVER_FROM_SCARY_OBJECT`'s entry hardcoded resume ATTACK. The game's
   `NDrone2_DSTATE_RecoverFromScaryObject` 0x173658 case 1: the mission-lead MODE byte 0x0C re-anchors
   the mission route (`NDrone2_ReFindMissionPath` 0x153a30) and returns to ALLY_LEAD_WAIT 0x20 —
   GO_TO_GOAL 0x63 when the re-anchor fails; other modes clear the scare latch and resume 0x56. The
   port now implements both arms.

**Verified:** escort_follow_07000001_live `escort_at_walking_speed_keeps_the_games_distance_and_the_exit_opens`
**passes** ("the contact settled at (−28.2,6.3,14.3) after 2085 distance checks"). Before the fix the
flee ran 8800 ticks into CIVILIAN_HIDING (0x13); after, the flee cycles back into the ally machine
within ~700 ticks per grenade. The test now allows the flee chain as the game-true transient with the
cites, pins the resume via a 600-tick grace window for the recovery-entered Wait (the recovery's
SetState(0x20) carries no band [M 0x173658 case 1]; the −7/0 band [M 0x15eda8 case 3] pins the
ALLY_LEAD-case-3 door), and keeps every decompile-backed assert.

## Red 2 — walkreal_07000014_live: "Rook's death channel 0x54" (:858)

**What actually happens [L + M]:** with the sight bit (the -6 fold of +0x4f8 & 0x10000 [M
0x15e698:21 et al.]), Rook leaves his wake into the sight chain and dies out of the combat chain
(pre-kill state 0x5c COMBAT_NO_SIGHT [L the instrumented run]); his bullet death lands in
DeathByExplosion (0x45) — the alert-status selector's else arm [M 0x1465e4-0x14697c, the port's
selector doc] — whose channel drain runs on the death state's own ticks: the channel is TRUE one tick
after the kill [L the +0 poll: `ch54 true, pend true, dch 0x54`]. The test read the channel on the
SAME walker tick the kill completed — not the game's delivery contract: `DroneFunc_SetDeathChannel`
0x13e660 fires from the death DSTATEs' case 2/3/0xc [M 0x163a80:44 (case 2), :59 (case 0xc), the
case-3 drain], at least one tick after the ENTER that lands the corpse.

**Fix:** the test's ch54 read now polls up to 30 ticks with that cite; the assertion itself is
unchanged (the channel MUST fire — and does, with objective 4 completing [M 0x1827d8]).

**Verified:** the kill + ch54 + objective-4 asserts all pass; `walkreal_07000014_live` is now 4/5
(the other four tests in the binary green).

## Remaining red (accounted)

`walk_the_infiltration_to_the_delta_exit` still fails at **leg 6** — "to the Delta exit #944": Bond
wedged at (23.31,1.14,−24.64), target (27.79,1.28,−25.32), d 4.53, `segment_to_target=Some(0.1227)`
cel 16, grounded=false. A bots-near dump found **no bot within 3.0** of the blocker — it is world
geometry in the delta-exit approach that this tree's RNG shift now steers Bond into (leg 6 never ran
on the pre-fix tree; master walks it green). Not diagnosed further in this window; next round owns it.

## What ran

- `cargo build --release -p nf-game` clean after each change.
- Direct runs (cargo test --release --test-threads=1, nf-view):
  escort_follow_07000001_live **1 passed** (escort_at_walking_speed);
  walkreal_07000014_live **4 passed / 1 FAILED** (leg 6, above).
- The full suite via the fast runner on the final tree: background jobs were killed between tool
  calls twice; the surviving run's totals are quoted in the suite section appended below if it
  reached a total line in the window (the runner's `suite.out` tail is the source).

## What's left

1. Leg 6's world-geometry wedge (the delta-exit approach) — diagnose the blocker at cel 16 and
   re-derive the walk or fix the port.
2. The Double Cross Part 1 / Countdown Part 1 finishability check rides the full suite (their
   binaries are in nf-view; the runner covers them).
3. A full-suite run reaching nf-view on the final tree (this window's runner gets through the
   earlier crates at least).

## Live checks for the main agent

1. **Rook's post-wake states** — gdbwatch Rook (skin 0x05000066, 07000014) for ~30 s after ch 0x0D
   from a FRESH boot with Bond armed and near: record +0x58 (state) and +0x228 (the sight flags).
   Expected per the sight chain: the ally-machine states drop msg 0xf (no case 0xf arm [M the four
   ally DSTATEs]) — if the game's Rook enters the combat chain the same way the port does, 0x5c
   COMBAT_NO_SIGHT is game-true; if he stays in the ally machine, the port's sight delivery is
   over-reaching and the kill setup needs the ally-true form.
2. **The delta-exit approach at (23.9,1.2,−24.7)** — a render at `--bond 23.3,1.3,-24.6` facing east
   (yaw ≈ 1.57 rad in the port's atan2(x,z) convention): what stands between that spot and the exit
   #944 at (27.79,1.28,−25.32)? The port's segment hits something 0.55 m ahead at cel 16.

## Usage

not visible to me
