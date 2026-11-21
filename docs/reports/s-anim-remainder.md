# s/anim-remainder: GAPS G-020 / G-030 re-check

**Moves the shared random stream (stays on the branch):** party guests on 07000007 now draw
`Rand(4)` + `Rand(5)` each time a stand-idle hold runs out.

## Player-visible
The party guests on 07000007 (PartyGirl, state 0x2f, property 0x47) now cycle through the four
stand idles 0x24..0x27, each held 5-9 s, as the game does. Before this they all stood on 0x24 the
whole time. Model-0x10 drones now always ask stand idle 0x26.

## What was checked against the decompile
- `DroneAnim_SetStandIdleAnim` 0x13ca68, read whole [M]. Ported in `Drone::set_stand_idle`
  (crates/nf-game/src/drone.rs):
  - the hold-clock early return applies in every state (:19-21);
  - Idle `0x24+Rand(3)`, PartyGirl+prop 0x47 `0x24+Rand(4)`, hold `(5+Rand(5))*60` (:31-42);
  - every other state 0x24 with hold 0 (:27-28);
  - model 0x10 → 0x26 (:46-48);
  - range/can-play fallback (:49-68);
  - NOT ported: the game's unconditional `CallAnim` (:70). The port keeps its older guard:
    `request_anim` for the no-hold states, and no call while one is pending in the random arms.
    The always-call form broke `walkreal_0700001b_live` (3 real-health drake walks) and
    `walkreal_07000006_live` real_damage_agent, which all pass on master. I measured this by
    swapping the master drone.rs in and out. Those walks need re-tuning before the faithful form can land.
- All 16 game call sites were checked for the respect argument. Fixed: PartyGirl UPDATE
  0x160140:42 (1, was forced every tick), CastleChatGuard1 0x1612e0:28/:63 (0),
  SpaceDrake 0x16dee0:20/:48 (0; ENTER/ANIM_DONE now go through `set_stand_idle`; the
  UPDATE arm's channel-off `SetStandIdleAnim(1)` [M :28-33] is still not added: it would cancel the
  look-around call every tick, and that was not tested), and
  patrol route-done 0x1514d8:13 (1; it used `request_anim`).
- `CallAnim`'s 0x25..0x27 → 0x24 fallback was already ported.
- Drone mode `+0xC5`: the sim carries the real mode (`Drone::mode`). The "not modelled" note
  in drone-animation.md applies only to the nf-formats disc census; reworded.
- docs/systems/anim-script-rest.md: every entry is still accurate. Its one unported branch is the
  facial speech path (`NDrone2_SpeechSFX` 0x141b00 → `AnimScriptPeekSfx` / `AnimFacialAppend`).
  Facial morphs are drawn, so it is not done here.

## Not done / unproven
- `+0x538` blend-out length for non-call anims: this is blend timing (motion feel), so it was not touched.
- Normals: these are draw-side (the renderer already skins them). Not re-checked.
- The sniper start switch (mode 2 / 0x19 with anim set 5 → set 0x12, `NDrone2_DefaultInit`
  0x14b300:646-656): I could not find it in the sim. It is not ported or tested here.
- Follow chain: `AllyFollowDone` 0x15f858 ENTER is `AlertStatusSet(1)` + `CallAnim(0x21)`, but the port
  plays the stand idle. `AllyFollow` ENTER is `CallAnim(0x1e)`, but the port plays the walk anim. Both are left as they were.
- `update_mission_route`'s shared blocked arm passes respect 1. CivilianMission 0x15d860:48 and
  TruckDriverMission 0x161030:46 pass 0. This only matters if a hold is left over from Idle/PartyGirl. Left as is.
- No PCSX2 check. A live read of guest `+0x59c` and the anim state on 07000007 would confirm the
  hold lengths.

## Tests
New test `stand_idle_variants_live`: 2 tests, both fail on the old code, both pass now.

