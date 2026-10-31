## Your task: `level-0c-secret-door-reach`, 0c: opening the secret door the way a player does

level-0c-remaining-teleports-to-walks (merged; read its report) found the secret door #544's plates #558 (E-arm) / #559 (body touch) sit in a corridor whose touch volume starts at x 52.24, while the body-touch capsule reaches only x <= 51.84 from the yard side — so the door cannot be opened from the yard, and `walkreal_0700000c_live` still places Bond at the plates. Find the game's way into that corridor [D the corridor's north end, the mezzanine annex vent #180/#182/#184 (ch 0x54), any door/hatch that opens onto it; M the classes involved], walk it on real input, open the door through the plates, and delete the placement. Fix any port gap with evidence. Run walkreal_0700000c_live and interactables_0700000c_live; quote the totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0c-secret-door-reach`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox5 at 2026-10-31T07:29Z

---
## Report (blocked, 2026-10-31T08:17Z)

