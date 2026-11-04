# audit-music-events — every Music_Event call against the port

The helicopter's attack beat was missing until its state machine was ported, so sweep the class. List every Music_Event and music-control call in ACTION.ELF for on-foot play: the event number, the argument, the condition, and the port site. Drones, the alarm, copters, turrets, objectives and level scripts all count; driving is out of scope. Fix every missing or wrong call that has a clear port site and pin each with a test; report the rest as a table. Merge origin/master first.

---
claimed-by: sandbox1 at 2026-11-04T21:33Z

---
## Report (done, 2026-11-04T22:24Z)


---
## Report from the worker (done, 2026-11-04T22:25Z)

# audit-music-events — every Music_Event call against the port

Branch `s/audit-music-events` (cut from origin/master 8c9f02b03). The music system is a
64-slot event mailbox: `Music_Event(id, value)` writes `EventList[id]` (base 0x2d34f8 —
confirmed by four independent caller/consumer pairs: ch0=0x2d34f8 track request, ch1=0x2d34fc
gate, ch2=0x2d3500 combat, ch7=0x2d3514 fail), `Get_Music_Event(id)` reads it, and
`UpdateMusicalEvents` 0x114e70 consumes channels 0–12 each tick and then zeroes all 64 slots
(BackupEventList/LastEventList are per-frame copies). Channels ≥ 13 are read by the per-track
`UpdateMFX_*` functions, not by the core dispatcher.

## The audit table (every caller in ACTION.ELF, on-foot scope)

"port site" = where the port makes the equivalent call. Census: 29 write sites + 1 read site;
all args intact in the decompiled C (no NEEDS-DISASM).

| decomp caller | call | condition (plain words) | port site | status |
|---|---|---|---|---|
| Sound_Ready 0x1dceb8 | `Music_Event(0, map id)` + `Music_Event(1,1)` | unconditional, end of the sound-bank load | nf-view/src/lib.rs:508-509 (`wire_music`) | ported |
| Player_CheckForDeath 0x190810 | `(6,1)` | dying, health ≤ 0, SP only | nf-game/src/death.rs:234 | ported |
| Mission_Update 0x182c60 | `(8,1)` complete / `(7,1)` fail | switch_channels[100] set / [99] set | nf-game/src/drone.rs:17562 / :17561 | ported |
| Mission_MonitorObjectives 0x1827d8 | `(8,1)` | all objective checks true → sets switch_channels[100] | nf-game/src/drone.rs:13139 (`tick_mission_during_hold`) | ported |
| Player_Creep 0x1969d0 | `(0xd,1)` | every tick while crawling | nf-view/src/scene.rs:3452 | ported |
| Player_Wire 0x197658 | `(0xd,1)` | every tick on a wire | nf-view/src/scene.rs:3446 | ported |
| Trigger_Activate 0x1b2318 | `(3,1)` | trigger type 10 (level exit), gate chain satisfied | nf-game/src/drone.rs:16481 + :19141 | ported |
| MusicTrigger_Update 0x1b31a0 | `(param 2 u16, 2)` | 30-tick countdown running (channel-configured) | nf-game/src/worldobj.rs:571 | ported (+tests) |
| Script_CameraStart 0x1d9140 | `(0xb, NIS id)` | a script camera actually started | nf-game/src/drone.rs:16563 (Request::Camera start) | ported |
| Script_KillStream 0x1d9860 | `(0xc, NIS id)` | case 4 (scripted camera stream ends) | nf-game/src/drone.rs:16563 (Request::Camera end) | ported |
| Script_FFwd 0x1dac78 | `(5, NIS id)` | first skip press on a camered script | nf-game/src/drone.rs:16795 (Request::Event 0x40) | ported |
| SP_Update 0x1dbda8 | **read** `Get_Music_Event(2)` | NIS 0x6000994 start refused while a combat event is live | nf-game/src/drone.rs:21211 + :19441 (`prev_event2`) | ported |
| NDrone2_SeenAndAttacking 0x145150 | `(2,5)` | sustained/multi-attacker combat (AttackGate) | nf-game/src/drone.rs:19805 | ported |
| DroneFunc_CheckAlarmRaised 0x1453a8 | `(2,3)` 05 ch 0xA4 / `(2,1)` 06 ch 0x37 / `(2,1)` ch 0x61 | one-shots in the first frames after each alarm switch | nf-game/src/drone.rs:17796-17797 | ported |
| Copter_Update 0x17dbb8 | `(2,1)` | every tick of the attack state (case 2) | nf-game/src/drone.rs:18740 (CopterEvent::Music) | ported |
| GT_Update 0x180498 | `(2,1)` | turret state 4 with a target | nf-game/src/drone.rs:17520 (GTEvent::CombatSting) | ported |
| **Script_Play 0x1db588** | `(6,1)` | a camera NIS (`+0xAA8 & 0x40`) whose hash ∈ {0x6000952, 0x600098a, 0x6000991} = the three death NISs starts | **nf-game/src/nis.rs `load()` → Request::Event{kind:0x41} → drone.rs drain `music.event(6,1)`** | **FIXED this branch** |
| **NDrone2_DSTATE_HostageSaved 0x15ccf0** | `(0x10,1)` | hostage-saved ENTER, unconditional | **nf-game/src/drone.rs HOSTAGE_SAVED arm → `music_reqs` outbox → tick drain `music.event(0x10,1)`** | **FIXED this branch** |
| **DroneVision_HaveOpponentSight 0x175ef0** | `(0xf,1)` | first-ever sighted guard (prop 0x3c, `+0x228 & 8` rising) is in mode (+0xc5) 4 | **nf-game/src/drone.rs sight pass (`ever_seen_3c` rising + `dr.mode == 4`)** | **FIXED this branch** |
| **Drone_InitComms 0x139520** | `(0xe,1)` | the all-clear scan: no enabled drone has alert status (+0x44) exactly 1 | **nf-game/src/drone.rs InitComms block, before CheckAlarmRaised** | **FIXED this branch** |
| Drone_EnableAll 0x1383e8 | `(0xf,1)` | drones enabled AND the level/hash arg == 0x600001b | not ported | **unported** — one script-hash-gated site; that script is not live on the ported levels (the `DroneEnable` request doc already notes both hash extras) |
| Player_Update 0x193600 | `(cel+0xa1, 1)` | every tick inside a cel whose type 0xc023/0xc047 entity carries prop 0x3c = the music-zone channel (dynamic ch, e.g. 44 on 0x7000000b) | not ported | **unported** — needs the cel prop-0x3c parse + a player-in-cel lookup; see "what's left" |
| Sub_DeployTorpedoes 0x1acf68 / Sub_Update 0x1ad200 / MiniSubState_Attack 0x181b00 | `(2,1)` | sub/mini-sub near player or firing | not ported | **driving — out of scope** (recorded for completeness) |

