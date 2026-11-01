## Your task: `tripwires-shot-from-range`, tripwire mines can't be blown up from a distance

On the tripwire levels (0700000a, 0700000b, 07000014) shooting a tripwire mine from range does not set it off in the port. Find what the game does when a round hits a tripwire's mine/emitter [M the tripbomb class's hit/damage handler, Bullet_CollisionHandler 0x12ce80's class dispatch; D the tripwire placements' collision], port it with the address cited, and pin: a P2K round from 15+ units at a mine detonates it (blast + beam gone), a round that misses does not. Run the tripwire/trigger binaries you touch, every *0700000b* / *07000014* walk and nf-game; quote the totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/tripwires-shot-from-range`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox7 at 2026-11-01T04:29Z

---
## Report (blocked, 2026-11-01T05:41Z)

