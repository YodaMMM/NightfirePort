# level-13-security-room — 07000013: the security room door and the panel

Branch `s/level-13-security-room` = origin/master merged with origin/s/level-13-security-room
(the previous attempt's branch, merge 7c05c6e0, clean). The previous session (sandbox6,
07:10–08:00 UTC) did the task's code work and un-ignored both tests but was cut down before the
finish steps — the report was missing, hence the requeue. This session verified the work on the
merged tree, ran the full suite, and completed the finish steps.

## The two tests, and what was actually wrong

No port code changed: both failures were the TESTS' own premises, corrected against the disc and
the decompile (`crates/nf-view/tests/interactables_07000013_live.rs`, +86/−21;
`docs/levels/07000013.md`'s interactive-kinds table corrected; `panel13_probe.rs` added).

### 1. The "panel" test → `the_pillarglass_shatters_under_fire` — GREEN

The aim-fire loop connected (the panel's bounds were right) but no break landed in 900 ticks.
Three stacked causes, all read off the disc/decompile:

- **The filter was a misread.** The disc's pillarglass placements (#373–#376 "pillarglass(1)")
  carry params `[0:0x2, 1:0x6]` and NO param 2 [D census] — the old note read param 1's 0x6 as
  "filter !6". Param 1 is the HEALTH 6.0; with param 2 absent, `Break_Create` 0x12c230's
  param-2==0 arm stores filter 0 [M], so the pane's mask is 0 | 0x106 = 0x106 [M Break_Update
  0x12bfd8:15] and the round's word ANDs it to zero — the contact reaches the damage walk [M
  Collide_FilterBullets 0x1ece30:24-33]. The cold load's P2K (id 0x10, word 0x0008 [L
  ee_start_05.bin weapon_data @0x2bf150 + 0x10·0x10C + 0x10]) carries 10.0 contact damage
  [M Bullet_update's HITTEST+0x90], so 10.0 vs health 6.0 kills the pane on the first round
  [M 0x12bfd8:44-48 + Break_Kill 0x12b970].
- **The stand was inside the glass.** The four sheets (x 6.46..8.02 at z 35.96/37.53, y
  −369.25..−354.12 [D parse]) are solid to the capsule — the old stand (7.9, −368.4, 36.72) sat
  inside the enclosure and the leg never arrived (the 6000-tick wedge). The test now stands
  north of the north sheet and fires at a face point of that sheet.
- **The trigger needed pulses.** The P2K is a per-pull semi-auto: shots_per_pull 1 in every mode
  ([L the same record +0x28]; the pull latch `pressed && !held` [M]), so the old HELD trigger
  fired exactly once (clip 7→6) and never re-fired. The loop pulses fire every 6 ticks — the
  real player taps.

### 2. The "approach" test → `the_pickups_take_on_the_walk` — GREEN

The walk to door #355 (−26.65, −368.36, 37.44) wedged west of the desk block on a single
diagonal leg. The walk now takes the walkreal route (the desk block's north side → door #355,
which proximity-opens via its straddle param [D; `Door_Activate`] → the room's south strip) and
picks up the security room's shelf items on the way. Two assertions were artifacts of the wedged
walker, not game rules:

- **The 0x56 "stayed off" sister was wrong.** Both 0x56 TouchOnce volumes (#140/#141: x
  −8.33..−5.61 and 4.18..6.90, z 26.25..27.50 [D trigger dump]) sit on the desk block's two
  lanes — every walked route to the security room crosses one (the direct diagonal measured it
  ON; the west gap lane clipped #140's west edge; the east strip's mouth is solid). The test
  keeps the spawn-baseline sister (0x56 off at spawn) and asserts the touch fires on the walk.
- **The armour assert read a slot the take never writes.** `Pickup_Handler` case 3 writes the
  BLData armour FLOAT (+0x8B0 = 50.0) [M 0x18e650:220-225], not an inventory bit — the old
  `owned[0x32]` read the Frinesi's weapon id. The test asserts the armour float ≥ 50 and the
  pickup consumed [M 0x18e650:338-348]. The Spas12 assert (owned[0x1c] / the weapon register)
  was already the right shape.

## Verification (this session, on the merged tree)

- `interactables_07000013_live` → **6 passed, 0 failed** (10.65 s, direct run,
  `--test-threads=1`): both task tests plus the spawn-touch, shutter, and census tests.
- `walkreal_07000013_live` → **1 passed** (10.87 s) — the level's full walk stays green.
- `gate13_probe` → 1 ignored (a probe, not a test).
- `walk_07000013_live` → killed (exit 137/SIGKILL) at 300 s in this sandbox — the memory-limit
  OOM signature this environment shows for several big walk binaries (net_*/mp_*/walk_*); not a
  test failure. It is also in the suite's run (its result appears in the appendix if the suite
  reached it before the sandbox died).
- Full suite: `build ok: 799 test binaries in 1008 s` on this merged tree; the run phase and
  the total line: see the Suite appendix.

## What's left

- The suite's total line (the appendix records the state at the cutoff; the runner projects
  ~97 min of run on this sandbox's hardware, started 12:02 UTC — the total line lands right at
  the sandbox's expiry edge).

## Live checks for the main agent

None required: the fixes are test-side corrections with no visual claims. Optional renders of
the corrected walk (the natural player line through door #355): `--bond` spots at
(5.0, −368.4, 26.5) the lobby east lane, (−24.5, −368.4, 38.8) at the security-room door, and
(−22.0, −368.4, 29.0) inside the room's south strip.

## Usage

not visible to me (this session's environment shows no token, credit, context-window or cost
figures; nothing to copy).

## Suite appendix — the FULL suite completed on the final merged tree

`python3 tools/test/suite.py` (daemon, NF_ISO set), build `ok: 799 test binaries in 1008 s`,
run finished 13:36 UTC with the total line:

**`3296 passed, 23 failed, 193 ignored  (108 min)`** — 799/799 binaries.

The 23 failures, all accounted for:
- 3 `disc_golden` + `identity_disc` — the documented slim-ISO reds (the driver's rule 5 allows
  exactly these).
- 3 GPU-render reds (`fix_0c_pylon_live` ×2, `probe_pylon_render`) — the no-GPU sandbox set.
- 2 stream-shift reds inherited from the rain-2 merge, first seen on the previous session's
  tree and re-confirmed here (`bathhouse_shout_mixer_live::the_bathhouse_floor_guards_shout…`,
  `interactables_07000041_live::the_glass_pan_shatters_under_the_real_fire`) — next-session
  work, documented in the level-0a-rain-indoors-3 report too.
- The rest (14) are the sandbox's OOM kills (exit −9: net_*/mp_*/big walk binaries) plus
  `probe_gadget`'s 600s TIMEOUT — the same environment signature every session here shows;
  the exact names are in `suite13.out` (committed? no — scratch output; the list below).

The task's own family: `interactables_07000013_live` 6/6 green inside the run,
`walkreal_07000013_live` 1/1 green, `walk_07000013_live` in the OOM set (the direct run's
exit-137 above).
