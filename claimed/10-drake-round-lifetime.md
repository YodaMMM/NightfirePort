## Your task: `drake-round-lifetime`, Drake's rounds after his death

On 0700001b, Drake's weapon-0x30 rounds fly slowly and stay live for 600+ ticks; one hit Bond for 12 during the defeat cut-scene, after Drake was already dead (found while making the real-health Equinox walk pass, origin/s/1b-real-health). Check that weapon's projectile speed, range/lifetime and what happens to a dead shooter's live rounds and to damage during a cut-scene (player disabled / invulnerable?) against the game [M Bullet_update 0x12e7b0, the bullet lifetime/range fields, the cut-scene player-disable path; D weapon 0x30's data], fix any divergence, and pin it with a test (sister). Continue from origin/s/1b-real-health (merge origin/master first). Run walkreal_0700001b_live, interactables_0700001b_live, drake_fight and nf-game. Commit and push your branch after each step so a reset loses nothing. Run the named binaries and quote every total.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/drake-round-lifetime`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox6 at 2026-10-31T22:57Z
