## Your task: `level-16-jetpack-escape-3`, 07000016's launch pit: the kill timing and the way out

Start from `s/level-16-jetpack-escape-2` (merge it into a fresh branch from master). That branch shows the port kills Bond one tick after ch 243 (HurtPoly-ShuttleLaunch #309) rises, with the pit door (ch 240) and the shuttle-bay lift (ch 80) both too late.

Live [L fresh boot, PCSX2, god mode, Bond placed on the bridge trigger #255 at (2.37, 1.04, -176.41), no input]:
- ch 0x81 on at the touch (~frame 687); Bond re-enabled at the pit floor (3.5714, -22.7564, -179.5529) around frame 1240 — the port's [D] hand-back is right.
- ch 243 rises at frame 2831, ~1591 frames after the re-enable (the port: ~863).
- Mission Failed at ~frame 3285, about 454 frames after ch 243 — not one tick after. Health read 100 on every logged frame (god mode rewrote it each frame), and Bond was nudged to (3.62, -23.90, -178.31) in that window.
- ch 240 and ch 80 stayed 0 through frame 3285.

1. Re-derive the timeline against these numbers: why the port's ch 243 comes ~730 frames early (the Wait semantics, the script start, the stream clocks) and why its kill is instant. Read `Hurt_Update` 0x17e5d0 in full: the damage rate, any delay, and how a hurt volume ends a mission (a fail script, a death NIS) [M].
2. Then find the player's way out in that ~1600 + 454 frame window: what the real route is from the pit floor (a ladder, a door, a lift, a script move). The previous round found no jetpack in the data [D]; follow the data, not the assumption.
3. Make `walk_the_chasers_defeat_completes_the_mission` pass with real input, or re-ignore it with the exact remaining blocker. Sister: an idle Bond dies about 454 frames after ch 243, like the game.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-16-jetpack-escape-3`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox3 at 2026-10-23T22:45Z

---
## Report (done, 2026-10-23T23:40Z)

