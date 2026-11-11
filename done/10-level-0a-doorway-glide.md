## Your task: `level-0a-doorway-glide`, Night Shift's west-hall doorway crossing matches the game

Branch from origin/master. Two walkreal_0700000a_live tests are ignored: the_west_hall_glide_height_matches_the_game (the settled y never reaches the game's 1.1285 [L 2026-10-26]: 1.0905 on master) and the_corner_crosses_like_the_game (the crossing mechanism is not the port's move clip; read the full ignore message). These are how Bond slides through a doorway corner, which a player feels. Find the game's mechanism [M the player collision handler and the doorway contact push], port it, and un-ignore both, with a test red on the old code. If the answer needs a live capture, write the exact question in the report. Run walkreal_0700000a_live, interactables_0700000a_live and every walkreal_*_live binary one level per tool call if the player collision changes.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/10-level-0a-doorway-glide`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox3 at 2026-11-11T15:22Z

---
## Report (done, 2026-11-11T15:49Z)


---
## Report (done, 2026-11-11 ~16:55Z) — full report on the branch: docs/reports/level-0a-doorway-glide.md (s/level-0a-doorway-glide, pushed). Summary:

Both reds REPRODUCED on master 784ba2d99 [L the runs]: the glide settles 1.0905 at (-9.935,
17.020) / 230 ticks (0.038 short of the game's 1.1285); the corner crossing ARRIVES but takes
6000 ticks (assert <=300 red) — Bond parked at the wedge corner (-17.87, 1.17, 4.87) grounded
FALSE (the band-edge dither) the whole cap. The game-side mechanism read [M] end to end:
the walked wall response is the BLData+0x750 push applied to obj+0x30 inside
Player_CollisionHandler 0x1949a8:136-140 (Collide_Update zeroes +0x750 [M 0x1ea398:50];
Intersect_CylGeom's per-tri push accumulates _DeltaP with the copy's segment shifted per tri
[M 0x109da0:447-470]; the add-back lands it in BLData +0x10/+0x20/+0x30/+0x50/+0x60 [M
0x1ebaa8:353-373]) — and the PORT ALREADY MODELS THIS SHAPE (world.rs capsule_push_scoped's
per-face shift + the pos += push mirror), so the missing behaviour is the walk's VERTICAL: the
feet band rides floors up to ~0.1 above the feet [M 0x194188:106] (exactly the glide's +0.038
step) without lifting, the walk-anim root ride (cc fresh per tick [M 0x194df0:86]) and the
walked dy [M 0x193600:284-295 + 0x192310:70-74] are both absent from master (band-settle-8
landed the dy on its own unmerged branch). No ignore lifted — the writers need live captures;
the ledgers carry the updated evidence + the four live checks (the glide look trace, the hall
floor's live y, the corner band hook, the tail operand decode). A floor-sweep probe
(probe_glide_floor_and_push) is in the tree. No crates/*/src change landed; no shared helper
changed; no test weakened.

## Usage

not visible to me (no token, context-window or cost figures are exposed in this session).
