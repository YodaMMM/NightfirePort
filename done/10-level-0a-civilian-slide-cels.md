## Your task: `level-0a-civilian-slide-cels`, civilians on screen never slide without a walk

Branch from origin/master. On 0700000a the office civilians walk with no leg movement (they glide) while in full view. Cause, measured by 10-level-0a-guard-no-anim [L sim run]: the animation state is right (AI 0x14 CivilianPatrol, danim 392, anim state 0x37), but the sim-side "drawn" flag (`Drone::drawn`, the stand-in for obj+0xFC) is false, so `NDrone2_DoAnimation` 0x149f88's not-drawn arm slides the body at the danim speed with no gait. The game sets obj+0xFC from `View_AddObjects` 0x1e62d8:71: the object's world sphere passes `Vision_InView` over the PORTAL-VISIBLE cels, with no rays. The port adds bullet-segment rays as a room stand-in; master already lets them pass glass (the lounge window), but thin posts (LaserTrackVert columns, mat 0x06) and non-pass foliage still block them, so a civilian in a visible room behind a post reads undrawn and glides.
Vantage that showed it: Bond at (4.59, 0.65, 2.43) (cel 29), looking down the security centre's west sightline, about frame 7485 on the real route (arrival plate, out of the cab, lobby, the crouched west-hall crossing under beam #607); placement #695 patrols the atrium lane there.
Do: give the drawn gate the game's own room rule: the drone's cel is in the portal-visible cel set from the viewer's cel (nf-view already has `CelGraph` + `portal_visible_cels_*`; move or share what the sim needs into nf-game, no draw changes), then the frustum test, no rays. Keep `drone_position_visible` (the spawner's reader) separate. Tests: a real-input walk from the load to that vantage, red on master (an in-view civilian on danim 392 with drawn == false), green after; also pin the lounge-window patroller (the z -4.71 glass pane, mat 0x10): he must read drawn and walk with his gait while in view through the glass; then check the guards-0c and 05-castle slide tests still hold (the opaque-occluder case: a guard in a room that is NOT portal-visible must still read undrawn).

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/05-level-0a-civilian-slide-cels`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox4 at 2026-11-12T01:12Z

---
## Report (done, 2026-11-12T02:16Z)


---
## Report from the worker (done, 2026-11-12T02:16Z)

