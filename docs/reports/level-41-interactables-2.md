# level-41-interactables-2 — every player-touchable kind on 07000041 behaves like the game

Branch `s/level-41-interactables-2` (from origin/master 8f27b95e; master had not moved during the
session). Session window 02:43–04:22 UTC.

## What's done

1. **The census re-run [D].** `census_07000041.rs --ignored` re-dumped the level: **593 placements
   across the maps, 53 kind buckets; the main map m99 carries 120 placements / 35 kinds** — matching
   the audit doc (docs/levels/07000041-audit.md, which the same probe fed). The audit's headline
   re-verified: **no doors (0xdb), no spawners (0xf1), no drones (0x0f), no exits (0xe8), no
   switches/locks on the main map** — the wrapper's player-touchable kinds are: the 0xEA
   TouchOnce pair, 0xF4 MovieTrigger, 0xF0 pickup, 0x20 breakables, 0x22 ladders, 0x3E tether
   wires, 0x3F stealth walls, 0x3A grapple point, 0x3B DynoRod, 0xD2 turret, 0xDE searchlights,
   0xFA third icons, 0xF9 voice triggers, 0xFE laser mine, 0x71 rotors (+ the FX/draw kinds).
2. **The kind-coverage audit.** The 16 existing tests covered 14 kinds live and carried TWO
   ignore-listed gaps (the laser mine's walk-in contact; the stealth-wall crawl entry). Two
   player-touchable kinds had no live test at all: **the chain trigger #99 answered by the player's
   E** (the tests drove ch 1 with a `set_script` stand-in) and **the movie trigger #81**.
3. **Two new live tests** in `crates/nf-view/tests/interactables_07000041_live.rs` (now 18):
   - `the_chain_trigger_answers_the_e_until_the_plate_locks_it` — SL_Trigger #99 (20.59, 0.00,
     −33.44; out ch 1, mask 0x4 = the E/activate dispatch, lock ch 0x6e [D PL#99 — the census
     re-run #572]) fires ch 1 on the real E while its lock is clear [M 0x1b2318 case 1;
     `Trigger_Locked` 0x1b2fc8:24-29], and the E-fired chain runs to the level hand-off
     (level_exit 0x0700001b). The lock sister: on a fresh load the real walk through the plate
     #100 (out 0x6e, mask 8 [D PL#100]) arms the lock channel and the SAME E is refused — ch 1
     stays cold [M 0x1b2fc8's locked = lock≠0 ∧ ch on]. This grounds the stand-in the file's
     header documents: the chain channel has a real player path.
   - `the_movie_trigger_e_requests_no_movie_and_no_exit` — MovieTrigger #81 (16.13, −0.00, −0.41;
     mask 0x4, movie code 0 [D PL#81 — the census re-run #554]): the E raises the port's
     `movie_request = Some(0)` [M Trigger_Activate 0x1b2318:51-62 case 0xd — GameState+0x10 := the
     OUT slot, GameFlow states 6+7 pushed] and NO level load rides it [M the audit's consumer
     chain: state 6 pops immediately via `psiMovieFinished` 0x1082e8 and nothing on the arm calls
     `ResetMap_LevelToLoad`; `level_exit` stays None]. Sisters: no request and no exit at the load.
4. **Both ignore-listed gaps resolved:**
   - `the_laser_mine_detonates_on_contact` un-ignored: the beam is a 1.6-unit-tall 16-unit box over
     the airlock gap with no floor beneath [D the probe's bounds lo 44.4,−9.3,−49.3 … hi
     60.3,−7.7,−33.4] — the walked teleport fell out of the volume before the collide pass
     credited the contact. The re-spot holds Bond's body capsule inside the beam (the stand-in
     transit); the explode arm itself is the game's own per-tick contact over the mine cel's tris
     [M 0x17c930:31-45's hit-list walk; the body is `Player_Collision` 0x194df0's capsule].
   - `the_stealth_walls_are_inert_without_a_ctx6_icon` (was `…crawls_on_the_use_key`, ignored): the
     re-spot probe (both walls, both ends, both axial facings, the vent pattern's mouth) never
     enters the crawl — and the decompile says it can't: the use key's crawl scan is gated on the
     third-icon context 6 [M Player_Activate 0x1a5170:66-97], the context is written only by
     `ThirdIcon_Update` 0x196e80 from the level's 0xFA icons [M player.rs's `icon_ctx` doc], and
     07000041 carries no ctx-6 icon — its two icons are ctx 2 (gated on ch 1) and ctx 3 [D
     PL#105/#114; the census]. The test now pins that negative (the walls are set dressing on this
     wrapper; the crawl mechanic itself is pinned on 07000005/0d by fix_swim_vent_live). The
     TEST was not weakened — the old test asserted a positive the disc data cannot deliver; the
     assert now cites the gate that blocks it.
5. **No port gap surfaced.** Every player-touchable kind behaves on this wrapper; the two new tests
   and both re-spots passed against the port without a lib change (0 crate-source commits).

## How it was verified

- `interactables_07000041_live`: **18 passed, 0 failed, 0 ignored** (`cargo test --offline
  --release -p nf-view --test interactables_07000041_live -- --test-threads=1`; was 14 + 2 ignored).
- `walkreal_07000041_live`: **7 passed, 0 failed, 0 ignored**.
- `nf-game`: **1033 passed, 0 failed, 9 ignored** (100 binaries).
- **The fast suite** (`tools/test/suite.py`): the build completed — "build ok: 1073 test binaries" —
  but the full run exceeds the window (the first binary, `mp_stress_explosives_live`, runs ~10
  minutes alone). A prioritized run (nf-game, nf-formats, nf-audio complete + the touched
  nf-view binaries: interactables_07000041_live, walkreal_07000041_live, walk_07000041_live,
  interactables_07000041_probe, census_07000041, interactables_07000016_live,
  walkreal_07000016_live) — total line:
  **549 passed, 4 failed, 9 ignored (4 min)**. The 4 failures are the known slim-ISO image class:
  `identity_disc::the_real_disc_passes_the_identity_check`, `disc_golden::bigf_header_fields_behave_as_documented`,
  `disc_golden::bigf_tables_match_the_golden_manifest`, `disc_golden::directory_tree_matches_the_golden_manifest`
  (the level-14 session reported the same four on this ISO; the task allows the disc_golden class).
  No FAILED/TIMEOUT line belongs to this task's binaries.

## What's left

- The full 1073-binary suite run (the window cannot fit the ~10-minute `mp_stress_explosives_live`
  plus the rest; the prioritized total above stands in).
- The doc §6 stale rows the audit already lists (the 0x71 "spawners" mislabel, the #82-#85 icon
  claim, the script id list) — the audit doc supersedes them; I did not touch the level doc.
- The #99-vs-the-game live check below (whether retail's scripted drone or the player's E fires the
  chain first in the real play-through).

## Live checks for the main agent

1. **The chain trigger's E reach**: at the airlock deck, render `--bond` frames from (20.59, 1.4,
   −32.0) facing south at #99's volume — should show the SL_Trigger's pick target and, on the E,
   the approach cutscene chain starting (ch 1 rises; watch `switch_channels[1]`).
2. **The lock beat**: after crossing plate #100 (watch `switch_channels[110]` 0x6e rise), confirm
   the E on #99 is refused in the game (the locked trigger's path [M 0x1b2fc8]) — the port pins it.
3. **The mine beam's contact face**: in the airlock gap, confirm the game's mine explodes when the
   tether-crossing path carries Bond through the beam's box (the port needed a held-capsule stand-in;
   the game's own crossing should contact naturally).

## Usage

not visible to me
