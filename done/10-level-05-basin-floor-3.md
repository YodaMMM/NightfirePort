## Your task: `level-05-basin-floor-3`, the solid hurt planes land with every level green

Branch from origin/s/level-05-basin-floor-2, merge current origin/master, and read docs/reports/level-05-basin-floor-2.md. Start the full suite in your first minutes: the fold touches every level with an ungated hurt plane and has never had a full run. Then: (1) walkreal_07000005_live::the_door_band_exit_chains_to_07000006_walked_real_damage is red from a roll shift at the stair-top guards #12/#13 (35 hits for 100 vs 31 for 85.5 pre-fold); take the report's lever (shoot the #12/#13 post from the stair base before the scripted legs enter it, gated on REAL_DAMAGE_BACKOFF). (2) Port the gated planes' solidness: Hurt_Update 0x17e5d0:26-37 sets or clears obj+0xF0 bit 0x20 every tick from the gate channel, and Collide_Pick 0x1eb320:140 skips bit-0x20 objects, so a gated plane becomes solid the tick its channel rises; test it red on the old code (07000016's HurtPoly-ShuttleLaunch #309 on ch 243 is a candidate). (3) Fix every suite red the fold causes. Live fact (PCSX2): the plunge fires with Bond standing AT the lip stand (132.0, 14.2, 125.4) (ch 190 rises), matching the port. Run every walkreal_*_live and interactables_*_live binary one level per tool call.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/10-level-05-basin-floor-3`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox1 at 2026-11-10T11:29Z

---
## Report (done, 2026-11-10T12:55Z)


---
## Report from the worker (done, 2026-11-10T12:55Z)