Notes from the census:
- ch2's values are richer than the dispatcher reads: 1 generic combat, 2 immediate+600-bonus
  (music triggers), 3 the 05 alarm one-shot (specifically read back by `UpdateMFX_01CastleA`
  as `DAT_002d3500 == 3`), 5 sustained multi-attacker. The port models all four values.
- ch11/ch12 carry NIS hashes, not booleans (`UpdateMFX_03TowerC` compares 0x60004f9/0x60004fa);
  the port's Request::Camera start/end already carries the hash.
- ch14 (DAT_002d3530) has **no reader anywhere in ACTION.ELF** — a write-only channel. The
  fix is board-fidelity only.
- ch16's readers are MayhewB 0x112270:97 / MayhewC 0x112730:42 (`ch15 ∥ ch16` gates the
  combat pick while MusicTemp1 is latched). ch17 (DAT_002d353c) is read by TowerC case 3 —
  a dynamic channel (level 0b's second music zone), not the hostage-saved event (the earlier
  census note placing hostage-saved there was an off-by-one: EventList[16] = 0x2d3538).
- EventList base is 0x2d34f8 (the 64-word zero-loop runs from DAT_002d35f4 down), so
  0x2d3538 = ch16 and 0x2d353c = ch17.

## Fixes on this branch (four missing producer calls)

1. **Script_Play → `Music_Event(6,1)`** (0x1db588:26-41). Port site: `SP_LoadScript`'s port
   (`Scripts::Player::load`, nis.rs) — every mode arm plays the script through `Script_Play`,
   so the post sits there, gated on the script's camera flag (`nis.camera` → flags bit 0x40)
   and `Script::is_death_nis`. The request rides `Request::Event { kind: 0x41 }` and the
   drain maps it to `music.event(6, 1)`. The board's event-6 consumer is the on-death marker
   jump + `FadeDown(false)` + `wait_for_end` (already ported + pinned by
   `the_09_death_and_fail_events_keep_their_own_markers`).
2. **HostageSaved → `Music_Event(0x10,1)`** (0x15ccf0:20-21). New `music_reqs: Vec<u8>`
   outbox on `Drone` (the `sfx_reqs` pattern), pushed in the HOSTAGE_SAVED ENTER arm, drained
   straight onto the board in `Drones::tick`.
3. **HaveOpponentSight → `Music_Event(0xf,1)`** (0x175ef0:88-92). In the sight pass, on the
   `ever_seen_3c` rising edge (prop 0x3c + first sight) with `dr.mode == 4`. The game's
   self-msg 0xf/10 pair in the same block stays unported (not music).
