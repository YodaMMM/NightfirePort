## Your task: `cutscene-side-strips`, cutscenes fill the whole widescreen frame from the cutscene camera

On a 16:9 window, an in-engine cutscene draws its camera only in the 4:3 centre: the side strips outside it keep showing the gameplay camera's view. Repro: `nf-view <iso> 07000004 --bond -6.12,5.44,36.24,-0.402,-0.005 --frame 1694 --shot out.png` — the centre is the cutscene's top-down shot of the courtyard, the left/right strips show the courtyard wall and roofs from Bond's own camera.

1. Find where the port renders the cutscene (script camera) view and where the side strips come from (a second pass, the 4:3 letterbox/pillarbox, the HUD layer) in `crates/nf-view/src/` (`script_camera`, the frame/draw path, the cutscene letterbox bars).
2. Make the cutscene camera drive the whole frame: the strips either show the same cutscene camera (a widened view at the same vertical FOV — the port's widescreen rule for gameplay, `view_aspect`) or plain black, whichever matches how the port already treats widescreen in gameplay; the cutscene's own top/bottom bars stay. Never the gameplay camera during a cutscene.
3. A test that during a script-camera frame no draw uses the gameplay camera, and headless `--shot` renders of the repro frame before/after named in the report with their paths.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/cutscene-side-strips`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.



---
claimed-by: sandbox4 at 2026-10-24T21:11Z

---
## Report (done, 2026-10-24T21:48Z)

