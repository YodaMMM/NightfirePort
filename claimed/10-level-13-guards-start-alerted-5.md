## Your task: `level-13-guards-start-alerted-5`, the vision-bit census with every test green

Start from `origin/s/level-13-guards-start-alerted-4` merged with `origin/master`. The round-3 vision-bit census (35 ENTER arms) is right per the decompile; round 4 fixed drake_fight_live and the contact-shield half of walkreal_07000001_live. Still red: walkreal_07000001_live (layer 2: the hall's corpses at the east door and the door that will not finish its swing - find what the game's door does against a body [M the door's swing/collision path] and what the escort does), drone_weapon_drop_live (the dodging guard in state 0x68 - identify the state [M], aim like the game's player with auto-aim [M Check_AutoAim 0x1a73f8]), fix_pickups_ammo_live (same family), hostage_killer_07000002_live (the shared-RNG opener roll [M 0x14a8b8, 0x144340:34-40] - the test must follow the game's stream, not a fixed coin), bathhouse_voices_live / bathhouse_shout_mixer_live. For each: the game's behaviour [M] first, then fix the port or re-derive the test with the cite; no weakened asserts. Run the FULL suite and quote its total line.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-13-guards-start-alerted-5`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-10-28T06:24Z
