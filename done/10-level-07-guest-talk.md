## Your task: `level-07-guest-talk`, 07000007: the party guests react to and talk to Bond

On the party level the guests never talk to Bond or react to him. `fix_guest_reactions_live::` has a test ignored on "the sight pass's `seen` gate is not headless-drivable yet".

1. Find the game's guest behaviour around the player [M — the party-guest / civilian DMODE inits and their DSTATEs (idle, mingle, look-at, greet/talk), `SpeechSFX` / the speech table they draw from, the gates: distance, sight, facing, cooldowns, the party channels], and what the port has; port the missing states and lines.
2. Make the `seen` gate drivable with real input (Bond walks up to a guest in view) and un-ignore that test; add a test that walking up to a guest plays a guest line (the speech id from the game's table) and turns the guest's head/body toward Bond if the game does.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-07-guest-talk`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox4 at 2026-10-25T01:18Z

---
## Report (done, 2026-10-25T01:48Z)

