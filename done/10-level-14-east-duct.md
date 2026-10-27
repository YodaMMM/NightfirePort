## Your task: `level-14-east-duct`, 07000014: the east duct mouth, then the infiltration walk

`walkreal_07000014_live::the_east_duct_route_reaches_the_stairwell_slot` and `walk_the_infiltration_to_the_delta_exit` are ignored. The real-input east crawl wedges at the duct mouth: both lanes (z 63.7 and z 65.4) stop at x 40.98 with head_bump true, crouched and grounded — the mouth's lintel sits at crawl-head height in the port's world (read the two ignore reasons in full).

1. Dump the triangles of x 40.9..41.6, y 3.1..3.9, z 63.0..66.0 and name their owner objects and glist flags. Is the lintel structural wall, or a flagged label/lid object the game's player collision skips [M `Collide_Pick` 0x1eb320, `Player_Collision` 0x194df0 and its crouch capsule]? Compare the crouched capsule's head height with the game's [M].
2. Fix the cause [M] (not a test-side teleport), pin it with a test that fails before the fix, and un-ignore the east-duct leg; then take the walk on to the slot crossing and the exit as far as it goes, un-ignoring what goes green.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-14-east-duct`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox1 at 2026-10-24T17:06Z

---
## Report (done, 2026-10-24T18:08Z)

Closed by the driver: the report is on `s/level-14-east-duct` (docs/reports/level-14-east-duct.md); the worker didn't run finish.sh.

---
## Report from the worker (done, 2026-10-24T18:58Z)


---
## Report from the worker (done, 2026-10-24T18:58Z)

# level-14-east-duct — the 07000014 east duct mouth, then the infiltration walk

Branch `s/level-14-east-duct`. The two ignored `walkreal_07000014_live` tests
(`the_east_duct_route_reaches_the_stairwell_slot`, `walk_the_infiltration_to_the_delta_exit`)
wedge at the "duct mouth". This task answered the mouth question and rewrote the route; the
real-input traverse is now blocked one step further on, at the sleeper-drone ambush.

## What's done

**1. The tri dump and owner attribution (task step 1) — the "lintel" reading was wrong.**

`crates/nf-view/tests/probe14_mouth4.rs` (new, assertion-free, run with `--ignored`) dumps the
mouth region. Findings, all [D] (level data) unless tagged:

- The blocker at both crawl lanes (z 63.7 and z 65.45) is **plc #16 "AirConditioningRoom"**
  (kind 0xc047, **glist 0x0**): its east wall face at x≈41.534 runs **z 62.2..67.2 at full crawl
  height** (two skins: tris 7418/7419/7422/7424/7599-7602/7713-7727; probe14_mouth4 part E lists
  every x≈41.534 face tri sorted by z — there is no hole at crawl height anywhere in z 62.2..67.2).
  The mini-world attribution (probe14_mouth3) reproduces the sim push
  ([−0.216, ~0, 0.0097] at (41.2, y 3.15/2.95, z 65.45)) from the room shell alone.
- So the answer to the task's question: **the wall is structural, and the game keeps it** — no
  label/lid skip applies. [M] the placement carries 0x8000 so
  `parsemap_block_map_data_static` 0x1d1998:92 builds the runtime cel for the whole object and
  `Collide_Pick` 0x1eb320 drops a node only on glist bit 0x20 (:140) or bit 0x1000 with the
  query's bit 0x20 (:145-151) — this cel's word is 0x0. The kind word (cel+0x3c, 0xc047) only
  selects the overlap-test shape at 0x1eb4e4 (bit 0x8000000); nothing in the pick path filters on it.
- The bit-0x10 objects in the region (`AirConBlockingPoly`/`2`, plc #258/#261, glist 0x10) are
  roof slivers at y ≥ 3.78 — above the crawl head — and the game collides bit 0x10 anyway (only
  `View_AddCels` 0x1e6ba0 skips it, the render side).
- The crouched capsule head height [M `Player_Collision` 0x194df0 case 3]: A =
  pos + (0.1 − (cc − 0.55)), r 0.55 → top = pos + 0.65 (cc 0.55; the port's `pick_arm`/`capsule`
  in crates/nf-game/src/player.rs implement exactly this). The game's one live rest measurement
  [L, recorded in player.rs's doc: the truck bed] parks crouched Bond at pos−floor 0.607 with
  cc 0.556 → head top ≈ floor + 1.251 vs the port's floor + 1.30. That ~5 cm difference is real
  but **was never the wedge cause**: the wall spans y 2.40..3.86 — no capsule of either height
  passes. Nothing to fix in the capsule.

**2. The real route (nav-web proof) and the rewritten tests.**

The level's own walkability graph [D block 0x05, map 54] threads a corridor at nav y 0.4 running
east–west at z≈62.9 from x 37.4 to x 49.1 (path 2 nodes 4-8) and then **south along x≈49**
(node 3 (49.08, 0.4, 52.9) → (49.1, 0.4, 42.8)) — i.e. under the vent room and down to the
flat-grate vent mouths at z=42.16 [D #655/#600]. The room floor ends at z≈63.64 past lid SP #266
(40.80, 2.37, 63.66 [D]); the drop into that corridor is the game's way out — not the east wall.
probe14_mouth4's scoped-vs-flat run pins the drop slot: floor **2.40 at z ≤ 62.2, −0.60 at
z ≥ 62.4** — a trench between the room lip and the south duct floor.

- `the_east_duct_route_reaches_the_stairwell_slot` is REWRITTEN onto this route (crawl south over
  the lip → drop → corridor east to x≈49 → south toward the mouths). The real-input drop WORKS
  (Bond crosses the lip and lands in the slot). Both 14 walk tests' ignore reasons now carry the
  full findings so the next session continues from the branch.
- `the_vent_grill_blocks_the_west_duct_per_the_game` stays green (ran; the grill plane z=58.86 is
  the game's own closed geometry — unchanged from level-14-vent-grate).

**3. The remaining blocker, named precisely: the sleeper-drone ambush.**

The drop slot (x 40.07..41.53, z 62.2..63.64, floor −0.6) is one capsule wide and the sleeper
drone #270 (skin 0x05000090 [D], (40.83, 1.08, 62.85)) patrols inside it. Bond lands
dead-centre on him (rest y 3.05, 2.38 below) and the sim recentres him on the carrier every tick
(x frozen at 40.9837 across plain-steer and `climb_stairs` jump legs). The starting gun's 12
rounds all miss from drone-back (health 10 untouched); the loadout's other guns are empty (clip 0
spare 0 ×5 via the shoot helper's own weapon cycle). This is where the leg stops.

## How it was verified

- Real-input runs through the walker (60 Hz, the game's input path): the lip crossing and drop
  reproduce every run; the wedge x=40.98 reproduces at the old mouth lanes (the wall face).
- probe14_mouth3/mouth4 tri dumps [D]; the nav-web dump [D block 0x05]; the decompile reads [M]
  0x1d1998, 0x1eb320, 0x194df0 quoted with line numbers.
- `walkreal_07000014_live::the_vent_grill_blocks_the_west_duct_per_the_game` green.

## What's left (next session)

1. **The sleeper push rule [L]:** a fresh-boot PCSX2 look at the game's drop-and-sleeper moment —
   does the game's drone-vs-player push park Bond *beside* the sleeper (horizontal resolve), or
   can the starting gun kill from on top? The port's push recentres Bond on the carrier; if the
   game resolves horizontally, that's the [M]-citable fix in the sim's actor-actor push.
2. Then the corridor east (x 41→49 at z≈62.9) and the x≈49 south run to the mouths z=42.16 and
   the mezzanine — un-ignoring `the_east_duct_route_reaches_the_stairwell_slot` and, with leg 1a
   complete, `walk_the_infiltration_to_the_delta_exit` (its legs 2+ are unchanged).
3. The stairwell **slot crossing** (the other documented blocker: the threshold face z=55.719
   [D probe14_chute] and the missing under-opening floor, probe14_landing y → −210) needs the
   same fresh-boot treatment — the teleport scans' tube/slot descent is likely not the player's
   route at all now (the grapple points #626/#657 at (45.30/40.78, 4.79, 42.86) sit at the vent
   mouths' level — the game's entry to the mezzanine vents), so the slot may fall out of the
   route entirely.

## Live checks for the main agent

1. FRESH boot into 07000014, walk Bond east along the vent room to the south lip and drop:
   record `obj+0x30` and the sleeper drone's `pos` at the landing tick (does Bond land beside or
   on top?), and whether the drone then moves under him.
2. Same boot, from on top of the sleeper: fire the starting gun straight down — does the drone
   take damage (its health word), and does Bond slide off after its death?
3. If a debugger watch is cheap: `gdbwatch.py` on the port-side analogue — the push direction
   the game applies to Bond's `obj+0x30` during the drone overlap (x/y components per tick).

## Run log (what actually ran)

- `cargo test --offline --release -p nf-view --test probe14_mouth3`, `--test probe14_mouth4`
  (all probes, `--ignored`), `--test walkreal_07000014_live` (`the_vent_grill_blocks...` green;
  the east-route test run 10× while iterating the legs).
- The fast runner suite (`tools/test/suite.py`) on the merged tree: 782 binaries built ok
  (1004 s). The run processes ~12 binaries/min and was still grinding at report time; the big
  level-loading tests (mp_stress_explosives_live, walk_07000013_live, models_weapons_live,
  walkreal_07000016_live, interact_0700000c_live, fix_death_restart_live, net_join_live, ...)
  were SIGKILLed by the sandbox (exit −9) even with the foreground quiet — this session's memory
  cap is tighter than the previous session's clean run of the same suite. The 3 `disc_golden`
  failures are the documented slim-ISO ones; `identity_disc::the_real_disc_passes_the_identity_check`
  also failed (not covered by the slim-ISO waiver — worth a master-side look). The suite's own
  total line did not print before this session's deadline; summing the per-binary result lines
  in its log at the deadline (18:58 UTC): **2701 passed, 6 failed, 235 ignored** across ~810
  completed binary sections — the 6 being the 3 disc_golden, identity_disc, and the two
  headless-render pylon tests (fix_0c_pylon_live, the same pair the previous session's suite log
  shows failing) — plus ~15 bins SIGKILLed by the sandbox (exit −9) and a handful of tail bins
  still running at the deadline.
- The touched crate's own tests (nf-view: walkreal_07000014, probe14_mouth3/mouth4) ran
  green/ignored as expected.

## Usage

not visible to me (no token, credit or context-window figures are shown in this environment).
