# The Tower-2 lift ride (0700004a) — the real end-to-end chain

The ride's own writers, ported and pinned (script-end-writer-4a). Every fact carries its evidence
tag: [M] ACTION.ELF decompile (function + address), [D] disc bytes (level bin / pack / placement),
[L] the port's real-path test run. The level doc is docs/levels/0700004a.md; this file is the
channel-writer chain the ride end rides on.

## 1. The pieces [D]

| Object | Data |
|---|---|
| The lift SP #42 (m41) | kind 0xD9, script 0x06000571, mode 1, stay 1, **mask 1**, **out 0x22**, gate 0 [D placement params] |
| The lift script 0x06000571 (0700004a.bin, kind 7) | len 5040; s5 Wait(1610) → **event 0x0d [0x40, 1]**; s4 Wait(1611) → **event 0x03 [0]** (the pause, p0 = 0) [D decode] |
| The −52 landing cut-scene SP #289 (m41, at 0, −53.53, −3.38) | kind 0xD9, script 0x060006cf, mode 0, stay 1, **mask 1**, **out 0x40**, gate 0 [D placement params] |
| The landing script 0x060006cf (**loadable pack 0780004a.bin**, the 0xC record (0x060006cf, 1)) | len 575; s9 t0 **event 0x04** (Player_Disable); s26 t575 **event 0x06 [1]** (the handback pose); s27 t575 **event 0x0d [0x22, 1]** [D decode] |
| The lobby cut-scene SP #276 (m41) | kind 0xD9, script 0x060006d3, mode 0, stay 1, **mask 1**, **out 0x84** [D placement params] |
| The lobby script 0x060006d3 (**pack 0790004a.bin**, the 0xC record (0x060006d3, 2)) | len 800; s9 t0 event 0x04 (Player_Disable); s9 t800 **event 0x12 [0x0720004a]** — the level exit (a movie code; `GetLevelToLoadFromMovieCode` 0x1d3560 → 0x07000013) [D decode] |
| The bottom plate #62 (m41, 3.03, −156.21, −3.35) | TouchOnce, out **0x84** [D placement params] |
| 07000013's spawn touch #153 | TouchOnce, out **0x4a**, covering the start #9 (2.16, −369.10, 10.96) [D 07000013.bin; walk_07000013_live] |

## 2. The chain, tick by tick

1. **t0** — the lift SP is mode 1: created in state 7 [M SP_CreateScriptPlayer 0x1db9a8:103-106 —
   `mode == 1 → +0x24 = 7`], SP_Update case 7 → `SP_LoadScript` [M 0x1dbda8 case 7], whose mode-1
   arm plays the script and lands state 2 [M SP_LoadScript 0x1dbc08:44-47]. The platform entity
   (0x20005b4) descends 150 units from key t90 to t5040 [D 0x06000571 keys].
2. **t1610** — the script's s5 event 0x0d writes **ch 0x40 = 1** [D s5; M Script_EventHandler
   0x1d94c0 case 0xd: `switch_channels[p0] = p1`]. The −52 floor door pair #110/#111 (ch_lock
   0x40 [D]) unlocks on the rising edge [M Door_Update 0x1363c8].
3. **t1610/1611** — ch 0x40 is ALSO the landing SP #289's out channel, and mask 1 is SP_Hit's
   out-channel bit: **trig = `switch_channels[out] != 0`** [M SP_Hit 0x1dc470:17-19 — bVar8 starts
   as the out channel's value; SP_Update 0x1dbda8:66-68 gates SP_Hit on the countdown and
   `bVar4 = flags & mask`]. SP #289 sits in state 0 (mode 0, mask ≠ 0 [M 0x1db9a8:100-107]); the
   state-0 arm takes the trigger: the script is loadable (the 0xC record), so the first tick arms
   `GameFlow_PushState(12.0, 0xd, 0xff)` [M 0x1dbda8:80-99] — the pack load — and the next tick
   loads + plays 0x060006cf, state 2 [M 0x1dbc08].
