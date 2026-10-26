# level-04-mission-hold-2 — the report

Task: keep `Mission_Update` case 7's hold freeze (`GS_PausePlayer(1, 0)` [M 0x182c60:123-130],
folded into the sim's input gate as `mission.internal == 7` by `s/level-04-mission-complete-early`)
without breaking the two tests that went red on the merged tree:
`walk_07000013_live::the_front_entrance_completes_the_mission` and
`civilian_no_shoot_live::the_trigger_fires_under_the_mark_and_the_civilian_takes_the_hit`
(each green on master, confirmed red here by re-running on the merged branch before any fix).

## What the game does during the case-7 hold (all [M])

Branch `s/level-04-mission-hold-2` = `origin/s/level-04-mission-complete-early` + `origin/master`
merged (no conflicts). Audit of the hold window:

1. **Which internal states reach case 7** (`Mission_Update` 0x182c60): case 2 (fail — case 1 saw
   ch 99, the fail channel, :45-57), case 3 (death — case 1 saw ch 0x62, Bond's health out,
   :58-64) and case 6 (success — case 1 saw ch 100, the completion channel, :85-122) all set
   `InternalState = 7`. The hold is one arm, shared by success and fail.
2. **The hold's length**: case 7 [M :123-130] runs `GS_PausePlayer(1, 0)` EVERY tick, subtracts
   `FRAME_RATE_MUL` from `TimeOut`, and at `<= 0` calls `GS_PauseGame(1)` and moves to case 4.
   `TimeOut` is 240 ticks on success (`(VIDEO_FRAME_RATE & 0xffff) << 2`, :122), 300 on fail
   (`(VIDEO_FRAME_RATE & 0xffff) * 4 + VIDEO_FRAME_RATE`, :52) and 1.0 on death (:58).
3. **What the pause actually freezes** — the per-player flag, not the world:
   - `GS_PausePlayer(char, ushort)` writes `(int)param_1` to `MPGame + slot*0x30 + 0`
     [M 0x17d280, disassembly: `sw $a0, ($v1)` with `v1 = 0x2a4980 + slot*0x30`].
   - `Player_Update` reads that flag through `GS_IsPaused(obj+0x94e)` and **returns entirely**
     when it is set [M 0x193600:88-91] — before input consumption, weapon work, movement,
     animation, death handling: the whole per-tick player body.
   - The use-key chain dies with it: the E-activation of triggers runs in
     `Player_Activate` [M 0x1a5170:135, case 0x1d → `Trigger_Activate`], whose only caller is
     `Player_WeaponFiring` [M 0x1a5460:402], inside the frozen update. So E at a load-level
     trigger cannot fire mid-hold.
   - The world does NOT freeze: `Game_Run` gates `SSys_Monitor` / `Light_Update` /
     `control_movement_object_handler` on the GLOBAL `GS_IsPaused(-1)` [M 0x1c9790:30-35], which
     reads the byte `GS_PauseGame` sets (0x2a37b8, [M 0x17d178 disassembly: `sb $a1, 0x37b8`])
     — and in the case-7 flow that global fires only when the hold ENDS (:127-129). Triggers'
     touch path (`Trigger_Update` 0x1b1a78:164 → `Trigger_Activate`) keeps running throughout.
4. **The port's gate matches this.** scene.rs's `mission_hold` (internal == 7) zeroes
   input/trigger/reload/use_key for the tick — Bond stands and cannot fire or use, while the
   world, the triggers and the HUD message tick continue, exactly the game's shape at the sim's
   fidelity. **No engine change was needed; the gate stays as the branch folded it.**

## The two tests were wrong, not the gate

Both failures were tests acting inside the frozen window — behaviour the game does not allow.

### `walk_07000013_live::the_front_entrance_completes_the_mission`

Reproduced on the merged tree: `assertion left == right failed: E at the front-gate exit …
left: None, right: Some(120586259)` (0x0730013). The test teleported Bond into the level's own
E8 #72 (gate 0x4E) volume and pressed E **during the 240-tick success hold** — the gate ate the
use key, so `level_exit` stayed unset.

The test was wrong twice over: the game cannot fire the E-activation mid-hold ([M] chain above),
and after the hold the case-4 `LevelEnd` hands the driving code to the port's end sequence
(scene.rs LevelEnd: `ptp_handover` + `mission_end = Some(next)`, `level_exit` nulled) — so no
post-hold press can land there either. Fixed by reordering to what a player can actually do:

- The E-press now happens **mid-mission** (internal 1, before the success walk). That is the
  game's own behaviour: the E arm falls through to the load with the 0x4E gate off
  [M 0x1b2318 case 10 — both the gate-on and gate-off branches load], so a player can request
  the driving interlude any time before completing the mission. The assert
  (`level_exit == Some(0x0730_0013)`) moved up with it.
- After the success asserts, the test drains the hold (steps while `mission.internal == 7`),
  steps case 4 in (`LevelEnd`), and pins the port's end-sequence tail:
  `mission_end == Some(0x0730_0013)` [M 0x182c60:65-76; scene.rs LevelEnd]. The
  `to_end == 0x0730_0013` pin [M :85-122] and every channel/objective pin are unchanged.

### `civilian_no_shoot_live::the_trigger_fires_under_the_mark_and_the_civilian_takes_the_hit`

Reproduced on the merged tree: `panicked at civilian_no_shoot_live.rs:126: a shot flies once the
aim is off (or the gun ran dry)`. The burst killed the civilian (10 health), which raises the
fail channel → internal 2 → the **300-tick fail hold** — and the test's tail then fired two more
presses "once the aim is off". The gate ate them (correctly).

The old tail asserted a shot the game cannot fire: the player is frozen for the whole fail hold
([M] chain above), after which the fail menu opens [M 0x182c60:77-81]. Per the task's rule the
TEST was wrong and is fixed with the addresses: the tail now aims off the body, asserts the
fail hold is live (`mission.internal == 7` [M 0x182c60:45-57, 123-130]), keeps the
"the mark clears when the aim leaves" crosshair pin (it evaluates on the current aim and passed
throughout), and pins the freeze itself — the clip does NOT move across the burst
([M 0x182c60:123-130 + 0x193600:88-91]). The no-latch fire evidence is unchanged elsewhere in
the file: "the trigger fires under the no-shoot mark" (line 111, the traced [L]
tests/traces/02-civilian.trace behaviour) and the guard test's live-state burst.

## Results

- `walk_07000013_live`: 2 passed / 0 failed (`--test-threads=1`, NF_ISO, NF_SILENT=1).
- `civilian_no_shoot_live`: 3 passed / 0 failed (same).
- The branch's own 04 tests, `walkreal_07000004_live` (incl. the kill→outro path and the hold
  pins): 4 passed / 0 failed.
- Full suite (fast runner, final tree = branch + origin/master): see the total line below.

## Suite (fast runner, final tree)

(to be filled when the run completes)

## Live checks for the main agent

None required by the fix (test-side only). Optional upgrade of the [M]-only freeze claim to [L]:

1. Fresh boot 07000002, shoot the civilian dead; during the "Mission Failed" hold: Bond must
   ignore stick/fire/use for ~5 s (300 ticks), then the fail menu appears. Watch
   `Player_Update` 0x193600 returning at the `GS_IsPaused` check (0x193600:88) and
   `MPGame+0` (0x2a4980) == 1 for the window.
2. Fresh boot 07000013, complete the entrance touches; during the "Mission Complete" banner
   (240 ticks): E at the front-gate exit must do nothing (no load, no fade), then the level
   transitions by itself.

## Usage

not visible to me
