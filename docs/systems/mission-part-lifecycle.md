# The mission part lifecycle — what a level entry keeps, resets, and re-runs

Settles how mission 0x07000011's 4a leg folds into its parts. Every fact carries its evidence tag: [M] ACTION.ELF decompile
(function + address; disassembly checked where the C looked incomplete), [D] disc bytes, [L]
EE snapshot (file offset = EE address) or the port's real-path test run.

## 1. The tables [L ee_start_05.bin]

MissionData @ 0x2a4350, 24 × 0x28-byte records: +0x00 level (matched against Mission_Init's
param_1), +0x04 map (matched against `BaseMap`), +0x08 order, +0x0c pointer to the objective
records, +0x10 count. The tower mission (map 0x07000011) has FOUR parts sharing one map:

| rec | @0x2a4350+i·0x28 | level | map | order | objectives |
|---|---|---|---|---|---|
| 15 | 0x2a45a8 | 0x07000011 | 0x07000011 | 0 | 2 |
| 16 | 0x2a45d0 | 0x07000012 | 0x07000011 | 1 | 2 |
| 17 | 0x2a45f8 | 0x0700004a | 0x07000011 | 2 | 1 |
| 18 | 0x2a4620 | 0x07000013 | 0x07000011 | 3 | 1 |

Objective records (0x18 bytes): +0x00 label, +0x04 fail label, +0x0c condition channel,
+0x0d condition value, +0x0f enable channel, +0x10 flags (bit 1 = invert), +0x14 state (word).
The tower's records [L ee_start_05.bin @ 0x2a4188/0x2a41b8/0x2a41e8/0x2a4200]:

| record | label | cond ch | enable |
|---|---|---|---|
| 15.obj0 (0x2a4188) | 0x04000035 "Escape from the tower" | 0x64 (100) | — |
| 15.obj1 (0x2a41a0) | 0x04000036 "Gain access to lower office floor" | 0x48 | — |
| 16.obj0 (0x2a41b8) | 0x04000037 "Activate elevator override control" | 0x49 | 0x48 |
| 16.obj1 (0x2a41d0) | 0x04000038 "Escape through the elevator shaft" | 0x4d (77) | 0x49 |
| 17.obj0 (0x2a41e8) | 0x04000039 "Gain access to lobby" | 0x4a (74) | 0x4d |
| 18.obj0 (0x2a4200) | 0x0400003a "Raise security gate…" | 0x50 | 0x4a |

Note 18.obj0's enable IS 0x4a: the game itself designs ch 0x4a as the arrival-at-07000013
channel — it folds 17's objective AND announces 18's.

## 2. Mission_Init (0x182270) — the per-entry reset is map-scoped and order-scoped [M]

`Mission_Init(level, param_2)`: finds the record whose level == param_1, sets `BaseMap` =
record+0x04 and `ThisOrderNum` = record+0x08 (0x182270:30-52), re-arms the layer
`InternalState = 1` + `Mission_SetStatus(1)` (:25-28).

The reset walk (:71-129) covers ONLY records with record.map == BaseMap (:73), classified by
order against ThisOrderNum (disassembly 0x182410-0x182438: `$a2` = 1 if order < ThisOrderNum
(previous), 2 if equal (current), 0 if greater (future); 0x182438 `movz $v0, $a2, $s2` —
param_2 != 0 folds previous into case 4):

- **current (2) and future (0)** reach the reset at 0x1824a8-0x1824c4: state (+0x14) ←
  (enable byte == 0 ? 0 : 1), and if the condition channel (+0xc) != 0,
  `switch_channels[ch] = value(+0xd)` (the condition channel forced to the record's value,
  normally 0 = cleared) plus `switch_channels_time[ch] = now` when the value is nonzero.
