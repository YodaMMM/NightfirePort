## Your task: `pane-ride-on`, intact glass really stops Bond on every level

Branch from origin/s/pane-collision-pairing, merge current origin/master, and read its docs/reports/pane-collision-pairing.md. The pairing is proven and the ride is wired behind `PANE_RIDE` in drone.rs. With it on, both dead_pane_live tests and the named battery pass except the three 07000001 real-damage walks, which die to guard melee at spots with no pane, because capsule solidity changes guard motion and melee timing there. Do not treat that as a reason to keep glass ghostly. Find which pane first changes which guard's path on 01, tick by tick, against the switch off. Then read how the game limits a moving body's contact set: `Collide_PickObj` 0x1eae80 and the room or cel link of a 0x20 object. If the port's capsule walk sees panes the game's would not, add per-cel scoping for capsules like the stand-in the rays got. If guards really do brush those sheets in the game, the walk is what changes: re-route it or change its fights, helpers gated to that level, never lowering damage or health. Then set `PANE_RIDE` on and remove both ignores. Keep the kind-0/1 props out of the set and carry that report's live check on them forward. Run dead_pane_live, glass_pass_every_level_live, every walkreal_*_live, the ai binaries and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/pane-ride-on`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

