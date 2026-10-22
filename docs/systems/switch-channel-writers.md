# Switch-channel writers — the census (channel-writer-census)

The full census of every `switch_channels[...] =` write site in ACTION.ELF (129 sites in 60
files). The level reports' "no writer on the disc" conclusions came from scanning for a channel
NUMBER as a constant; most writers carry the index in a DATA field (a placement param, a script
op's argument), so the number appears only in the level's own records. This doc maps every
writer family to the data record that supplies the index, then resolves the task's listed
channels against the discs' records.

## The writer families ([M] all ACTION.ELF)

| writer | write | the index's data source |
|---|---|---|
| `Pickup_Handler` 0x18e650:330-333 | `ch[+0x28] = 1` + the time stamp | the pickup record's +0x28 — placement param 3 [M 0x1d04f0:457-462] or the script Entity command's param 3 [M 0x1d8bfc] |
| `Trigger_Activate` 0x1b2318:30/42/97 | `ch[+0x20] = 1`, `ch[+0x2c] = 1` | the trigger's out: placement param 0 (TOUCH/TOUCH_ONCE/AND-family) or param 1 [M Trigger_Create 0x1b1720:61-91] |
| `Trigger_Update` 0x1b1a78:83/154/184/193 | OR-in, copy, assign, CLEAR | the trigger's in/out slots (ext+0x26.. = placement params 1..8) |
| `Switch_Activate` 0x1b02a8:25 | `ch[obj+0x12] ^= 1` (TOGGLE) | the switch's channel — placement param (Switch_Create 0x1adcc0:40 seeds it from +0x16) |
| `Switch_Update` 0x1adea8:32 | toggle | the switch's +0x12 |
| `Door_Create` 0x136100:97 / `Door_SetState` 0x135618:26 | `ch[obj+0x66] = 1` / `= (state & 0xc) != 0` | the door's state-mirror channel (+0x66) |
| `SP_Update` 0x1dbda8:220-226 | `ch[+0x30] = (script running) != 0` | the SP's out channel — placement **param 6** [M SP_CreateScriptPlayer 0x1db9a8] |
| `Script_EventHandler` 0x1d94c0:187-200 (events 0xD/0x11) | `ch[i] = v`, conditional | the script stream's event args — `[0x12, n, kind] + n×u32` [D docs/formats/nis-script.md] |
| `DroneFunc_SetDeathChannel` 0x13e660:48 | `ch[drone+0x13c] = 1` | the drone's death channel — placement **param 12** (byte) |
| `DroneSpawner_Control` 0x137e68:135 | `ch[spawner+0x12] = 1` | the spawner's channel param |
| `Copter_Update` 0x17dbb8:101/250-267 | `ch[+0x72/+0x74] = 1` | the copter's channel params |
| `Hurt_Create` 0x17e870:56 / `Destroy_Update` 0x12c4f8:23 | `ch[param+6/+0x12] = 1` | the hurt/destroy record's channel |
| `Searchlight_Update` 0x1cb900:143 / `Monitor_Update` 0x1afcc8:173 | `ch[+0x1b/uVar2] = 1` | the object's channel field |
| `Player_Wire` 0x197658:110/162 | `ch[wire] = 1/0` | the wire's channel (set while riding, cleared on drop) |
| `Mission_MonitorObjectives` 0x1827d8:70-120 | `ch[99] = 1`, `ch[100] = 1` | CONSTANT: 99 = the mission end, 100 = all objectives complete. Reads the objective table (per level, [M 0x2a3f18]): each record watches a byte channel (+0x0c), bit 1 of +0x10 INVERTS (complete-on-clear) |
| fixed ids: 0x5e (death 0x190810:40), 0x62 (init 0x1910f8), 0x7c (collision 0x1949a8), 0x96/97/98/99/9b (drone comms 0x139520/0x14b058/0x153358/0x136938), 0x1a (castle guard 0x1612e0), 0x6f/0x2d (Trigger_Calc 0x1b1900), 0x53 (Explode 0x17c350), 0x76 (SSys_Msg 0x1ad878), 0x48 (weapon 0x1a24c8/0x1916b0/0x191dd0) | constants | — |

## The listed channels, resolved

Every level record cite is [D] (the port's parsed archive — placements `params`/NIS streams;
probe: placements + `Stream::decode`), every mechanism [M] as above.

| level | ch | writer (the real path) | the record that carries it | state in the port |
|---|---|---|---|---|
| 07000003 | **0x4E** Retrieve Dragon Safe contents | `Pickup_Handler` 0x18e650:330-333 on taking the safe briefcase | script **0x06000563 stream1 cmd[6] @frame 91**: Entity obj_type 0xF0, hash 0x020005F1, params [kind 2, id 0, rounds 0, **ch 0x4E**, sound 0xffff, respawn 0] — parented to stream 0 (the safe, hash 0x020005F0) | take-write ported (drone.rs); the briefcase's SPAWN is the gap — see below |
| 07000003 | 0x51 | not yet resolved — the only records carrying 81 are the AND trigger (below) as an INPUT and float noise | — | open |
| 07000003 | 0x52 (the exit's key gate) | `Trigger_Update`'s AND write [M 0x1b1a78] | placement idx316 (type 0xEC AND): out = param0 = 0x52, ins = {param1 = 0x4E, param2 = 0x51} — the exit unlocks when BOTH objectives' channels are on | the AND is ported; blocked on 0x4E/0x51 above |
| 07000016 | **0x50** | script events: **0x0600090a stream21 event 0xD writes [50, 1]**; **0x060009e9 stream7 event 0xD writes [50, 0]** (the clear!) [D]; also an SP idx186 (type 0xD9, param6 = 0x50) mirrors it [M 0x1dbda8:226], and a pickup (0x02000637, param3 = 0x50) | the kiko scripts | the event/SP writers are ported; the walk beat = which fires on the real route |
| 07000016 | **0x81** | `Trigger_Activate` on a TOUCH_ONCE plate | placement idx166 (type 0xEA): out = param0 = 0x81; a second SP idx168 (param6 = 0x81) | ported |
| 07000016 | **0x8D** (the KikoDeathButton gate) — SET | a TOUCH_ONCE plate (idx166 second record: out = param0 = 0x8D) AND the death-button switch idx185 (hash 0x020008C1, type 0x29, param5 = 0x8D — `Switch_Activate` 0x1b02a8:25 TOGGLES it, so a second press CLEARS it) | the Kiko death button | the set is ported; **the clear** = the switch's own toggle arm — whether the real walk presses twice (or another site clears it) is open |
| 07000016 | 0x50/0x81 note | — | the 0x02000637 pickup with param3 = 0x50 ALSO writes 0x50 on take | — |
| 07000001 | **0x47** Escort Mayhew | placement map53 idx193, hash **0x02000675** (type 0xF0 — the escort/pickup class — the 0xF0 class is BOTH the placed pickups AND the script-spawned ones [M 0x1d04f0 case 0xF0 → Pickup_Create]), param1 = 0x47 — NOTE: the pickup's channel field is param 3, not param 1, so this record's 0x47 rides a different field (id 0x47 = weapon 0x47? the fists?) — the escort writer is open | — | open (live check) |
| 07000005 | **0x46** | the guards (type 0x47, flags 0xc047, 18 placements) carry param1 = 0x46 — the drone channel params; the death channel is param 12 [M 0x13e660] | — | open: which guard param fires 0x46 (live check) |
| 07000008 | **137** | the fusebox chain (fix-safe08's doorway bolt) — `Switch_Activate`/the fusebox SP | the fusebox's script (0x060009E2-family) | ported (fix-safe08) |
| 0700000a | **0x3C** | the guards: ~25 type-0x47 placements with param0/6/7/8 = 0x3C — the death channel is param 12 [M 0x13e660] | — | open (live check) |
| 0700000a | **0x37** | a TOUCH trigger idx540 (param6 = 0x37) + an OR trigger idx543 (param2 = 0x37) | — | ported |
| 0700000d | **200 (0xC8)** | type 0xDB = **DOOR** [M 0x1d04f0 case 0xdb → Door_Create] — 4 door placements (idx213) carry param3 = 0xC8, but the door's channel fields are param0/1/2 (+0x6a/+0x66/+0x68 [M 0x136100:25-27]) — param3 sits beyond them, so the 200 write is NOT the door mirror | — | open (live check: which write hits 0x2a38d2+200) |
| 0700000d | **103 (0x67)** | an SP (idx65535, hash 0x02000B51, param1 = 0x67 — the script hash slot!) + idx212 (type 0xF9, param0 = 0x67) | — | open |
| 07000011 | **0x1F** | the exit (idx124, 0xE8 LOAD_LEVEL, param2 = 0x1F = the gate READ); a type-0xE9 record (hash 0x0200063D, param2 = 0x1F) | the writer of the gate channel itself | open |
| 07000013 | **0x23** | an **SP (idx70, hash 0x020005E2, type 0xD9, param6 = 0x23)** — `SP_Update` 0x1dbda8:226 writes its out when its script ends | the SP's script runs to completion on the level's beat | ported mechanism; the script's start beat = live check |
| 07000015 | **0xC8** | type 0xDB = door (idx126, param3 = 0xC8) — the same door family as 0700000d | — | open (live check) |

## The Dragon Safe chain, end to end

1. The safe sits at (6.70, 6.14, −42.55) [D the world-anchored SP idx65535's placement].
2. Script 0x06000563 ("the searchlight alarm" — its stream3 event 0xD writes ch 0x96 = 1 [D])
   is anchored by TWO type-0xD9 SPs: idx65535 (world-anchored, **mode 64**) and idx72
   (identity-anchored at (0,0,0), **mode 0**) [D].
3. Mode 64 = the one-tick preload: `Script_Play(0)` + ONE `Script_Update` + `Script_Stop`,
   state 1 [M SP_LoadScript 0x1dbc08:56-62]. It poses the entities (stream 0 = the safe at the
   SP's base; stream 1 = the contents' holder at (6.61, 5.51, −42.55) — the port's compose
   verifies the parent+base math) but does NOT reach frame 91.
4. Mode 0 = wait-for-trigger: when started, `Script_Play(loop=0)` → the script plays THROUGH →
   at frame 91 stream1's obj_type-0xF0 Entity runs `Pickup_Create` at the stream's posed
   position [M Script_CreateEntity 0x1d8968's 0xF0 arm, 0x1d8bcc-0x1d8c20] — the briefcase with
   switch channel 0x4E.
5. Taking it writes ch 0x4E [M 0x18e650:330-333] → the objective completes → with 0x51, the AND
   trigger unlocks the exit (0x52).

**The port gap**: the reserved slot exists (script_pickup_reservations) but sits inert at the
RAW key (−0.089, −0.63, 0.0017) and nothing reaches frame 91: the world SP's preload stops at
one tick (as the game's does), and the identity SP (mode 0) is never started because the port
has not wired its trigger. **Which in-game event starts the identity SP — and where the
briefcase really lands (the identity SP's base is the world origin)** — is a live check (below).

## Live checks for the main agent

1. `tools/trace/gdbwatch.py` on `switch_channels[0x4E]` (0x2a38d2 + 0x4E) and on
   `switch_channels_time[0x4E]`, fresh boot of 07000003, play to the safe: the exact tick 0x4E
   rises — at load + ~1.5 s (the preload) or on entering the safe room (the identity SP's
   trigger) or on taking the briefcase.
2. `fieldprobe` on the briefcase pickup object: its obj+0x90 translation when it exists —
   inside the safe ((6.6, 5.5, −42.6)-ish) or at the world origin (the identity SP's base)?
3. The same for 07000016's ch 0x8D: does the real death button TOGGLE (a second press clears),
   and does anything else write 0 on it (gdbwatch the write sites during a real walk)?
4. 07000001: which code writes ch 0x47 — gdbwatch the write to 0x2a38d2+0x47 during the escort.
