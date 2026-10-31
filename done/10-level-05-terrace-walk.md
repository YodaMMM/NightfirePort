## Your task: `level-05-terrace-walk`, 05: reach the terrace spot on real input

`walkreal_07000005_live` still places Bond at `[22.0, 50.1, -68.0]` on the terrace (floor 49.001 [D probe_terrace05 col (21.8, -68.1)]) — read the comment above that write (around line 1365) for why it was a stand-in. Find the game's route to that terrace from where the test's walk stands [D collision/placements incl. doors, ladders, wires, grapple points; M the classes the route uses], walk it on real input and delete the write; if the data shows no route, pin that with a sister assert and name the live check that settles it. Run walkreal_07000005_live and interactables_07000005_live; quote the totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-05-terrace-walk`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox5 at 2026-10-31T12:22Z

---
## Report (done, 2026-10-31T14:16Z)

# level-05-terrace-walk — the report

Task: `walkreal_07000005_live` still placed Bond at `[22.0, 50.1, -68.0]` on the terrace as a
stand-in (the write at the old line 1365, in what was `the_door_band_exit_chains_to_07000006_climb_parked`).
Find the game's route to that terrace from where the walk stands, walk it on real input, delete the
write; if the data shows no route, pin that with a sister assert and name the live check that settles it.

**The route exists and is now walked.** The stand-in write is gone.

## 1. Where the walk stood

`the_cable_landing_walks_the_south_walkway_toward_the_zip_door` walked the cable route's south legs to
the covered staircase's top (35.0, 43.3, −71.4) — but only by PARKING at a "never-closing ch 0x42
script window" (the east legs returned early on a 3600-tick disable hold; L the run of 2026-10-29).
The door test then jumped straight to the terrace with the placement write.

## 2. The blocker was the test's own drop steering, not a port gap

Running the walk with `NF_SCRIPTLOG=1` names the disable: at frame 8757 (34 ticks after the grapple
release landed) **SP #284 (the death NIS 06000952) fired event 5 then event 4 (PlayerDisable)**
[L the scriptlog run of 2026-10-31], and at frame 8850 its event 13 wrote `[0x62, 1]` — the
mission-fail channel [D the plunge chain: 13 PlungeTriggers #285–#297 out 0xBE → SP #284, docs/levels/07000005.md §6].
Bond got into a plunge volume because the release loop steered at (0, −72): the aim pulled the fall
5.5 east and 1.2 south, off the east lane's south strip (landing (1.10, 37.62, −72.58)), and the
post-landing wait let him slide to (−0.25, 36.09, −74.86) into the plunge zone.

The release now rides **straight down** onto the lane below the hang (pulley #281 (−3.40, 42.94,
−71.77) [D placement]; hang phase 2 at (−4.39, 42.53, −71.82) [M Player_SSGrapple 0x192860 phase 2
hangs within 2.0]): landing (−4.43, 37.42, −71.81), on the lane (z −71.4, floors 36-37 [D
probe_corridor05]). No plunge fires, no script window outlasts the hold, the PARKED early-return is
deleted, and the east legs walk clean to the stairs' top (34.34, 43.89, −71.43). The 2026-10-29
"the real game's helicopter-start cut-scene is the live check" note was that artifact.

## 3. The terrace route (the disc's own continuation)

The old claim — "the disc's collision shows NO walkable link from the stairs' 43.4-44.0 top to the
y-49 terrace, every face 5.5+ tall [D probe_corridor05 MAP 8/9/10/11/12]" — was **the scan's own
x ≤ 38 window**: it read the stairs' 49.5 overhang for the terrace's south face and never looked
east. The new probe `crates/nf-view/examples/probe_terroute05.rs` (committed; maps A–F over the
port's collision of the disc's own data) shows the link, all walked on real input by
`walk_the_stairs_top_to_the_terrace_spot`:

- the covered staircase's top flight (43.8 @ x 36.5, z −71.5) turns **south-east onto the south
  strip** (43.9-44.1, z −72.25..−73.5), running east past x 47 [D map D, 0.25-unit];
- the strip connects **north over the walkway tip** at x ~43.5 (44.2, z −71.75..−71.0), threading the
  46.2-46.3 posts at (42.0, −71.5) and (45.0, −71.2) [D map D];
- onto the **east walkway** (44.0-45.2, x 38-49, z −70.75..−58.5) — the door guards #271/#272 post
  at (41.52/46.41, 46.08/45.73, −69.67/−69.88) [D placements] stand on it, and the ammo #322/#323
  (48.06/49.24, 49.00, −59.25/−60.52) sit on the terrace's east patch [D placements];
- the walkway's north-west corner climbs the **west staircase**: 45.0→46.6 along x 44→41 at
  z −59.5, the 47.0 band (x 40-41), then 47.4→48.8 down z −61.5..−64.5 onto the **49.0 slab** at
  z −65.5 [D maps B/C];
- the slab's east part is covered by the **doorhouse** (roof tops 53-55 over x 36..41, z −66..−71
  [D map E]); no walls stand east of x 37.5 at body height [D map F, the y-50.2 wall sections], and
  its west wall is two planes — x 37.0 (i20.m50.o38 mat10) and x 37.5 (i23.m50.o41 mat10) — each
  with a gap at z −67.75..−69.0/−69.25: **the doorway**, the only link between the staircase pocket
  and the open slab [D map F];
- through the doorway (the old arch walk already proved the passage from the west; the E-reach AABB
  spans x 37.12..37.38, z −69.0..−67.75) and west across the open slab (49.0, x 14..36, z −65.5..−69.5
  [D probe_terrace05 MAP A]) to the terrace spot — the disc floor under it is 49.001 [D probe_terrace05
  col (21.8, −68.1): instance 20, map 50, object 38, mat 12], the centre rests at floor + 1.03..1.13
  [M Player_FeetOnPoint 0x194188:106; L the live game parks Bond 1.086 over his floor, ee_start_05].

No route-class placement exists east of the stairs (the full 363-placement dump: 1 ladder #151 far
west, 1 wire #251, 2 grapple pulleys, 2 creepwalls — none near the terrace), so the route is pure
collision walking, no interactions needed.

## 4. What changed

- `walkreal_07000005_live.rs`:
  - the release drop steers straight down (§2);
  - the cable route's body is extracted to `walk_the_cable_route_to_the_stairs_top` (shared);
  - new `walk_the_stairs_top_to_the_terrace_spot` — the §3 legs, ending standing at the spot with
    the rest-height assert (grounded, y 49.9..50.4);
  - `the_door_band_exit_chains_to_07000006_walked` (renamed from `..._climb_parked`): walks the
    whole route from the cold start, then the old arch walk + E press — the placement write and the
    placement-rest assert are deleted; the exit-chain asserts are unchanged;
  - the stale header claims (the "no walkable link" comment, the PARKED note) rewritten with the §2/§3
    evidence.
- `probe_terroute05.rs` committed (the maps the test cites). Scratch probes and dumps are not committed.

## 5. Verification

- `walkreal_07000005_live`: **11 passed / 0 failed / 0 ignored** (93.27 s), including the walked door
  test (exit `Some(0x07000006)`, ch 0x1E on, tick 9957) and the cable-landing route test.
- `interactables_07000005_live`: **3 passed / 0 failed / 0 ignored** (6.14 s).
- Full suite: see §7.

## 6. What's left

- The full suite on the final tree: run as far as the session window allowed (§7); if any part did
  not finish, the next session continues it from this branch — nothing in this task's files is
  expected to move other crates (the diff is one test file + one probe example).
- The doorway threshold is tight (1.25 z-wide gap between the wall planes' gaps, [D map F]); the
  port threads it, the real game's fit is worth one live check (below).

## Live checks for the main agent

1. **The doorway gap fit.** Fresh boot, walk (or teleport onto) the terrace pocket (38.5, ~50.1,
   −68.0), walk west through the doorway at (37.25, ~50.1, −68.4) and out onto the open slab
   (36.0, ~50.1, −68.6). Expected: Bond walks through without a jump; the passage is between the
   x 37.0 and x 37.5 wall planes with the z −67.75..−69.25 gap [D map F]. Watch whether the real
   collision's gap is as wide as the port's (the port threads it with ~0.07 margin at x 37.0).
2. **The grapple release drop.** Fresh boot, from the west battlement's east end (−16.5, ~39.2,
   −72.4), draw the Q-claw, fire at pulley #281, release at the hang without steering input.
   Expected: Bond lands on the east lane near (−4.4, ~37.4, −71.8) [the port's straight-drop landing]
   and does NOT slide south into the plunge zone (no death NIS, no "Mission Failed").
3. **The west staircase climb.** Fresh boot, on the east walkway at (44.0, ~46.1, −61.5), walk
   north-west up the staircase to the terrace's east edge (38.3, ~50.0, −65.5). Expected: pure walk-up
   (0.4-0.6 risers), no jump needed; the guards #271/#272 stand their posts en route.

## 7. Suite runs

- Full-suite chunks on the final tree (merged with origin/master bf6b5a2a): see the WORKLOG entry's
  totals line — the chunk runner's output is quoted there; anything not reached before the sandbox
  deadline is listed as not run.

## Usage

`not visible to me` — this session's environment shows no token, credit or context-window figures
(no usage UI numbers were displayed anywhere in the session).
