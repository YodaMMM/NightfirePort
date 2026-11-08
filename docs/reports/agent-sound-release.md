# Headless sound ends by length; the guard body-fall sound (s/agent-sound-release)

**This branch moves the shared random stream.** With the guards' lines now ending, the talk
functions (`voice_busy`, `Drone_IsTalking` 0x149798) stop refusing and draw from the stream; the
first extra draw on the 07000011 route lands at tick 105. 14 real-health walks are red (list below).

## What changed

- `nf_audio::VoiceClock`: the mixer's voice timing without the mix. A `Sound` with no output
  device (`output: None`) now reads its end flags from it instead of never getting any: each
  voice runs `pitch / 0x1000` samples per 48 kHz tick, one game frame (800 samples) per batch,
  with the key-on delay, the loop wrap and the 192-sample key-off release. A unit test feeds the
  same commands to the real `Mixer` and the clock and checks every voice ends on the same frame
  (one-shots at three pitches and a delayed key-on, a key-off; a loop and a pitch-0 voice hold).
- `nf_view::attach_headless_sound(scene)` puts a deviceless driver (the level's banks and streams,
  no music stream) on the sim; `Sim::step_bond` runs `sound_tick` after each step, the app's
  order. The music command queue is left for the caller, as before. The walkreal walkers
  (`walker.rs`, `walker_b2.rs`) attach it in `Walker::new`. Before, a headless run never updated
  the EE pool: no node ever ended, so every line a guard spoke kept him "talking" until the
  256-node pool filled, after which every start returned handle 0.
- The body-fall sound: op 4 event 6 in a drone's anim script (`AnimProcessScriptCmds` 0x1c1b38
  case 6, `Effect_Create(1.0, obj+0xdc, obj, 0xc0)`) → `Effect_Body` 0x1cf400: the floor's body
  sound (`EffectInfo[mat]+0x18`) at the contact, heard by the drones at half weight
  (`Sound_ModAlertness(0.5)`). The contact is the footsteps' feet-ray stand-in [S]. No ScriptCam gate
  (the call is the command's own). The MP `MPSettings+0x180` gate is not modelled (SP only).

## What a player notices

Shot guards make a thud when their body lands (the 05 start guard's death script played it twice, 16 ticks apart).
The deviceless run (no audio device) no longer piles up sounds that never end.

## Tests

- `guard_body_fall_sound_live` (new, 2): the 05 start guard's death plays body id 0x135 at his feet
  at half weight (heard on ticks 73 and 89 after the kill); a fired P2K shot's node is freed 59
  ticks after the shot. Both fail on the old code (no body sound; the node never freed).
- `nf-audio` lib (25): pass, with `the_no_device_clock_ends_voices_on_the_mixers_frame`.
- `cutscene_sync_live` (2): the later dialogue lines of the 02 and 08 intros used to never start
  in a deviceless run (both stream slots stayed full forever), so the sync check never compared
  them. They start now, and the test measures each line's real key-on (the captured `Play` matched
  by sample hash) instead of its first item, which can be the precache. Every pair is 0 ticks off.
  A line keyed on by the setup's first sound tick is skipped (its start belongs to the setup).
- The sound binaries that build a `Sound` (21 binaries) all pass.
- walkreal: 87 of 96 binaries green. Harness fixes made for the new timeline: 0c's wire #283
  grab line, 0c's laser press waiting out the sniper's reload, 06's pilaster lane chosen from
  where Bond really stands, 07000011's flight-C band (32.51 on the last step), 01 Agent's return
  fire holding its line (no back-away). 07000009's music asserts are green again now that the
  music queue is left in place.

## Red: real-health walks on the new stream (14)

- 07000001 `..._real_damage_00agent`: the start-room fight (guard #0 side-steps instead of
  standing mid-line; 91 health lost to #0 and the SMG #13) then dies at the hall door (tick 3777).
- 07000002 `walk_the_castle_all_objectives_real_health_agent`: the computer's ch 0x48 never rises.
- 07000003 `..._real_damage_agent`: wedged on the gate-plate leg after a servant leg.
- 07000005 `..._walked_real_damage_00agent`: dies at tick 9776.
- 07000006 `..._party_real_damage` and `_agent`: past door #59 now; both die in the courtyard (the
  Agent run arrives with clip 0 spare 0 after a failed resupply).
- 07000008 `..._real_damage` and `_00agent`: die at ticks 25079 / 28314. With the body sound alone
  on the old pool, `..._real_damage` and `_agent` were red; `_agent` is green now.
- 0700000c `..._0d_handoff_real_damage`: the deck sentry #20 outlives his first stand; Bond
  reaches the yard on 17.6 and dies there.
- 0700000d `walk_arrival_to_the_c5_door_real_damage`: the west-roof sweep takes three passes
  (g29 shoots 30 on the way), the bay sniper #18 32 more; the slot drop lands at 0.
- 07000013 `..._real_damage` (already red with the body sound alone) and `_agent` (a 6000-tick
  wedge on the south-east bend).
- 07000015 `..._real_damage_agent` and 07000016 `..._chasers_defeat_..._real_damage`: deaths.

These are tactics written against guards that froze once they had spoken. They need the batch
re-pin that random-stream branches get.

## Unproven

- The 161 non-walkreal binaries that build a `Walker` also run the headless driver now. They were
  not run (no full suite). Expect timeline shifts in the long ones.
- The clock advances one 60 Hz frame per batch. During a level build (FRAME_RATE 30) a batch
  covers two game frames on the PS2. Headless runs batch once per step either way.
