# Guard talk, sight barks and object cleanup

The 25 never-mentioned guard-AI functions (backlog `unported-all-2026-10-20.md`, guard-ai-other
1/6): where each lives in the game, whether the shipped game reaches it, and where the port now
carries it. Evidence tags: [M] decompile/disassembly (file + line), [D] disc, [L] live PCSX2,
[S] port choice. Callers were grepped over `work/decomp/ACTION.ELF` (mangled call names) and the
ELF was scanned for `jal`/`j` encodings and absolute data words of each address.

## The engine's class-delete dispatch (shared context)

`control_funcs` (symtab: 0x26F890, 1020 bytes) is the per-object-class op table: `+0` update
(`control_movement_object_handler` 0x1336F8:62-63), `+4` collision (`Collide_Update` 0x1EA398:59-63),
`+8` delete (`control_delete_object` 0x133178:16-17) — each `control_funcs[class*0xC + n]` [M].
Drone-class entries: `Drone_Control` 0x139268 @0x26F8A8, `Drone_CollisionHandler` 0x140968
@0x26F8AC, `Drone_Delete` 0x136CE8 @0x26F8B0; a second drone-ish class repeats
`Drone_Control`/`Drone_Delete` @0x26F95C/0x26F964; `DroneSpawner_Delete` 0x13A338 @0x26FAD8
(class 48); `Drone_AIVolume_Delete` 0x13A1D0 @0x26FB80 (class 62) [M ELF words + symtab name].
The dispatch is on every level's object teardown — these deletes are reached wherever their
objects exist.

## Per-function outcomes

| function | verdict | where in the port |
|---|---|---|
| `0x1291E8` `NDrone2_initDTYPE_BotInit` | cited (empty body) | `drone.rs` GetDTYPENEW init dispatch (mode-row `init` name key) |
| `0x136CE8` `Drone_Delete` | cited (replaced) | `drone.rs` FADE/DEAD removal (`present = false`) |
| `0x13A0F8` `Drone_InitOnce` | cited (empty body) | `drone.rs` `Drone_bDisableSystem` fold |
| `0x13A1D0` `Drone_AIVolume_Delete` | cited (replaced) | `aivolume.rs` (`build` Vec — no runtime deletion) |
| `0x13A1F8` `Drone_VisibilityForPosition` | cited (replaced) | `aivolume.rs::visibility_factor` |
| `0x13A338` `DroneSpawner_Delete` | cited (replaced) | `drone.rs` spawner-setup block |
| `0x13A578` `Drone_GetSkinHash` | cited (site recorded; feed pending) | `triggers.rs` 0xE hash test (`Toucher.skin`) |
| `0x13A5A8` `Drone_LevelReset` | cited (replaced) | `drone.rs` `Drones::new` + `target_registry` |
| `0x13A878` `Drone_FellOutMap` | cited (replaced) | `drone.rs` DeleteMe arm (`Enable(0)` + `obj+0xFE \|= 1`) |
| `0x13DEF8` `DroneAnim_SetAnimCall` | cited (replaced) | `drone.rs` `DroneAnim_CallHandler` fold |
| `0x13E230` `DroneAnim_SnapRotate2` | cited (empty body) | `drone.rs` `DroneAnim_SnapRotate` doc |
| `0x13E3E8` `DroneAnim_AnimEndCallback` | cited (replaced) | `drone.rs` HT-anim end (`+0x579 = 0`) |
| `0x13E770` `NDrone2_PowerstationTalk` | **ported** | `drone.rs` `Drone::powerstation_talk` (from `handle_talking`) |
| `0x13F420` `DroneFunc_NewSightTalk` | **ported** | `drone.rs` `Drone::new_sight_talk` (+ `Drones` latches) |
| `0x13F550` `DroneFunc_LostSightTalk` | **ported** | `drone.rs` `Drone::lost_sight_talk` |
| `0x13F688` `NDrone2_HandleTalking` | **ported** | `drone.rs` `Drone::handle_talking` (tick hook) |
| `0x13FB98` `DroneFunc_UnderCoverAimTalk` | unreached-effect | `docs/port-map/unreached.md` (dead in both modes) |
| `0x141150` `NDrone2_GetHeadPosOld` | cited (replaced) | `drone.rs` `Drone::head` (+0.6 fold) |
| `0x144340` `NDrone2_MonitorTalkToMissionObj` | already cited | `drone.rs` 356/3366 (pre-existing, name + 0x144340) |
| `0x147578` `NDrone2_DoHeadTrackingBehaviour` | cited (replaced) | `drone.rs` head-tracking block |
| `0x149798` `Drone_IsTalking` | cited (replaced) | `drone.rs` `Drone::voice_busy` |
| `0x149CA0` `NDrone2_SetupGroups` | cited (replaced) | `drone.rs` spawner/template setup |
| `0x149D58` `NDrone2_SpeechSFXd` | cited (replaced) | `drone.rs` `voice_busy`/`talks.push` request form |
| `0x14A808` `NDrone2_SetupHostageExecute` | not ported (pending) | noted at `drone.rs` `+0x2B0` captor link |
| `0x14AFF8` `NDrone2_PostDroneControl` | cited (replaced) | `drone.rs` NDrone2_Steer block |

