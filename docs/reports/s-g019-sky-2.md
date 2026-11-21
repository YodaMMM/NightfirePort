# s/g019-sky-2: how the storm thunder reached the 0b roof fight

**First: this branch moves the shared random stream and turns 18 walks red on other levels. Keep it
on the branch.** The voice-line fix below changes when guards may bark, and the talk functions
draw from the shared stream. With it, 18 tests in nine walkreal binaries (01, 02, 03, 06, 08, 11,
13, 15, 16) go red. 17 of them pass on master (checked on a master build, same machine). The
master run of 01's following_mayhew_down_completes_the_level did not finish inside the 10-minute
cap, so that one baseline is unconfirmed.
The 0b walks that s/g019-sky broke are green again on this branch.

## The path from the thunder to the fight
1. The thunder is a 2-D `Sound_Play(100, 0x5d/0x5e)` [M 0x1e5ad0:104-107]. It takes a node of
   the 256-node sound pool, like every sound.
2. It is not heard by the guards. `Sound_Alertness` 0x1ddc48 would read a 2-D node (at the world
   origin), but 0x5d and 0x5e carry weight 0.0 in `SFXOutputData` [D ACTION.ELF 0x2c94a0 +
   id*0x18 + 0xc]. The port's hearing list does not get them either. Not the link.
