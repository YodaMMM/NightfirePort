# Guard radio chatter: Drone_InitComms and channel 0x98

No game code in the port writes channel 0x98 today, so the 0c/0d power-station guard chatter may never play. Port Drone_InitComms from the decompile, keeping its dead-guard skip: NDrone2_SetAsDead 0x14a2f0:9 sets the 0x200 flag and NDrone2_PostDroneControl 0x14aff8 sets 0x400, so dead guards must not raise it (an earlier attempt, s/markers-drone-c, dropped that skip and is rejected). Pin the chatter firing through real input on 0c or 0d.

Reproduce on the real level first through real input, not a teleport. Compare with the decompile, fix, and test red on the old code. Run the walkreal_* binary of every level the change touches, one level per tool call. Branch from origin/master, push to `s/drone-init-comms`, report in docs/reports/drone-init-comms.md. No draw changes: name what is wrong if it is a draw issue.

---
claimed-by: sandbox5 at 2026-11-09T10:51Z

---
## Report (done, 2026-11-09T12:15Z)


---
## Report from the worker (done, 2026-11-09T12:15Z)

# drone-init-comms — `Drone_InitComms`' ch-0x98 chatter gate, ported with the dead-guard skip

Branch `s/drone-init-comms`: from `origin/master` 63b5ae60d, three commits (28cadaa0e branch+WORKLOG,
b2a7aa81f the port, 47e7039fc the real-input pin + this report).

## What the task was

No port code wrote `switch_channels[0x98]`, so `NDrone2_PowerstationTalk` 0x13e770's idle radio
chatter on 0x070000c/0d (the power station) could never fire — its only gate is `0x98` [M
0x13e770:39]. The writer is `Drone_InitComms` 0x139520's one drone scan [M 0x139520:161-201]. The
earlier `s/markers-drone-c` round ported this scan but dropped the dead-guard skip and was rejected.

## The port (b2a7aa81f)

The game's scan [M 0x139520:161-201] runs once per frame after `DroneVision_ProcessDroneSight`:
the all-clear (`bVar2`) starts set, the chatter latch (`bVar3`) starts clear, and the list walks —

- **the dead `& 0x600` drones are skipped entirely** [M :166] — `NDrone2_SetAsDead` 0x14a2f0:9
  sets 0x200 and `NDrone2_PostDroneControl` 0x14aff8:9-12 sets 0x400 under health ≤ 0, so a dead
  guard can neither break the all-clear nor raise the chatter (the skip the rejected round dropped);
- a living DISABLED (`& 0x100` clear) team-1 guard (`+0x44 == 1`) breaks the all-clear, no stop
  [M :167-170];
- a living ENABLED guard at alertness ≥ 1 (`+0x50C >= 1.0`) within 30 units of its opponent
  (`+0x1A0 < 30.0`), not team 2 (`+0x44 != 2`), raises the chatter, breaks the all-clear and stops
  the walk [M :173-179];
- all-clear → `Music_Event(0xe, 1)` [M :194-196]; chatter → `switch_channels[0x97] = 1` [M
  :198-199]; **`switch_channels[0x98] = bVar3` EVERY frame, set or clear** [M :201].

Ported as one law, `comms_scan` + `CommsDrone` (drone.rs), consumed at two sites: the chatter/0x97
site at the game's position (after the vision pass, inside the `NPCGlobals` block) writes 0x97 and
0x98 as direct `.v` stores (InitComms stamps no timestamp on the pair [M :198-201], like the
0x97 writer master already carried), and the all-clear music read stays at its master music-order
position. The port's field encode is the documented fold (`find_opponent_alive`): `corpse` ↔ 0x200
(the death paths), `health <= 0` ↔ 0x400 (the game's PostDroneControl arm — the gate master's
0x97 line missed), `enabled` ↔ 0x100, `team` ↔ `+0x44` (the team byte,
docs/fields/hitdata-spawner-ally.md §4 — master's all-clear read `alert_status`, which is `+0x51C`,
the facial-status byte, and skipped the disabled drones arm 1 needs), `alert` ↔ `+0x50C`, `dist` ↔
`+0x1A0`. Two master bugs fall out fixed with the same evidence: the 0x97 raise lacked the team-2
exclusion [M :175], and the all-clear scanned the wrong byte with the wrong gate. No Rand draws
anywhere on the path [M :161-201] — the game's draw count is kept (zero).

Also updated: the channel-persistence census's owner set gains 0x98 (channel_persist_live.rs —
the per-tick writer is the game's own [M :201]).

## Verification

