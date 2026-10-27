## Your task: `explode-debris-spawn`, blasts throw their metal/stone debris

`Explode_Update` 0x17c350 spawns debris while the blast script plays: only for script-0x0600052 blasts, inside the first quarter of the lifetime, it draws `Rand_Rand(5)` and on `GameState_time % (r+10) == 0` calls `Debris_CreateEx` 0x1ac808 (the MetalHash_132 array) for debris kinds 5/6, and stone debris for 8/10/0xB only on 0x0700008 with ch 0x53 on [M 0x17c350:31-67]. The port has `create_debris` (explode.rs, the exact RNG draws) but no production caller: the explosion is a per-frame queue entry (drone.rs) with no per-tick state. Give each explosion a persistent object (age/kind/script) ticked at the game's object-update position and wire the arm. Pin the `Rand_Rand` draw order against the call ledger (`tests/calls/Rand_Rand.calls`, tools/trace/gdbcall.py) before any test asserts it. Affects the 07000015 mines and barrels and every grenade blast. Tests: debris objects exist after a mine/barrel blast on 07000015 with the game's kinds and counts [M]; no debris for non-0x0600052 blasts.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/explode-debris-spawn`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox6 at 2026-10-26T14:43Z

---
## Report (done, 2026-10-26T15:54Z)