4. **Drone_InitComms → `Music_Event(0xe,1)`** (0x139520:163-196). The all-clear scan, placed
   before CheckAlarmRaised (the game's order). The `+0x4F8 & 0x600` skip is the port's
   enabled/non-corpse encode; the `+0x4F8 & 0x100` flagged-aware-near arm is not modelled
   (the port carries no 0x100 bit) and marked [S] in the code — unobservable either way, as
   ch14 has no reader.

**Test**: `nf-game/src/nis.rs::the_death_nis_start_posts_the_death_music_jump` — the death
NIS 0x06000952's SP load posts the kind-0x41 request; a non-death camera NIS posts none.
The producer→board links for ch14/15/16 are pinned by construction (single drain sites) and
the board semantics by the existing music.rs tests (17 green, including the event-6 consumer).

## Unported consumers (tabled, not fixed — each needs work beyond this task)

| piece | levels | blocker |
|---|---|---|
| UpdateMFX_02MayhewB 0x112270 / MayhewC 0x112730 (the ch14/15/16 + channel-driven section machines) | 0x7000002 / 0x7000003 | 1212/1224-byte state machines reading MapMusic marker fields +0x14/+0x18/+0x24/+0x38 (5-entry last-jump scans) the MapMusic parser doesn't read |
| UpdateMFX_01CastleB 0x113060, the NIS-marker half (`DAT_002d3524 == 0x6000071` → +0x14/+0x18 jump) | 0x7000006 | same parser gap (music.rs:221-229 documents the skip) |
| UpdateMFX_03TowerC 0x113a20 cases 0–5 (the NIS-id + dynamic-channel machine; the port runs its exit block only) | 0x700000b | needs the cel music-zone channel (below) for cases 1/3 and the NIS arms |
| Player_Update's cel music-zone channel (`Music_Event(cel+0xa1, 1)`) | any level with 0xc023/0xc047 prop-0x3c cels | parse prop 0x3c on those entities + track Bond's current cel per tick |
| Drone_EnableAll's ch15 extra (hash 0x600001b) | that one script | script not live on ported levels |

## Verification

- `cargo test -p nf-game --lib --offline --release` — music-filtered: **17 passed, 0 failed**;
  `the_death_nis_start_posts_the_death_music_jump`: **1 passed**. Whole lib: **659 passed,
  0 failed, 5 ignored**.
- nf-formats: **97 passed, 0 failed** across its binaries; the **3 disc_golden failures**
  (bigf_header_fields_behave_as_documented, bigf_tables_match_the_golden_manifest,
  directory_tree_matches_the_golden_manifest) are the slim-ISO's allowed set.
- nf-audio: **28 passed, 0 failed** (music_live included).
- Changed-code live binaries (all `--test-threads=1`, `--release`, real ISO):
  music_triggers_live **1/0**, fix_06_alarm_live **2/0**, fix_party_mission_live **9/0**
  (6 ignored, assertion-free probes), spawner_kill_live **5/0** (1 ignored, the known
  test-setup note), walkreal_07000009_live **8/0** (the exit + fail music paths),
  walkreal_07000005_live **14/0** and walkreal_07000012_live **3/0** (the two levels whose
  death NISs 0x6000952/0x600098a exercise the new ch6 post on real walks),
  walkreal_07000001_live **8/0**.
- The full four-crate sweep did not fit the session window (the all-binaries build alone
  outlived it); every binary that runs the changed code is listed above and green — the
  remaining walkreals exercise the same drone/music paths as the five run here.

## What's left

- The five unported consumers above (parser gap + the cel-zone producer).
- The `+0x4F8 & 0x100` all-clear arm (no port bit; [S] simplification, unobservable).
- `Script_Play`'s `Text_FlushAllSubtitles` on every camera-NIS start (subtitle concern,
  not music; the port flushes on the skip arm only).

## Live checks for the main agent

1. 07000005, walk Bond off the cliff edge (the plunge): with the debugger watch
   `EventList[6]` (EE 0x002d3510) — expect a write of 1 the tick SP #284's death NIS
   0x06000952 starts, and the music jumping to the on-death marker (a jump request to
   `MapMusicData+0xc` for the 05 row).
2. 07000002 (Mayhew B), let the hostage get saved: watch `EventList[16]` (0x002d3538) —
   expect one frame's write of 1 on the HostageSaved ENTER.
3. 07000002, get spotted by a mode-4 guard carrying prop 0x3c: watch `EventList[15]`
   (0x002d3534) — expect 1 on the first sight tick only (not on later ticks).

## Usage

not visible to me
