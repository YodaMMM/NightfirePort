# agent-07-door-guards

Based on `s/agent-lvl-07-08`. 07000007, the library door pair #404/#417 (CivilianDoorGuard 0x31).
No shared random stream change: the speech now goes through `speech_request` (no draw), and the
`ChooseSFX` draws are the same draws as before.

## What was wrong

The pair blocked a walking Bond (the previous branch's push) but never warned him or failed the
mission: both guards were 0xff mirrors. The earlier pairing let every guard be a candidate at boot,
so the scans chained 404→417→442↔441. 404 mirrored 417, 417 mirrored 442, and 442 mirrored 441.
The handler returns a mirror's partner's raw +0x2CC, which was 0xff, and the 0x31 UPDATE has no
arm for 0xff [M 0x1416d8:19-21, 0x160648:86-87]. A later "watch-across" deferral had been added on
top to keep the old walk order (plate #412 before the reveal) green.

## The game's rule

- The boot ENTER is `SM_InitObject` inside `NDrone2_PostLoad_Init` 0x14d890:21, after that
  drone's own `DefaultInit` :13. `Drone_PostLoad_Init` 0x136938:106-108 runs it drone by drone
  down the list. The list is in creation order (`LList_Add` 0x1cca28 appends at the tail; the
  drones are created in placement order).
- A drone not yet reached is still the zeroed create block [M control_create_object 0x1344e0:17]:
  mode +0xC5 = 0 and no `+0x4F8 & 0x100`. DefaultInit writes the flags at 0x14b300:60 and the mode
  through DoModeSettingsNEW at :501. So `SetDronePartner`'s scan [M 0x141360] at a boot ENTER sees
  only the guards ahead of the caller.
- On 07000007 (list order 404, 417, 441, 442):
  - 404 finds no one.
  - 417 pairs 404. `Vec_AngleDifference` is b − a [M 0x1e2b78], and atan2(417→404) − yaw(417) ≈
    1.59 − 3.11 < 0, so 417 takes 0xff and the planes go on 404 with the normal toward the hall.
  - 441 finds both claimed.
  - 442 pairs 441 the same way (planes on 441, normal west). This pair is unchanged.
- 404 now evaluates and 417 mirrors him:
  - zone 2 (within 2.0 in front of the gap): anim 0x5f, and 404 speaks line 0x4f/0x50.
  - zone 3 (past the line, within 6.0): line 0x8f..0x92, mission fail reason 0xc (FailLabel
    0x4000033, ch 99), and Attack [M 0x160648:73-94].
  - An armed Bond still turns them hostile first, through `NearArmedPerson` [M 0x160648:39-57].

## Changes (crates/nf-game/src/drone.rs)

- The pairing request carries the ENTER's frame. Each drone records `post_load_frame` (its first
  ENTER). A candidate is eligible only if it was initialised before the caller's ENTER: an
  earlier frame, or the same frame and earlier in the list. The deferral (`door_pair_watch`) and
  the watch-across rule are removed.
- The zone 2 and zone 3 lines go through `speech_request` (`SpeechSFX(dc, line, 0, 1, 1)`), not
  the raw sound queue. The game's argument is a speech-table line, not a sound id
  [M 0x160648:78-79, 89-90].
- Zone 0 turns the guard back to the placement facing (`+0x694 = +0xC24`, new `watch_yaw`)
  [M 0x160648:99]. A 0xff zone does nothing.

## Tests

- New `library_door_guards_warn_07000007_live`, two tests:
  - The real walk from the party start, then an unarmed Bond walks up to the gap: 404 speaks
    0x4f/0x50, 417 plays the challenge, nothing fails.
  - Bond is put 0.5 past the pair's line after the real walk: ch 99, FailLabel 0x4000033, line
    0x8f..0x92.
  - Both fail on the old code: no line, no ch 99.
- `door_guard_live` (nf-game): the pair graph is re-pinned to 404↔417 and 441↔442, with planes on
  404 and 441 and the normals pointing south and west.
- `interact_07000007_live`, `walk_07000007_live`, `spots_07000007_kimiko_trigger_fires` and
  `fix_party_mission_live` (the guards-reappear test) teleported Bond onto plate #412 before the
  reveal. That plate is inside the pair's zone-3 band, so they now play Drake's reveal first: Bond
  is put on the reveal plate #356, the fly-through is skipped, and ch 21 takes the pair off. That
  is the real path's order (see agent-lvl-07-08). The shared helper is
  `crates/nf-view/tests/common/reveal07.rs`.
- Green: library_door_07000007_live, near_armed_person_07_live, civilian_first_sighting_alert_live,
  walkreal_07000007_live (5 + 1 ignored), cutscene_timer_pause_live, fix_death_loop_07_live,
  interactables_07000007_live, solid_objects_07_08_live, party_door_lock_07000007_live,
  interogate_07_live, the nf-game lib (688), fix_guest_reactions_live, consider_alerted_live,
  civilian_mission_wait_live, fix_npcs_live, party_start_rules_live, urn_break_live,
  use_key_census_live, npc_doors_live, fix_actors_live, guard_stuck_sweep_live,
  enemy_mission_kind8_live, speech_facial_fallback_live, fix_drone3_live, fix_drone4_live,
  head_track_trace_live, split_runs_identity_live, script_channel_events_live.

## Unproven

- Not checked against PCSX2. Questions for the real game, fresh boot of 07000007:
  - Are #404/#417's +0x2BC/+0x2CC 417↔404 / 0 on 404 / 0xff on 417?
  - Does an unarmed Bond in front of the gap hear a 0x4f/0x50 line?
- With the push holding, zone 3 cannot be reached by walking before the reveal. Whether the
  game's push is as total is a live question.