4. **t1611** — the lift script's s4 event 0x03 [0] sets `SCRIPTINFO+0x9C = p0 − 1 = −1` [M
   0x1d94c0 case 3] — the hold. `Script_Update` runs nothing while the countdown is negative and
   only decrements a POSITIVE one [M 0x1da998:17-18,112-114] — the platform freezes at the −52
   landing. The event also snapshots every stream object's position (`Vec_Copy2(obj+0xC0 ←
   obj+0x30/0x40)` [M 0x1d94c0 case 3 tail]).
5. **t1612..t2186** — the landing cut-scene plays (575 ticks [D]): Bond disabled (s9 event 0x04 →
   `Player_Disable` [M 0x1d94c0 case 4]), the scene animates the rope descend.
6. **t575 of the scene** — s26 event 0x06 [1] records the handback: `SCRIPTINFO+0x40` ← the
   record's object matrix (or Bond's own when the object is 0) [M 0x1d94c0 case 6: Mat_Copy into
   SCRIPTINFO+0x40, the param byte → +0xAB6], and sets the +0xAA8 bit 0x100; s27 event 0x0d writes
   **ch 0x22 = 1** [D s27].
7. **The same tick** — Script_Free (the SP's script ended, stay = 1 → state 5 [M 0x1dbda8 case 2])
   sees the 0x100 bit and calls `Player_Enable(glb_players, SCRIPTINFO+0x40, +0xAB6)` [M
   Script_Free 0x1da880:46-49] — Bond re-enabled at the recorded pose. ch 0x22 ON hits the lift
   SP's state-2 rule: mask&1 && out-channel ON → **SP_UnPause** [M 0x1dbda8 case 2 head;
   SP_UnPause 0x1dca58 zeroes SCRIPTINFO+0x9C] — **the hold releases and the lift resumes**.
8. **t5040** — the lift script ends; the platform parks 150 units down (the lobby). The SP's
   stay=1 sends it to state 5 (delete) [M 0x1dbda8 case 2 tail]. **Nothing writes ch 0x4a on
   0700004a** — every writer was checked: the 13 plates' outs, the 162 doors' ch_open/ch_lock
   groups, the 6 spawners' outs (param 6 = 0 on all six [D]), the 4 SPs' outs (0x22/0x83/0x84/0x40
   [D]), the 11 scripts' events (0xd/0x11 writes: only ch 0x40 [D dump]), and every
   `switch_channels[...] =` write in ACTION.ELF (the writers list, all data-driven, none 0x4a).
9. **Bond walks off the platform**; the bottom plate #62 fires **ch 0x84** [D] → SP #276's state-0
   trigger (mask 1 = the out channel, step 3) → the lobby cut-scene 0x060006d3 (pack 0790004a)
   plays; at its t800 the **event 0x12 [0x0720004a]** exits the level [D; M 0x1d94c0 case 12:
   `Player_RamSave` + `ResetMap_LevelToLoad(p0, 1, 0)`] → 0x07000013 (the movie-code resolve,
   the level-chain work).
10. **07000013** — Bond spawns INTO touch #153, which writes **ch 0x4a** [D]. The mission monitor
    watches every part's records of the mission [M Mission_MonitorObjectives 0x1827d8:22-38 — the
    rows filter on BaseMap; the port: mission.rs monitor() over `LEVELS`], so 4a's "Gain access to
    lobby" (complete ch 0x4a, enable 0x4d [L ee_start_05.bin @ 0x2a41e8]) folds on 07000013 with
    the banner, and the record's enable byte re-arms it after Mission_Init's part reset [M
    0x182270].

## 3. The stand-ins this removes (nis.rs SP state 2)

- **`ch 0x22` written when the lift script's hold engages** — the real writer is the landing
  cut-scene's end event (step 6) [D 0780004a.bin 0x060006cf s27]; the port's SP out-channel
  trigger (step 3) and pack loading were already ported, so the real chain plays.
- **`ch 0x4a` written when the lift script ends** — nothing on 0700004a writes it (step 8); the
  real writer is 07000013's spawn touch #153 (step 10) [D], already pinned live by
  `walk_07000013_live::the_warm_chain_arrives_with_all_previous_channels_on`.

## 4. Open item

The handback pose (step 7): the event-6 record's object matrix composition — base-composed
(1.80, −107.66, −6.83, a void column) vs raw (1.80, −54.13, −3.45, the freeze pose on the
platform). fix-final-blockers2/walkreal-c hold the analysis and the live check (gdbwatch the
enable matrix + Bond's y through the −52 landing); the port currently base-composes, so the
rider falls after the landing cut-scene. The CHANNEL chain above is independent of it: the
script end, ch 0x22's writer and ch 0x4a's writer behave as documented either way.

## Tests

- `fix_finale_live::the_0700004a_lift_ride_reaches_the_lobby` — the real ride end: the t1610
  ch 0x40 writer, the 575-tick disabled window, the ch-0x22 rise 576 ticks after ch 0x40 (fails
  on the removed stand-in: it wrote 0x22 two ticks after the hold), the resume, the parked
  platform at the script's end, and ch 0x4a OFF on 0700004a.
- `walk_07000013_live::the_warm_chain_arrives_with_all_previous_channels_on` — the ch-0x4a writer
  on 07000013 and the objective fold (pre-existing pin).
- `interact_0700004a_live::the_minus52_landing_doors_unlock_on_the_ride` — the sister: the
  t1610 writer's own consumer (the door pair's lock edge).
