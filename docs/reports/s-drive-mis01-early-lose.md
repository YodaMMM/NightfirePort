# s/drive-mis01-early-lose

## What was wrong
The driving view's MIS01 run failed at tick 424 with no input. Trace: trigger 644 on the
helicopter's flight path runs `ECallStage(42)`; stage 42's rule 5 (`SRuleDifficulty` mask 6,
Agent and 00 Agent [M 0x20aac0]) queues `EKillAIElement`, `EPlayerLose`, `EDisableTrigger(712)`
[D]. Rules 45 / 85 / 105 / 107 in stage 19 (either pursuer blown out or dead) disable 644 first
[D]. So the lose itself is the game's rule on Agent: the pursuers must be stopped before 644.

The view got three things wrong around it:
1. The spline ran at rate 1.0. The spline controller's path element rate is 0.5
   (`FUN_0013da80` +0x58 = 0x3f000000, copied to the path player's +0xa4 by `FUN_001b1110`;
   `FUN_0013dcc0`'s reset leaves it) [M]. The helicopter reached 644 at tick 425; now 849.
2. The car's velocity on the spline mixed the land step into it ((q - o) x 60 with `o` already
   moved by the body step): it read (64, 94, 99) or (-98, -94, -65) on alternate ticks while the
   car moved about 0.55 u a tick. The vehicle's state-3 update `FUN_00131620` copies the path
   player's pose and velocity [M]; the view now skips the body step while the spline owns the
   car, so the velocity is the path step x 60 (the one-way trigger test and the script's facts
   read it).
3. The difficulty was fixed at 1 (Agent). The front end's pick now rides into the drive
   (`load_driving_scene_at`, PTP+0x984 mapped 1 -> 0, 2 -> 1, 3/4 -> 2 [M 0x174bf0:70-81]); the
   plain `load_driving_scene` keeps the boot default (2, Agent [M 0x1c7e78:25]).

Also: `ESetNextAISpline` (0xd1) for the player sets the path player's next animation
(`FUN_00158370` -> `FUN_0013dd28` -> `FUN_001b1838`, +0x9c) [M]; the helicopter now ends on
`path_Heli_Hover` at the end of `path_Helicopter02` instead of wrapping to the start.

## Tests
- New `crates/nf-view/tests/driving_mis01_opening_live.rs` (3): Agent fails on the tick 644
  fires, 644 inside 800..=960; Operative flies 4000 ticks with no lose and ends at
  (-143, 542); the spline velocity equals the position step x 60. Fails before: 644 at tick
  425, the velocity off by 1.67 u a tick.
- `crates/nf-game/tests/driving_intro_disc.rs` re-pinned for the rate: MIS01 letterbox off at
  frame 18 (was 10), MIS13B camera end at frame 258 (was 130).
- nf-view driving tests: 17 binaries, 20 passed. nf-game drive*/driving* tests: 39 binaries,
  286 passed. nf-game lib `driving::`: 258 passed. nf-view lib `driving::`: 3 passed.

## Not proven
- Whether the body's generic integrator still runs under the state-3 spline update (the pose
  is overwritten each frame either way) [S].
- The view has no sniper gunner yet, so on Agent the player cannot stop the pursuers and the
  lose at 644 always happens there. In the game the player shoots during this whole section;
  there is no point where control passes to driving before 644.
- Whether the pursuers can die or blow out without the player's shots (an AI crash) is not
  traced; a live break on `FUN_0022c890` for trigger 644 on Agent would settle it.
- `ESetNextAISpline` with no controller in place (the game builds one) is not taken.
