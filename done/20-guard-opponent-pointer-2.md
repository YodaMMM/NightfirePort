## Your task: `guard-opponent-pointer-2`, the opponent pointer without the two AI reds

Continue `origin/s/guard-opponent-pointer` (merge `origin/master` first; read its report). Merged onto master it turns two tests red (both green on master, both red when re-run alone):
- `grenade_live::guard_decides_to_throw_grenade_unforced` (grenade_live.rs:249): the guard never decides to throw within 1800 ticks (can_throw_grenade never passes; NF_AILOG=1 shows why);
- `bathhouse_shout_mixer_live::the_bathhouse_floor_guards_shout_reaches_the_mixer_too` (bathhouse_shout_mixer_live.rs:234): no wake-group guard shouts.
Find which reader of the pointer/grace changed each path and what the game does there [M the grenade decision and the shout/wake path; cite lines]. If a test pinned the old stand-in, fix the test with the decompile cite; if the port reader is wrong, fix it. Run nf-game's crate tests, both binaries, every *guard*/*drone*/*alarm*/*shout* binary and the walkreal binaries for 03, 09, 0b, 13; quote each total.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/guard-opponent-pointer-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox3 at 2026-10-30T12:33Z

---
## Report (done, 2026-10-30T14:25Z)

