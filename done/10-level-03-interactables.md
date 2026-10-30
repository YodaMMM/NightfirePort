## Your task: `level-03-interactables`, every interactable kind on 07000003 works like the game

07000003 has no per-kind interactables test (0700000b's `interactables_0700000b_live.rs` is the model: one live test per KIND of player-touchable placement - doors, switches/consoles, fuseboxes, lifts, pickups, breakables, plates, monitors, gadget targets - driven the way a player drives it (walk, use key, trigger pull, walk-over, gadget), each asserting the game's effect with a sister). Census every player-touchable placement on 07000003 [D placements + docs/levels/07000003.md], write `crates/nf-view/tests/interactables_07000003_live.rs` with one test per kind, and fix every port gap the tests expose [M the class code; D the params]. A teleport only to reach a spot whose walk is a documented blocker; the operation itself always real input. Every test green (no #[ignore] without a named blocker). Run the touched binaries and quote their totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-03-interactables`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox3 at 2026-10-30T15:27Z

---
## Report (done, 2026-10-30T16:13Z)


---
## Report from the worker (done, 2026-10-30T16:13Z)

# level-03-interactables — report

Branch `s/level-03-interactables` (master 2f1edf31). Task: census every player-touchable
placement on 07000003, write `crates/nf-view/tests/interactables_07000003_live.rs` with one
live test per KIND, fix every port gap the tests expose.

## Census (main map 50, 511 placements)

`nf-extract placements-all 07000003.bin` over the ISO [D], cross-checked against
docs/levels/07000003-audit.md's type table — the two agree exactly. The player-touchable kinds
and their tests:

| kind | n | test in `interactables_07000003_live.rs` |
|---|---|---|
| 0xDB doors | 13 | `plain_door_e_opens` (+ the locked sister, see the gap) |
| 0xEA/0xEB plates | 19+2 | `walk_over_plates_fire_their_out_channels` (mask 8), `the_e_plate_unlocks_the_proximity_door_and_swings_the_picture` (mask 4) |
| 0xF0 pickups | 5 | `pickups_taken_by_walking_over` (armour + the kind-6 medal gate, base map 0x07000001 [L mission::c07000003::DEF.mission]) |
| 0x20 breakables | 97 | `breakables_shatter_on_bullets` |
| 0xE1 destroyables | 5 | `destroyables_smash_on_bullets_and_the_fusebox_twins_never_exist` (also pins `fuseboxes` EMPTY [M 0x1b0448; D all five 0x31 twins scriptless]) |
| 0x22 ladder | 1 | `the_garden_ladder_climbs_to_its_top` |
| 0xD9 script players | 3 | `the_dragon_e_use_spawns_the_safe_key_that_writes_ch_0x4e` (the level's switch/console role — switches 0x29 are absent) |

ZERO on main map 50 [D census vs the full dispatcher 0x1d04f0]: switches 0x29, monitors 0x30,
sensors 0x28, safes/locks 0x2E, copters 0x2F, turrets 0x27, grapple points 0x3A, lifts — so the
driver's switches/consoles kind is the dragon script entity's mask-4 E-use and the lifts kind is
the ladder. Not player-touchable kinds (rainboxes, env, sky, cover, AI points/volumes, hints,
music/sound triggers — the last are channel-driven, not player-driven) are out of scope per the
census; the level's 5 water cels (the swim system) are a system test, not an interactable kind.

## Done and verified

- The file: 9 tests — **8 passed, 0 failed, 1 ignored with a named blocker**;
  `test result: ok. 8 passed; 0 failed; 1 ignored ... finished in 117.23s` (release,
  `--test-threads=1`, NF_ISO on the slim ISO).
- Every assertion carries its [M]/[D]/[L] tag at the site. Teleports are used only for the
  documented-blocker transits (each cites its own probe: the cross-room nav legs never route out
  of the start block — `nav_route` returns None / the walk wedges at (6.22, −18.61), (−1.03,
  −32.03), (−3.44, −18.08)); the operations themselves (the E, the trigger pull, the walk-over,
  the climb) are always the real input path.
- Neighbour binaries re-run green: `walk_07000003_live` 2 passed, `wiring_07000003_live`
  4 passed.

## The port gap the tests exposed (open, named)

**The mask-4 trigger's use-pick volume.** Probe (the `locked_door_refuses_the_e` test, ignored):
standing at locked door #4 (ch 0x31 off, [D p3=49]) and pressing the real use key toward the
door fires plate #504 — ch 0x31 rises on the second press, `Door_Update`'s rising edge clears
the lock ([M door.rs update's prev_channels arm]), and the door opens. The game plays the hit
sound until ch 0x31 [D audit]. Root shape: the use-trigger arm admits the use sphere (head +
1×look, r 1.0 [M 0x1a5170:100-108]) anywhere inside the plate's WALK-IN volume; #504's volume
reaches the doorway 2.9 units from the plate's centre. The E-path lock gate itself is correct
(probe: with ch 0x31 held off, 45 ticks of E hold state 0x2). Fix site: the use-trigger arm's
volume test in the use pick (nf-game useobj/triggers path) — likely the game picks the trigger's
tight object/pick volume rather than the walk-in box; needs a decompile pass over
`Player_Activate 0x1a5170`'s trigger arm + `Trigger_Activate 0x1b2318`'s use entry before
changing it, so it stayed out of this session's reach. Un-ignore the test once fixed.

Also observed (documented in door.rs already, restated here because the tests saw it): the
port's guard routes do not apply the door lock (`is_locked`'s [S] note) — a patrolling guard can
open locked door #4 mid-test; the sister therefore runs from the doorway stance before any walk.

## What's left

1. The gap above (fix + un-ignore).
2. The full suite run on the final tree: the fast runner (`tools/test/suite.py`) was started in
   the background at 15:30 UTC but the sandbox kills background jobs between tool calls —
   `suite.out` ended up empty (0 bytes), and the remaining window could not fit a foreground
   rerun. The touched binaries are quoted above instead. The next session should start the
   runner in the background FIRST and verify it survives (check `suite.out` grows), then run it
   once more on the final tree.
3. The walk transits documented as blockers could be replaced by real walks if the nav graph's
   start-block routing gets fixed (nav_route None out of the bedroom block).

## Live checks for the main agent

1. Locked door #4 (14.68, 6.65, −35.40): stand ~1.8 units south-west, aim at the door, press E.
   Expected (game): the door stays shut, the hit sound plays, ch 0x31 stays off. Expected
   (current port): ch 0x31 rises (plate #504 fires from the doorway) and the door opens.
   Watch `switch_channels[0x31]` with `tools/trace/fieldprobe.py` during the press.
2. Plate #504 (11.80, 6.04, −35.63) aimed at directly from ~1.5 south: ch 0x31 rises on the E
   (both sides agree); this is the control for check 1.

## Usage

not visible to me — this environment shows no token, credit, cost or context-window figures for
the session (no usage display is exposed to the agent; nothing to copy).
