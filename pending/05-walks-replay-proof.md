## Your task: `walks-replay-proof`, every mission walk replays from its own inputs (FIRST PRIORITY, after 05-replay-parity-walker-side-effects)

Branch from origin/master (merge s/05-replay-parity-walker-side-effects first if it is on origin). A walk test only proves what a player sees if a player could produce it: the same inputs, replayed in the app (`nf-view <iso> <level> [--difficulty N] --replay file.nfr`), must reach the same end. Measured: the 0700000b real-damage walk replays tick-identical to tick 7860, then splits, because walkreal_0700000b_live.rs writes `sim.bond.pos = [12.21, 47.4, 19.68]` after "the exec floor" leg (the "one kept stand-in" hop) and again for the use-plate stances; the replay in the app ends in Mission Failed with Bond falling off the outside of the building at frame 11450. Position writes also sit in walkreal_0700000a (3), 07000009 (2), 07000014 (2), 07000015, 07000016, 0700000c, 0700000c_barrelrow.
Do, for each mission walk (walk_* tests in walkreal_07*.rs, not the probe_ files):
1. Remove every write to Bond's state (pos, vel, yaw, health, weapons) and every sim poke (channels, drone fields, doors) between load and mission end. Replace each with player input: jumps, crouches, other routes, use presses. Where the walker truly cannot, take the route a player takes (furniture tops, the game's own ledges) and say in the report what it is.
2. Record the walk with the walker's single step path (from 05-replay-parity), save the .nfr, replay it on the SAME loader the app uses for that flag set (plain `load` without --difficulty; `load_with_carry_difficulty` with it), and assert the same mission_end tick and Bond's position every 600 ticks.
3. Start with 0700000b (Night Shift, real-damage, difficulty 1), then 0a, 14, 09, 15, 16, 0c.
A walk that still needs a write after this is renamed probe_* and does not count as a level walk.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/05-walks-replay-proof`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.
