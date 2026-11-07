## Your task: `pane-collision-pairing`, intact glass panes stop Bond and bullets, and shot-out ones do not

crates/nf-game/src/drone.rs (around line 19300) builds `_pane_parts` for the placed 0x20 panes but never adds them to the collision world. The comment says that with them in the set, walks, guard sight and dropped items on 01/09/0a/0b/14 stopped at shapes where no pane stands in the game, because the shape-key/matrix pairing is unresolved. Both tests in crates/nf-view/tests/dead_pane_live.rs are ignored for this reason. Read done/30-dead-pane-collision-2.md in the queue first. Then settle the pairing from the game: which glist and which matrix `Break_Create` 0x12c230 gives a pane, how `Collide_PickObj` 0x1eae80 walks it, and what `Break_Kill` 0x12b970:60 removes. Print the panes the port would add on those five levels next to their placements [D] and find which rows sit where no pane stands, and why (wrong key, wrong matrix, or an object that is not a pane). Script-created panes already ride through the entity arm just above, so keep one rule for both. Then turn the parts on, keeping the death-tick grace the comment describes, and respect each pane's filter [M Collide_FilterBullets 0x1ece30]: a smash-only pane must not stop or take a round the game lets through. Remove both ignores. Pin through real input: Bond cannot walk through an intact pane, can walk through after shooting it out, and a round stops at the pane only while it stands. Run dead_pane_live, glass_pass_every_level_live, walkreal_*_live for 01, 09, 0a, 0b and 14, and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/pane-collision-pairing`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox4 at 2026-11-07T07:58Z

---
## Report (done, 2026-11-07T08:59Z)

