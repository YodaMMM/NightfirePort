## Your task: `lobby-roof-glass-09-3`, 07000009: break the lobby roof glass #123 with a shot whose line is clear

`interact_07000009_live::shooting_the_lobby_roof_glass_breaks_it` is ignored. Master now gives a round ONE contact, at the nearest solid hit, and kills it there [M Collide_Update 0x1ea398:216-229, Bullet_CollisionHandler 0x12ce80]. From the test's stance (settle near (8.5, -369.0, 34.5), aim at (8.02, -366.0, 36.4)) every P2K round meets world geometry at t ~0.31 of its flight segment, before the pane #123 (slab x 8.023, z 36.10..37.38, y -369.25..-354.12 [D]; plane crossing ~0.47), so no contact reaches the glass. The glass's health is 6.0 [M Break_Create cvt.s.w; D param 1], the P2K's contact 3.5 [L weapon_data].

1. Name the triangle the rounds hit (owner object, surface byte) and decide whether a real player's shot from there would be stopped the same way in the game.
2. Find a stance and aim a player would use with a clear line to #123 (real input only, the aim through turn/look, no yaw writes) and pin: two P2K contacts leave 6.0 -> 2.5 -> -1.0, the kill on the second, sound 0x2E.
3. Sister: the neighbouring pane #116 is untouched.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/lobby-roof-glass-09-3`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

