## Your task: `level-14-full-walk-4`, Countdown Part 1's walk past the hood to the exit

A previous session (`s/level-14-full-walk-3`) ended after setup with no change. Start from origin/master (it has `level-14-full-walk-2`: the release onto the east vent lane is green, the walkable gate is up-facing only [M Intersect_CylGeom 0x109da0 0x10a650-0x10a674], the phase-2 hang drift is ported) and read `docs/reports/level-14-full-walk-2.md`. `walkreal_07000014_live::walk_the_infiltration_to_the_delta_exit` is `#[ignore]`d on the leg-1f hood-underside blocker. Clear it from the game's rule [M] (collision, the lane geometry [D]), walk leg 1f (the lane south over the flush lid, the connector west to the #301 drop), the ch 0x47 wait, then legs 2..6 to the exit on real input only. Un-ignore when green end to end; otherwise back to `#[ignore]` with the exact blocker. Commit and push progress early.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-14-full-walk-4`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox6 at 2026-10-27T15:16Z

---
## Report (done, 2026-10-27T15:51Z)

# level-14-full-walk-4 — leg 1f walks green (the walk-2 blocker is gone on master); the walk stops at leg 2's start: the game's vent drop never carries Bond across

Branch `s/level-14-full-walk-4` = `origin/master` (6ff39ea9) + the updated `#[ignore]` comment.
No gameplay code changed this window — the walk-2 blocker dissolved under the later collision
merges, and the walk advanced to a NEW, evidence-complete blocker one leg further in.

## WORKLOG entry

level-14-full-walk-4 (2026-10-27): branch s/level-14-full-walk-4 from origin/master. Leg 1f
walks GREEN — the walk-2 hood-underside blocker is gone (cleared by the later collision
merges): the lane south past the mouth, the connector west, the SP #276 ch 0x47 write and
objective 1 all pass inside walkreal_07000014_live::walk_the_infiltration_to_the_delta_exit
(still #[ignore]d). The blocker moved to leg 2's start: the vent drop into the facility never
carries Bond — the mezzanine south edge is a wall (z ~38.21, top ~3.5-3.7, not jumpable
[M apex 0.78]), the #301 touch (out 0xd9) fires with no effect, SP #276 ends without moving
Bond, and nav_route west from the mezzanine is None. Report + live checks:
docs/reports/level-14-full-walk-4.md.

## What is done

1. **The leg-1f blocker is gone — verified green in the real test.** The task's target blocker
   (the hood-underside wedge on `walk_to((45.0, 2.4, 40.0))`) does not reproduce on master. The
   un-ignored run walked: the lane south past the mouth **arrived (45.14, 2.95, 40.77) at +58
   ticks**, the lane south to the connector **arrived (45.13, 2.95, 39.95) at +41**, the
   mezzanine west across the sloped connector **arrived (41.88, 2.79, 39.03) at +1465**, then
   the SP #276 cut-scene wrote **ch 0x47** and **objective 1 completes** (`mission.states[20][0]
   == 4`). The walk-2 "hood underside band y ~3.4..3.9 over z 42.2..43.5" premise does not
   match the geometry: the probe re-run shows the strip z 42.3..43.5 open to the sky over the
   lane (2.38), with solid covers only at z ~42.1 (top 3.93) and z ~41.7 (top 4.10), both with
   ≥1.55 head clearance above the lane floor [D probe14_hood A/B up/down-ray grids] — the walk
   passes under both without contact.
2. **The new blocker diagnosed with the geometry, the trigger and the SP.** From the mezzanine
   (41.88, 2.79, 39.03) the game's route must drop into the facility ("the vent drop TouchOnce
   #301" per the route header). Measured:
   - The mezzanine floor (2.15) spans z 39.5..38.25 [D probe14_mouthfin C/D]; south of z 38.0
     the floor is a deep open trench (z 37.25..38.0, down-rays open past −1) spanning the whole
     band x 39.4..45.
   - The trench's north edge is a WALL: south-rays at x 40.9 hit z **38.21374** for every y
     2.5..3.5 and open only from y 3.7 [D this window's scratch rays] — top ~3.5-3.7. The
     walking jump apex is 0.78 [M Player_HandleJump vel = −GRAVITY·0.4, GRAVITY 9.8,
     crates/nf-game/src/player.rs:1541], and clearing the wall needs the spine bottom above
     3.6, i.e. center > 4.6 — **the game's Bond cannot jump it either**; a run+jump attempt
     from the edge confirmed the port's walker never leaves the ground there.
   - The **#301 TouchOnce (out 0xd9 [D], centre (40.72, 3.10, 39.01), band 39.68..41.75 /
     2.07..4.14 / 37.62..40.40)** fires on the west walk (the connector walk crosses x 41.75
     inside the band) but nothing in the port moves Bond down.
   - **SP #276 (0x06000833, 1400 ticks [D])** runs to its end (sim disabled ~1000 ticks, ch
     0x47 written at its tail) and leaves Bond at the mezzanine — the port's SP does not
     SetPosRot the player (the scratch printed pos constant (41.88, 2.83, 39.03) through the
     whole cut-scene and past it).
   - `nav_route` west from the mezzanine is **None**: the only nearby nodes (41.3/38.7,
     0.4, 36.1) sit SOUTH of the trench, unreachable on foot; the next nav-connected ground is
     the far facility floor [D the nav dump: 83 nodes west at y<3, none on the mezzanine].
   The test is therefore **still `#[ignore]`d** with the exact blocker in its comment (never
   red and un-ignored), and the old walk-2 blocker text is replaced with this one.