3. The link is a stale voice handle. A guard keeps his line's handle in `Drone::voice` (+0x4D0).
   The port dropped a finished handle only inside `voice_busy` (the talk functions' busy test),
   so it stayed after the line ended. The pool reuses a freed node for the next sound
   (`Sounds::alloc`), and then `audio.finished(handle)` reads false again: the guard reads as
   talking, his non-forced lines are skipped (`speech`: `voice.is_some()`), his busy tests fail,
   and the talk draws behind them shift. A thunder that starts at a different tick changes which
   node each later sound gets, so it changes which guard is revived. Measured on 0b: with the
   gate on, thunder playing vs thunder muted, the sim state is identical until tick 14240, where
   guard #0's finished handle (node 19) is taken by another sound in the muted run only and he
   reads as talking again.
4. The game does not keep the handle. `NDrone2_ControlSTANDARD` 0x1490f0:82-94 reads +0x4D0 on
   every control pass and zeroes it once `Sound_IsFinnished` 0x1de240 says finished [M].
   `Drone_Control` 0x139268:48-61 runs it whenever `PreDroneControl` returns, and that returns 1
   on every path except the SetState arm [M 0x148db0:56/86/128]. `NDrone2_SpeechSFX` 0x141b00:105-106
   skips a non-forced line only while +0x4D0 is non-zero [M].

## Fix (logic)
- `Drones` per-drone pass: after `voice_finished` is refreshed, a finished line clears
  `voice` (crates/nf-game/src/drone.rs, beside the refresh).
- Test `drone::abseil_tests::a_finished_line_drops_the_handle_so_a_reused_node_is_not_talk`
  (nf-game lib). It fails on the old code (the handle stays after the finished pass) and passes
  with the fix. It also checks the sister case: a line that is still playing keeps its handle and
  stays busy.
- Proof that this was the link: with the fix, 0b's Agent walk dies at the same tick (17442), at
  the same spot, with the same hits, on master and on the sky branch. The sky gate no longer
  changes the fight.

## 0b walk re-tune (test only)
With the fix, `walk_fresh_spawn_to_the_mission_end_real_damage` (Operative) and `..._agent` died
at t17442 to the crouch-firing #5 (state 0x70, AIM_CROUCH_FIRE) at d 1.6. Guard #5 now strafes
in a crouch when Bond reaches him, and the first K5 dart (t17361) misses him. The K5 did not fire
again before he closed in.
- d1/d2: the opportunistic dart press waits for the pitch error to be under 0.03 as well as the
  yaw. This was enough for the Operative walk (mission complete t17777). The recording
  `tests/replays/0700000b_operative.bin` was re-recorded with NF_REC_0B_WRITE=1, and
  replay_parity_0700000b passes.
- d2 only: a 12-tick stand at the deck door before the sprint. Of 18 lengths tried between 0
  and 90 ticks, only 12 reaches the latch. This is a re-roll lever, the same kind other walks use.
- d3 (00 Agent) is unchanged and still green.

## Tests run (NF_SILENT=1, -j 4/8)
- nf-game lib: 907 ok. broken_object_bark_live 4, guard_barks_live 4, sky_view_gate 4,
  sky_slots_live 3, sky_flash_live 4: all ok.
- nf-view: walkreal_0700000b_live 8/8 ok. replay_parity_0700000b, _0700000a, _07000004 ok.
  escort_follow_07000001, first_sight_flag, fix_06_driver, fix_guest_reactions,
  hostage_killer_07000002, interactables_07000015, near_armed_person_07,
  speech_facial_fallback, sky_view_gate_live: all ok.
- Every walkreal_* binary (109). 100 binaries pass. 9 are red, 18 tests (17 confirmed green on
  master, following_mayhew unconfirmed):
  - 01: the_plate_refuses_bond_alone (the contact is killed by guard #3 and the level
    restarts), the_exit_fade_starts_on_the_exit_tick..., walk_from_the_spawn_to_the_exit_with_the_contact,
    following_mayhew_down_completes_the_level
  - 02: walk_the_castle_all_objectives_real_health_00agent
  - 03: walk_the_full_route_and_complete_every_objective_real_damage_00agent
  - 06: walk_the_castle_interior_to_the_party_real_damage_agent, ..._00agent
  - 08: walk_the_whole_mission_on_real_input
  - 11: walk_spawn_to_the_lower_floor_load_point
  - 13: walk_the_warm_chain_to_the_front_entrance_and_out_real_damage
  - 15: walk_spawn_to_the_omega_sector_exit, ..._agent, ..._real_damage_agent, ..._real_damage_00agent
  - 16: walk_the_chasers_defeat_completes_the_mission_on_real_damage_agent (listed twice by the
    harness), ..._00agent
  These need their own re-tunes before this branch can merge. The walks were tuned against the
  old stale-handle behaviour, in which a guard who finished a line was often kept quiet.

## Found, not changed
- Pool reuse order. The game's `DList_MoveFromFree2InUse` 0x1cccb8 pops the free list's head,
  and `DList_RemoveFromInUse2Free` 0x1cc868 appends the freed node at the far end. `DList_Init`
  0x1cc778 builds the list so the first allocations are nodes 255, 254, ... 0. That is FIFO reuse
  [M]. The port pushes new nodes 0..255 and then reuses the lowest free index. The in-use walk
  order (oldest first, `DAT_00349b0c` → +4) also differs from the port's index order. Both
  change handle numbers and the order of driver starts (and so the IOP `SFXrnd` draws per
  sample [M SFX.IRX StartSample 0x3524]). Not ported: it would shift every level again.
- A freed node is left in state 3 [M Sound_UpdateSounds 0x1dda70:107-111], and
  `Sound_IsFinnished` is true only for state 2. A stale pointer to a freed node therefore reads
  "not finished" in the game, while the port reads "finished". Drones poll every frame, so they
  always see the one state-2 frame. Other `finished` callers were not checked.
- The IOP rnd. Every sample start draws three `SFXsfrnd` (pitch, volume, pan) [M SFX.IRX
  0x3524], so any extra sound (thunder included) shifts the pitch and length of later lines.
  This is the game's own coupling and is ported as such. On 0b it did not change the outcome
  once the stale handle was fixed.

## Unproven
- No live capture. The ControlSTANDARD clear is [M] only.
- The port refreshes `voice_finished` at the end of each drone's pass. The game does this at
  the head of the control. The one-tick order against the sound update is not pinned.