- **previous (1 with param_2 == 0, case 4 with param_2 != 0)** fall through with NO store:
  the `beq $a2, $t3, 0x1824e8` (case 1) and `bnel $a2, $s0, 0x1824ec` (case 4) branches both
  jump straight to the loop tail (disassembly 0x182488-0x1824a4; the C shows the same two
  no-op arms). Previous parts' objective states and channels are KEPT.

So a level entry never resets or re-runs earlier parts' records — 0x07000011/12's completed
objectives stay complete on the 4a leg, and their raised channels (0x48/0x49) survive.

## 3. Mission_MonitorObjectives (0x1827d8) — the walk is also map-scoped [M]

The monitor loops all 24 records but walks objectives only where record.map == BaseMap
(:24). On every tower leg ALL FOUR parts' records are walked (one shared map) — that is the
mechanism behind the "late AND": 16.obj1 (cond 0x4d) folds on the 0700004a leg where
TouchOnce #277 raises 0x4d [D 0700004a.md §2], one level after its own record.

Per record: `bVar1` = `switch_channels[cond]` (inverted when flags bit 1), `bVar9 &= bVar1`
for every walked record; the state switch (:40-90) has cases 0 (announce once
record.level <= current level), 1 (state 1: announce when the ENABLE channel is on), 2
(inverted-variant fail), **3 (announced: cond on → msg 0x2000053 + state 4 — the fold)**, 5
(failed: FailLabel). There is NO case 4: a folded (state 4) record is never re-run —
completed objectives stay completed. The final all-AND writes `switch_channels[100] = 1`
(:119-123) only when every walked condition is on — on the 4a leg 17.obj0's cond (0x4a) is
still off, so the mission does NOT complete there.

## 4. Where ch 0x4a fires — the settle

0700004a has NO ch 0x4a writer: the 13 plates' outs, the 162 doors' ch_open/ch_lock, the 6
spawners (param 6 = 0), the 4 SPs (outs 0x22/0x83/0x84/0x40), the 11 scripts' events, and
every `switch_channels[...] =` write in ACTION.ELF were censused [D
docs/systems/tower2-lift-ride.md §2 step 8]. The writer is 07000013's spawn touch #153 (out
0x4a, covering the start) [D 07000013.bin]; walk_07000013_live pins the real chain [L]:
ch 0x4a fires after the warm load into 07000013, 17.obj0 folds (state 4) and 18.obj0
announces (enable 0x4a). Under the real ride-end writer the lift script ends writing NO
channel (the removed stand-in wrote ch 0x4a there).

**Verdict**: the real ride end neither resets nor re-runs 07000012's
records ([M 0x182270 §2 — previous parts get no store; M 0x1827d8 — no case 4]) and the ride-end
writer fires at the right time and place ([D census + D 07000013.bin + L walk_07000013_live]).
The level-12 objective test's sister assertion "12's override objective stays complete through the fold [M 0x182270
keeps previous parts]" is CORRECT and stands. What was wrong was its third assertion —
"4a's own objective folded at the ride's end (ch 0x4a)": it pinned the removed stand-in's
write. The game's record 17 can only ANNOUNCE on the 4a leg (enable 0x4d fires via touch
#277; cond 0x4a never does): `states[4a][0] == 3` at the ride's end, the fold happening on
07000013. campaign_flow_live's objective-77 block pins exactly that now, with these
citations.

## Tests

- `campaign_flow_live::campaign_flow_story_missions_4_to_7` — the objective-77 block:
  `states[i12][1] == 4` (the late-AND fold on 4a), `states[i12][0] == 4` (the sister,
  previous parts kept), `states[i4a][0] == 3` (announced, not folded — the corrected
  assertion).
- `walk_07000013_live::the_warm_chain_arrives_with_all_previous_channels_on` — the fold at
  its real place: `states[4a][0] == 4` + `states[13][0] == 3` after touch #153's ch 0x4a.
- `fix_finale_live::the_0700004a_lift_ride_reaches_the_lobby` — the ride end is real: ch
  0x4a OFF on 0700004a (fails on the removed stand-in, which wrote it).
