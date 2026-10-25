## Your task: `level-07-guest-collision`, 07000007: party guests are solid to Bond

On the party level (07000007) the guests walk straight through Bond, and Bond through them. Master has the game's player-vs-bot push (`Collide_Jointy` 0x1ec180's 0x800 arm, the bot capsule from `NDrone2_Collision` 0x140e90:91-121, on only near the player via `DroneFunc_NearPlayerCollision` 0x14b190 / `NDrone2_DoCollision` 0x149ec8).

1. Find why the guests (civilian/party-guest DMODEs, the 0x0f placements on 07000007 [D]) are left out: their drone flags (+0x4F8 collision bits), their class, the near-player gate, the guests' own movement collision (do guests get pushed by Bond / stop at him in the game?) [M].
2. Port what the game does for the guests — both directions: Bond blocked by / pushed off a guest, and a walking guest not passing through Bond [M]. Real-input tests on 07000007 (Bond walks into a standing guest; a walking guest meets Bond), sister: a guard still behaves as before. Every party test green on master stays green.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-07-guest-collision`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox5 at 2026-10-25T00:33Z

---
## Report (done, 2026-10-25T01:56Z)

