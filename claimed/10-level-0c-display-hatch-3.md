## Your task: `level-0c-display-hatch-3`, 0700000c: past the bay floor into the display case

Start from `origin/s/level-0c-display-hatch-2` (merged to master: the capsule toucher + the bay-drop test) and read its report. Its new live checks, run on the real game (PCSX2, 0700000c through the game's loader, god mode, stick forward) [L 2026-10-26]:
1. **The bay floor's south side is a wall:** placed at (5.42, 12.9, -111.38), yaw 3.1416 (south), 120 frames: settled (5.42, 12.8964, -111.369), then held at z -111.3958 the whole time (x drifting 5.4186 -> 5.3938), y 12.8773. No drop into the 6.27 pocket from the bay's south edge.
2. **The mid floor's real height:** dropped from y 12.6 it settles at **(3.09, 12.3877, -113.54)**; dropped from 13.5 it lands at (3.2505, 12.3948, -113.54). The earlier 11.4248 / 11.6434 readings were Bond placed INSIDE the slab (the game holds a body placed below the surface where it is) — so the port's 12.36 mid-floor root is right within 0.03; there is no 0.93 gap to chase.
3. (SS #179's rest matrix y — not read: the object's runtime address is unknown. Find it from the SS list the port builds [D] and the game's object list [M control lists], or via a gdbwatch on the burn's write; then settle whether its poly freezes the roof walk at z -110.66.)
So the case is NOT entered from the bay's south edge or from the mid floor. Find the remaining route into the 6.27 pocket / the jetpack #414 at (5.50, 7.55, -110.37) from the level's own data [D placements, triggers, scripts, SS #179's keyed motion] and the game's rules [M], port what the port lacks, and un-ignore `from_the_duct_top_the_open_hole_carries_bond_down` (or replace it with the real route's test).

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0c-display-hatch-3`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox6 at 2026-10-26T19:08Z
