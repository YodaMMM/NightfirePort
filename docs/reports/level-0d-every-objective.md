# level-0d-every-objective — the 0700000d mission walked end to end on real input

Branch `s/level-0d-every-objective` (from origin/master 284c1676). One file changed:
`crates/nf-view/tests/walkreal_0700000d_live.rs`. No src changes were needed — the port's
simulation already covers every objective; the test just stopped before them.

## What the mission data lists ([D]/[L]/[M])

Record 12 of the mission data ([L] ee_start_05.bin @0x2a4530, level 0x0700000d) lists three
objectives; record 11 (0700000c) carries the fourth this level completes
(docs/levels/0700000d.md §2, pinned green by `census_0700000d::mission_record_12_matches_the_snapshot`):

| # | Text | Complete ch | Writer |
|---|---|---|---|
| 3 (rec 11) | "Advance through Warehouse TR-2" | 0x23 | the start trigger #436 [D] |
| 4 | "Investigate building ST-3" | 0x1d | the TouchOnce plate #378 [D] |
| 5 | "Photograph the Laser Prototype" | 0x1e | the CamSubjects #494/#495 [D; M CamSubject_Update 0x1bc668:14-24] |
| 6 | "Destroy door C5 with mounted laser on catwalk" | 0x1f | the Destroy leaves #93/#96 bigdoor01/02 [D; M Destroy_CollisionHandler 0x12bd48, Destroy_Update 0x12c4f8:23] |

Mission end: the AND mux #434 [0x1e & 0x1f & 0x28] → ch 100 [D] → `Mission_Update` state 6
ends the level to **0x730000d** [M 0x182c60:102-103].

## What the existing tests already covered

- objective 3: the walkreal's start block (ch 0x23 under Bond, states[11][2] = 4).
- objective 4: the walkreal's yard walk to plate #378 (ch 0x1d).
- objective 5: the walkreal's camera leg into CamSubject #494 (ch 0x1e).
- objective 6: NOT covered — the walkreal stopped at the C5 door with the (by then obsolete)
  "documented stop": the mounted-laser cut-scene unported, "nothing else writes ch 0x1f".

## The fix: the walkreal now walks the WHOLE mission

The stop's premise was already corrected by the level census
(docs/levels/0700000d.md §7, 2026-10-17/2026-10-22): **ch 0x1f is reachable by shooting** —
`Destroy_CollisionHandler` 0x12bd48 subtracts EVERY class-5 bullet's damage (the 0x67 check
only dims the lights, :31-33) and the death writes switch_channels[ch] [M :35]. The disc's own
hint "your only way out is through that security door marked C5 … considerable firepower"
[D USATxt 0x4000066] says the shooting route is the game's too. The interactables test
`c5_shot_open_completes_the_level` already proved the chain; the walkreal now does it as the
walk's own legs:

1. **The shooting stand** (real walk): yard north to (135.0, 4.5, −281.0), 11+ m off the
   bigdoor02 #96 yard face (bounds 131.78..137.08, 3.99..7.35, −269.85..−269.15 [D placed
   gfx]) — outside the death blast's reach (blast damage 1000 [D p5]; `top_up_health` guards
   Bond anyway).
2. **The weapon**: the level's own mounted laser 0x67 (GunImp #225 [D]; the mounted class
   sustains fire with no ammo pool [L probe: clip 0/spare 0 keeps firing, 50 damage a hit]).
   Selected through the walkreal's established gadget-select idiom (`switch_request`, the same
   mechanism the camera leg uses) [S port choice, not a game rule].
3. **The kill** (real trigger path, `shoot_at`'s aim loop): ~200 hits on health 10000 [D]
   until ch 0x1f; the leaf's `dead` asserted; objective 6 completes one tick later
   (states[12][2] = 4 [M 0x1827d8:88-99], the monitor tick the fix-objective-banner work
   pinned).
4. **Through the dead door into the corridor** (real walk): the re-firing Touch plate #433's
   low stretch raises ch 0x28 while the crossing continues [D; M triggers.rs — its exit clears
   it, which is why the crossing now comes AFTER the kill; the old test crossed before the
   door and over the closed leaf's top, an artifact the new order removes].
5. **The mission end**: the AND #434 [0x1e & 0x1f & 0x28] raises ch 100 [D list; M
   Trigger_Update's AND arm]; `Mission_Update`'s state-6 timeout (240 ticks [M 0x182c60:88-91])
   runs; `mission.status() == 6` and the exit is **0x0730_000d** [D record; M 0x182c60:102-103]
   — the level exit firing.
6. **Every objective asserted complete**: states[11][2] (3), states[12][0] (4),
   states[12][1] (5), states[12][2] (6) — all state 4 [M mission.rs states].

Test result: `walkreal_0700000d_live::walk_arrival_to_the_c5_door ... ok` in 5.14 s.

## The remaining stand-in (unchanged, documented)

The crane sequence into the ST-3 yard is still a documented stand-in (the old test's one
teleport at the dock's south edge): the game's route is touchplate #460 → ch 0x2b → the
`shipping_handle` SS #112 + script player #115 (script 0x060009EF) carrying Bond across, then
the crane-rotate chain (#450 → ch 0xa → SP #492). The pieces move now
(`interactables_0700000d_live::crane_e_plate_arms_the_handle_ss_and_the_shipping_sp`,
`crane_rotate_e_plate_starts_sp492`, `crane_ladder_grab_and_climb` all green), but the full
ride into the yard on real input is still open work — a follow-up task, not a completion
blocker for the objectives (the walk enters the yard at the documented stand-in and walks
everything after on real input).

## How it was verified (what ran)

This sandbox kills background jobs between tool calls (the third sandbox to measure it; the
885-binary suite + its ~10 min build cannot fit the 2 h window even with
`tools/test/chunk_suite.py` accumulating). What ran, all green, `--test-threads=1`, NF_ISO
set:

- `walkreal_0700000d_live` 1 passed / 0 failed (the extended walk).
- The whole 0700000d family: `interactables_0700000d_live` 11/0 (incl.
  `c5_shot_open_completes_the_level`), `walk_0700000d_live` 3/0, `patrol_walk_0c_0d_live` 2/0,
  `census_0700000d` 2/0, `audit_0700000d` 2/0, the 0d gunimp family 6/0.
- A `nf-view` crate sweep (timeout-cut at 9.3 min, alphabetical): **179 passed / 0 failed**
  across 38 binaries (through `campaign_flow_live`).
- `chunk_suite.py` cumulative: 14 binaries recorded, **258 passed / 0 failed** (nf_view's lib
  96/0 among them). 871 binaries remain unrecorded in this window.

No src files changed, so no other crate's behaviour could move; the chunk's nf-audio /
nf-formats / nf-game bins that ran are green. The suite's total line could not be produced in
this window — the next window should run `tools/test/chunk_suite.py` (state file pattern in
docs/tools/) until ALL DONE.

## What's left

- The crane-ride leg on real input (replacing the last stand-in) — needs the SP #115 ride
  carry + the yard drop walkable; the SS/SP/wire/ladder pieces exist.
- The full 885-binary suite total line in a window that fits it.
- `bigdoor01` #93's hittable volume (the disc gives it no gfx bounds [D] — only #96 parses),
  so the test kills #96; the game's cut-scene kills both via the mounted laser.

## Usage

not visible to me
