## Your task: `goggles-remaining-draws`, the goggle draws left over

origin/s/goggles ported night sight and x-ray (read docs and its goggles_render_live test). Left: (1) in night sight a class of objects draws at a quarter of their alpha (no port object carries that flag yet — find the flag [M the night-sight draw path] and which disc objects carry it [D]); (2) cold breath has an x-ray-only variant, and breath is not wired into the game ticks — wire breath (the game's cold-breath spawner on snow levels [M]) and its x-ray variant; (3) the guards' weapon-0x18 laser ray is not drawn in either mode; (4) tripwire beams' movement along their path. Continue from origin/s/goggles (merge origin/master first). Render before/after headless (`--goggles 1|2`), tests with sisters. Run every render/gadget/hud binary and load_scene_equality/digest. Commit and push your branch after each step so a reset loses nothing. Run the named binaries and quote every total.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/goggles-remaining-draws`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