3. No production code changed; the branch carries only the test-comment update and this report.

## What ran

- `cargo test --offline --release -p nf-view --test walkreal_07000014_live -- --ignored
  --nocapture` (NF_ISO set, release): the walk ran green through the crossing, the trench, the
  stairwell, the shaft entry, the release, leg 1f, the ch 0x47 wait and objective 1, and failed
  at leg 2's `follow_nav_route` wedge (the evidence above) — **2 passed, 1 failed(ignored-only)
  on the file** with the main walk re-ignored after the diagnosis; the file's other two tests
  re-ran green (see below).
- `cargo test --offline --release -p nf-view --test walkreal_07000014_live`: **2 passed,
  1 ignored** — the crossing chain and the vent-grill test green on the unchanged tree.
- `cargo run --release -p nf-view --example probe14_hood` (the walk-2 leftover probe, first
  run of this window) and `probe14_mouthfin`: the lane/covers/connector grids behind §1 and §2.
- A teleport scratch (removed before the push) reproduced the mezzanine state and measured the
  wall, the touch, the SP and the nav — never committed.
- The full `tools/test/suite.py` did NOT fit this window: the sandbox was reset (repo, work/ and
  the ISO all redownloaded — setup consumed the first ~25 min), the background suite died
  between tool calls (the known sandbox behaviour), and the remaining window went to the leg-2
  diagnosis. The touched crate's tests (the walkreal file) are the verification above.

## What is left

1. **The vent-drop handoff** — find which script/SP record moves the player at the vent drop
   (the #301 out 0xd9 chain into SP #276's records, or a script gated on 0xd9), port it, and
   re-run the walk from leg 2.
2. Legs 2..6 to the Delta exit on real input, then un-ignore.
3. The full suite on the final tree.

## Live checks for the main agent

Fresh boot 07000014, at the mezzanine after the connector walk (Bond ≈ (41.9, 2.8, 39.0)):

1. Walk west into the #301 band (x < 41.75 at z ~39): does the REAL game start a cut-scene
   (the SP #276 vent drop) and where does Bond stand when control returns — inside the
   facility on the lower floor? gdbwatch the player object's pos through the SP.
2. Does the game's Bond ever walk/jump the mezzanine's south edge himself (the wall top
   ~3.5-3.7 at z 38.21), or is the drop exclusively cut-scene/script-carried?
3. Which record carries the move: trace the script reads of channel 0xd9 (the #301 out) during
   the touch — the SP start, a SetPosRot on the player object, or a floor/lid movement?

## Usage

not visible to me — this sandbox exposes no token, credit or context-window figures to the session.