## The ported talk flows

All four hang off the port's existing speech request path (`Drone.talks` → `Drones::speech`,
docs/systems/speech.md §1). The per-drone walk in `Drones::tick` runs the game's two callers:
`DroneVision_ProcessDroneSight` 0x176A28 (the sight barks) and `NDrone2_ControlSTANDARD`
0x1490F0 (`HandleTalking`), with the global latches (`0x298FA8`/`0x298FA9`), the AFO throttle
(`counter_210`) and the powerstation last-channel (`DAT_00298FCC`) on `Drones`.

- **`NDrone2_HandleTalking` 0x13F688** — level 0x0700008: type-0xc guards within 3 units of the
  player run the 0x9b → 0x9c → 0x9d → 1… script (force 1, `+0xCF4 = now + 5 s / 15 s`), the
  switch-channel 0x4a once-line 0x9e finishes it (script = 99), `counter_210` paces at 30 [M
  0x13f688:29-96]. Levels 0x070000c/0x070000d dispatch to PowerstationTalk [M :24-27]. The
  `NDrone2_CanSeeObject(player,5,0)` gate is the port's `los` stand-in [S fold].
- **`NDrone2_PowerstationTalk` 0x13E770** — busy/health gates, then `switch_channels[0x98]`
  enables the chatter; the ELF's static table at 0x29AB20 holds 17 records
  `{u16 ch(0c), u16 ch(0d), u32 id}`, ids 216..232 [M ELF bytes 0x29AB20..0x29ABA8; the decomp
  loop reads id at rec+4 and stops at the next record's zero id]; a guard speaks the record whose
  channel column names its channel and whose channel is on and is not the global last-played one;
  0c remembers the first column, 0d the second [M 0x13e770:44-72]. The port pins the table as
  `Drone::POWER_SFX`. The `+0x4F8` 0x600/0x100, class ≠ 0x11 and fell-out arms are folded [S].
- **`DroneFunc_NewSightTalk` 0x13F420 / `DroneFunc_LostSightTalk` 0x13F550** — on 0x070000c/0d
  for type 5/6 guards: NewSight when a drone has held sight 60+ frames with the fa8 latch clear
  (sets fa8, clears fa9), line `0xd6 + Rand(2)`; LostSight when unsighted 120+ frames with fa9
  clear and nobody having seen the opponent last frame (sets fa9, clears fa8), line
  `0xd3 + Rand(3)` [M 0x13f420/0x13f550; speech.md §2.3]. The port's `seen_ticks`/`unseen_ticks`
  stand in the game's per-drone sight timers [S fold]; the LostSight "the lost target is the
  player" check is a fold (SP drones only track Bond) [S]. `DroneFunc_TargetSeen` follows the
  play in the game [M :36-40]; the port's sight bookkeeping already covers its job.

Determinism: the only new RNG draws are the barks' own `Rand(2)`/`Rand(3)` — the game draws them
in the same gated branch, and the gates (levels 0c/0d, types 5/6, busy/health) keep every
existing test level's draw sequence untouched.

## Tests (`nf-game` lib)

- `afo_approach_dialogue_first_line_and_range_gate` — 0x9b + cooldown + sister (range gate).
- `afo_dialogue_cycle_once_line_and_gates` — the cycle, the 0x9e once-line, and the
  script-99 / cooldown / throttle sisters.
- `powerstation_chatter_table_and_gates` — table records (53→220 on 0c, 70→216 on 0d), the
  last-channel skip and the 0x98 master gate sisters.
- `sight_talks_gates_and_id_shapes` — id shapes and the level/type/health sisters.

## Not done here

- `NDrone2_SetupHostageExecute` 0x14A808: the port has no `HostageKillerAttack` state to hang it
  on; the mutual-position/yaw setup (+0x670/+0x694) is documented at the `+0x2B0` captor link and
  left for the hostage-execution port (docs/systems/speech.md).
- `Drone_GetSkinHash` 0x13A578: the trigger-side hash test exists (`Toucher.skin`), but the port's
  drone toucher build does not feed the hash yet (SP drone touchers are built without `skin`).
