## Your task: `level-16-jetpack-escape-4`, 07000016: what the real mission order changes before the launch pit

On master the pit timeline is pinned and matches the live game (`walkreal_07000016_live::the_pit_pulse_kills_an_idle_bond_about_450_ticks_later_like_the_game`): touch trigger #255 -> the pack cinema 0x06000631 -> re-enable at the pit floor (3.57, -22.76, -179.55) -> ch 243 at +~1603 -> the death sequence fails the mission ~450 ticks later. The live run's touch-to-re-enable gap (553 observed vs 747 modelled) was a measurement artifact: Bond was placed on the trigger around frame 470, not at the logger's first row (687), so the gap is ~770 [L].

The data shows no way to survive pulse 1 from the pit floor (the six-band hurt stack tiles the pit, the blast rooms are sealed by BlockingPolys 117/118, the shuttle-bay lift's ch 80 and the pit door's ch 240 are written after the pulse [D]). Yet the shipped mission is completed by players. The live run skipped everything before the bridge (it placed Bond on #255 directly): the chasers, the Kiko death button and their channels.

1. List every channel and script state the mission's real order sets before #255 is touched (the chasers' defeat, the Kiko button, objectives) and find any branch they select: an alternate SP or script for the cinema or the launch (another SP gated on those channels), a different hand-back pose, a gated hurt volume (its channel low), or a script that moves Bond [D script dump + M SP_Update/SP_Hit].
2. Run the port in the real order (`walk_the_chasers_defeat_completes_the_mission`'s legs) through the bridge and compare the cinema, the hand-back pose and ch 243 with the teleported run.
3. If the order changes nothing in the data, name the live check precisely (fresh boot, play the real order, what to watch) and stop there; the walk test stays ignored until it is settled.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-16-jetpack-escape-4`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox4 at 2026-10-24T01:07Z
