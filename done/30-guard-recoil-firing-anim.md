## Your task: `guard-recoil-firing-anim`, guards with a recoil weapon fire on the anim event and play the recoil

`crates/nf-game/tests/anim_events_live.rs` `play_firing_anim_gates_the_recoil_weapon_shot` (~line 232) is #[ignore]d. DroneAnim_PlayFiringAnim 0x13da48: a drone whose weapon has the recoil flag (+0x3E) and a shot due fires only on the anim event +0x21, and plays the 0x5C recoil [M]. The gate is in drone.rs handle_firing / fire_now, but the harness never gets a shot due: the state machine or an unmodelled gate resets it. Find the real state where the game reaches PlayFiringAnim with Bond in range [M the attack states' UPDATE paths]. Drive a recoil-weapon guard there with real ticks, not forced fields. Prove all three rows (recoil without the event: no shot; recoil with the event: shot + 0x5C; no recoil: shot). If the port's state machine resets the due shot where the game keeps it, fix the port. Un-ignore the test. Run anim_events_live, enemy_accuracy_live and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/guard-recoil-firing-anim`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox5 at 2026-11-03T05:08Z

---
## Report (done, 2026-11-03T06:52Z)

