## Your task: `ssys-wire`, the security subsystems count down and fire in play

crates/nf-game/src/ssys.rs ports `SSys_Init` 0x1b01b0, `SSys_Monitor` 0x1ad998, `SSys_Msg` 0x1ad878 and `SSys_RamLoad` 0x1b0208, but nothing outside that file calls them: only its `REC_FRAME_RATE` constant is used, so the 240 s laser and camera countdowns, their announcements and their channel writes never run. Confirm that first. Wire all four the way the game does: `ResetMap_GameInit` 0x1d5080 calls Init (respect `IsWarmReset`), `Game_Run` 0x1c9790 calls Monitor after the `GS_IsPaused` gate and only while `ScriptCam == 0`, and `Player_RamLoad` 0x1957a8 calls RamLoad. Nothing in ssys.rs arms an item yet: the only writer of an item's channel is `Switch_Create` 0x1adcc0:29-31, which stores the switch's channel when placement field +0x44 is non-zero. Port that, and census every level for switches with a non-zero +0x44 [D], listing level, item and channel in the report. Check the save side too: find what writes the SSYS block that RamLoad reads, and make the port's checkpoint and Death-and-Replay path keep it. `Msg` sounds must go through the normal sound path with their level gates (0x0700000b with channel 0x76 off, camera 1 on 0x0700000a). Keep the game's tick order against the other switch-channel writers. Pin through the real path on a level from the census: Bond trips the channel, the announcements play at 180/120/60/6 s, the channel drops at 0 and whatever hangs off it reacts. If the census finds no placement, say so in the report and keep the wiring with a unit pin. Run every test that mentions switch channels on the census levels, walkreal_0700000a_live, walkreal_0700000b_live and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/ssys-wire`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox3 at 2026-11-07T01:06Z

---
## Report (done, 2026-11-07T02:21Z)