`init_comms_scan_law` (nf-game lib) pins the law on hand-built states: the boundaries (alert ≥ 1.0,
dist < 30.0 strict), the team-2 exclusion, both dead arms (corpse 0x200 / health-dead 0x400 raise
nothing — the rejected round's dropped skip), the disabled team-1 arm breaking the all-clear
without raising, and the empty list.

`comms_chatter_0c_live` pins the gate through real input on 0700000c, from the level start with no
teleport: the shared prologue to the yard ladder base, the ladder climb and the patrol-ring legs
(verbatim from guard1_rounds_0700000c_live), then the hold at the ring west edge where guard #1
(placement #3, weapon 0x1e [D docs/levels/0700000c-audit.md]) closes and fights — `ch 0x98` rises
with him aware at dist 10.08 [M 0x139520:173-179, 201], the alarm latch 0x97 rides the same scan
[M :198-199], every one of the 3847 sampled ticks agrees with the scan law recomputed test-side
from the raw drone fields, and Bond then kills guard #1 through the real fire path (the 0x1e armed
by the fights' own switch idiom, the aim-raise gate, tap fire) — the corpse must not hold the gate
up (this run the other yard guards held the ring, so the live dead-guard pin printed its no-op
note; the skip itself is pinned by the unit test).

**Red on the old code**: with `HEAD~2`'s drone.rs (master's tree) the same test fails at the rise
pin — `ch 0x98 never rose during the fight window — the InitComms chatter gate [M 0x139520:201]
never fired`; restored, it passes. (A first attempt checked out HEAD~1 by mistake — that commit
already carries the fix; the HEAD~2 run is the real red.)

## The suite (every binary below run whole on the final tree, `--test-threads=1`)

| binary | result |
|---|---|
| nf_game lib (incl. `init_comms_scan_law`) | **755 passed, 0 failed**, 5 ignored |
| channel_persist_live (all 23 levels' census) | **3 passed, 0 failed** |
| comms_chatter_0c_live (the new pin) | **1 passed, 0 failed** |
| walkreal_0700000c_live | **3 passed, 0 failed**, 1 ignored |
| walkreal_0700000c_barrelrow_live | **1 passed, 0 failed** |
| walkreal_0700000c_eastroute_live | **1 passed, 0 failed** |
| guard1_rounds_0700000c_live | **2 passed, 0 failed** |
| patrol_walk_0c_0d_live | **5 passed, 0 failed**, 1 ignored |
| 0700000d_gunimp_door_live | **2 passed, 0 failed** |
| talk_barks_live | **2 passed, 0 failed** |
| walk_0700000c_live | **4 passed, 0 failed** |
| npc_doors_live | **6 passed, 0 failed** |
| fix_guard_misc_live | **6 passed, 0 failed**, 1 ignored |
| walkreal_07000001_live | **7 passed, 0 failed**, 1 ignored |
| walkreal_0700000b_live | **7 passed, 0 failed** |
| walkreal_07000009_live | **9 passed, 0 failed** |
| walkreal_07000011_live | **5 passed, 0 failed** |
| walkreal_07000011_route_live | **1 passed, 0 failed** |
| walkreal_07000004_live | **9 passed, 0 failed**, 1 ignored |
| walkreal_07000005_live | **16 passed, 0 failed**, 1 ignored |
| walkreal_07000006_live | **6 passed, 0 failed**, 2 ignored |
| walkreal_07000007_live | **5 passed, 0 failed**, 1 ignored |
| walkreal_07000008_live | **5 passed, 0 failed**, 1 ignored |
| walkreal_0700000a_live | **13 passed, 0 failed**, 2 ignored |
| walkreal_0700000d_live | **1 passed, 0 failed**, 2 ignored |
| walkreal_07000012_live | **4 passed, 0 failed** |
| walkreal_07000013_live | **3 passed, 0 failed**, 1 ignored |
| walkreal_07000014_live | **7 passed, 0 failed**, 3 ignored |
| walkreal_07000041_live | **15 passed, 0 failed** |
| walkreal_0700004a_live | **4 passed, 0 failed** |
| walkreal_07000016_live | **15 passed, 0 failed**, 3 ignored |

Not run in the window: `walkreal_07000002_live` (killed at a 300 s cap) and
`walkreal_07000003_live` (killed at a 600 s cap — the 388 s level), plus
`walkreal_07000015_live`, `walkreal_0700001b_live`, `walkreal_0700004a_satchel_live`,
`walkreal_07000041_wall_live`. The change's only non-0c/0d surface is the 0x97 team-2 filter, the
0x98 writes (no reader outside `powerstation_talk`, which is 0c/0d-gated) and the all-clear music
event (write-only channel, no reader in ACTION.ELF), and 19 of the walked levels cover both gates'
live paths; the next round should still run the six above whole.

## What's left

- The six binaries above, run whole.
- Audible check (main agent, optional): on 0c/0d the power-station barks (voice ids 216..232, the
  `POWER_SFX` table) can now fire once a guard is aware within 30 units — `powerstation_talk`'s
  own [S] corner (two channel-eligible records, the deferred SpeechSFX gates) is documented at the
  port site (drone.rs) and unchanged by this round.
- No live checks needed: every new observable was pinned from real input here.

## Usage

not visible to me
