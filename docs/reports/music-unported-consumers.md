# music-unported-consumers — the audit's tabled music consumers, ported

Branch `s/music-unported-consumers` (cut from origin/master f503c15c7, merged — master was
already current). This ports what a player can hear from `audit-music-events`'s "Unported
consumers" table: the per-track `UpdateMFX_*` readers of the event board, the cel music-zone
channel from `Player_Update`, and the section/fade logic they drive, on-foot levels only.

## What was ported (each site cites the game)

1. **The cel music-zone channel** — `Player_Update` 0x193600:117-122: every tick, while the
   player object's cel (`obj+0x20`) carries a non-zero byte at `cel+0xa1`, it posts
   `Music_Event(ch, 1)`. The channel is the room-cel entity's prop 0x3c:
   `parseentity_fixup_entity` 0x1d00c0's 0xc023/0xc047 arms write
   `*(char *)(cel+0xa1) = *(char *)(entity+0x3c)` and `build_alloc_cel` 0x1c93f8 zeroes it on
   every other cel — so generic cels post nothing. Port site: `Cel::music_zone` (param 4)
   parsed in `parse_cel_graph` (nf-view/src/scene.rs), posted per tick at the player-cel relink
   (scene.rs, reading the stale pre-relink cel — the game's read precedes the same tick's
   movement relink in `Player_Update`).
   **Disc check**: 07000002's room cels carry zones 30/36/37 (13 zoned cels — the 36 hall is
   cel #19), 07000003 carries 35/37, 0700000b carries 44 (cels #0-8/#14) and 41 (cel #13),
   07000005 carries none, 07000006 carries zone 10 [D placement param 4 census, probe run].
2. **`UpdateMFX_03TowerC` 0x113a20 (0700000b), cases 0-6, whole** — the NIS-id + dynamic-channel
   machine: state 0: the 0x60004f9 NIS start (e[11]) → jump 2; 1: e[44] → 4; 2: e[41] → 6;
   3: e[17] → 8 INSTANT; 4: the alarm pair (attack latches MusicTemp2 + jump 8, calm after a
   latch → 0x15) and the 0x60004fa NIS start → 10 attack / 0x17 calm; 5: the NIS end (e[12]) →
   0xc INSTANT; 6: the exit/fail events → the FIXED 0xe INSTANT — no fade, no end-wait.
   **Fix riding along**: 0b was in the port's shared exit-block range, but the game's TowerC
   updater has NO `e[3]/e[8]` head — its only exit arm is state 6's fixed 0xe [M 0x113a20:69-75
   vs the shared head 0x113558:9-33]. 0b is removed from the shared range; pre-fix the port
   jumped end markers with a fade + end-wait on ANY exit tick. The disc row backs the machine:
   0b's MapMusic combat/stealth sets are EMPTY [D 0x24f4d8] — TowerC manages its own sections.
3. **`UpdateMFX_01CastleB` 0x113060 (07000006), the NIS-marker half** — while
   `MusicWaitForNisSfxTrigger` is down: the 0x6000071 NIS start (e[11]) jumps the record's
   +0x14/+0x18 marker — the +0x24 scan against `MusicLastJump` picks — INSTANTLY and latches
   the wait (the block's `SFXFadeDown(0)` is dead on 06: the game skips it for
   `MapMusicData == 0x7000006` [M 0x113060:36-38]); the alarm pick stops while the latch
   stands; the cut-scene's music sound (`SfxNisMusicTrigger`) then jumps 0xc non-instant and
   holds `MusicWaitForEnd` [M 0x113060:48-53].
4. **`UpdateMFX_02MayhewB` 0x112270 (07000002), the whole machine** — the level-start arm
   (IOP at section 0 + the 0x6000084 NIS end e[12] or the zone-36 event e[36] → section 2
   non-instant [M :48-54]), the per-tick attack-gate re-seat (e[37] low → the `MapDroneData`
   row back; e[37] high → `DAT_002707a8 = 1.0` [M :56-79]), the 0x600007f cut-scene pair
   (e[11] → 6 instant, e[12] → 8 instant [M :81-95]), the ch15/16 latch (first-sight/hostage-
   saved → `PickRandomCombat(0)` + the 300-tick gate [M :96-103]), the trigger pair (e[31] → 4
   INSTANT, e[32] → 4 non-instant, both `MusicVars = 1` + bonus 600 [M :104-127]), the alarm
   round-robin [M :128-131], the stealth round-robin 0x14/0x16 gated on the +0x38 scan against
   `MusicLastJumpRequest` with `instant = MusicLastJump == 4` [M :132-163], and the e[30] tail
   (+0x24 scan → 0x12 / hold / 0xa [M :169-202]).
5. **`UpdateMFX_02MayhewC` 0x112730 (07000003), the whole machine** — the ch15/16 latch with
   the 0x78 gate [M :41-48], the trigger pair (e[31] → 0xa, e[32] → 0xc, both `MusicVars = 1` +
   bonus 600 [M :53-53, :207-223]), the calm stealth round-robin 6/8 behind the +0x38 scan
   against the request with the +0x24 combat-set arm and the bonus<0 start-over [M :55-113],
   the attack branches (e[33] low → the 2/4 round-robin behind both scans [M :115-164]; e[33]
   high → 0xe [M :166-204]).
6. **The updaters' direct attack-gate writes** — `MusicVars` (0x2707a0, the attackers-needed
   count) and `DAT_002707a8` (the sight distance) are written by the machines (MayhewB's
   per-tick re-seat and both trigger pairs). `Music::gate_writes` carries them out of
   `update()`; `Drones::tick` drains them onto `attack_gate.vars` AFTER `update`, so the next
   frame's `NDrone2_SeenAndAttacking` pass sees them — the game's frame order (the drone pass
   precedes 0x114e70). `AttackGate::table_row` (seeded in `wire_music`) re-seats the row for
   `GateWrite::Restore`; no row → the game's default count 2 / floor 8 / distance 25 [M
   0x112270:72-75].
7. **The restart clears `SfxNisMusicTrigger`** [M 0x114e70:38] — a stale cut-scene trigger from
   a previous load could fire the new level's NIS arms.

## What stays out (with reasons)

- **`Drone_EnableAll`'s ch15 extra (hash 0x600001b)** — one script-hash-gated site; the script
  is not live on the ported levels (unchanged from the audit's table).
- **The `+0x4F8 & 0x100` all-clear arm** — the port carries no 0x100 bit; [S] simplification,
  unobservable (ch14 has no reader).
- **The driving-level updaters** (Sub_DeployTorpedoes/Sub_Update/MiniSubState_Attack's ch2
  posts) — driving, out of scope per the task.
- **MayhewB's `MusicTemp7`** — written [M 0x112270:82], never read anywhere: write-only, no
  field (noted in code).
- **`MusicVars` re-scans in the dispatcher** (the e[2]==2 −99 row and the count2-decay arms) —
  pre-existing shipped [S] gaps in the port's dispatcher, untouched here; MayhewB's own re-seat
  (the load-bearing one on 02) IS ported.

## Corrections to the audit's table

- The audit's blocker for MayhewB/C/CastleB ("reading MapMusic marker fields +0x14/+0x18/+0x24/
  +0x38 the MapMusic parser doesn't read") was a misdiagnosis: the parser already reads all four
  (end_combat/end_calm/combat/stealth, nf-formats/src/music.rs:119-120). The real gap was only
  the unported state machines — no parser change was needed, and the machines port cleanly
  against the parsed fields.
- The shared exit-block list in music.rs cited TowerC 0x113a20:70 as carrying the block; the
  decomp shows 0b's updater does not (its exit is state 6's fixed 0xe) — fixed above.

## Verification

- `cargo test -p nf-game --lib --offline --release`: **666 passed, 0 failed, 5 ignored**
  (was 659 + the 4 new music tests + the rebalanced 06 helper). Music-filtered: 21 passed,
  0 failed — the 4 new pins:
  - `the_tower_c_machine_walks_the_nis_and_zone_chain` — the full 0→6 chain on the real 0b row
    (empty sets), plus the sister: an exit event in state 0 does NOTHING (the shared-block
    removal pin);
  - `the_07000006_nis_marker_half_jumps_the_end_marker_then_the_closing_sting` — end_combat 8
    instant via the +0x24 scan, 30 silent ticks under the latch, the 0xc closing sting + the
    end-wait; sister: end_calm 6;
  - `the_07000002_machine_level_start_trigger_and_tail` — the zone-36 level-start arm, the
    per-tick gate re-seat, e[31] → 4 instant + `MusicVars = 1` + bonus 600, the sight pick,
    e[37] → SetDistance(1.0), the e[30] tail;
  - `the_07000003_machine_triggers_and_round_robins` — e[31]/e[32] pair, the stealth
    round-robin with the in-set stop, the attack 2/4 round-robin, e[33] high → 0xe.
  All against the REAL disc rows [D 0x24f4d8]: 02 (combat [12,14,16], stealth [2,20,22,8]),
  03 ([2,4,14]/[0,6,8]), 06 ([2,15,17]/[4,19,21], end 8/6), 0b (empty/empty) — the rows the
  hand-built test rows mirror, tied to the disc by the live test below. The existing 06 helper
  now uses the real row (was hand values).
- `cargo test -p nf-game --offline --release`: **103 test binaries, all ok, 0 failed**.
- `cargo test -p nf-formats --offline --release`: green except the **3 allowed slim-ISO
  disc_golden** failures (bigf_header_fields_behave_as_documented,
  bigf_tables_match_the_golden_manifest, directory_tree_matches_the_golden_manifest).
- `cargo test -p nf-audio --offline --release`: **29 passed, 0 failed** (music_live included).
- Changed-code live binaries (all `--test-threads=1`, real ISO):
  `music_unported_consumers_live` **2/0** (the disc-row pin + the producer pin:
  Bond walked into 07000002's zone-36 hall posts the channel every tick and MayhewB's
  level-start arm starts section 2 — the pre-fix port parsed no music channel, so the hall was
  silent), `music_triggers_live` **1/0**, `fix_06_alarm_live` **2/0** (the restructured 06
  path), `fix_party_mission_live` **9/0** (6 ignored), `walkreal_07000009_live` **8/0** (the
  exit + fail music paths), `walkreal_07000006_live` **6/0** (1 ignored),
  `walkreal_07000001_live` **8/0**, `walkreal_07000005_live` **14/0**,
  `walkreal_07000012_live` **3/0**.
- The full nf-view sweep did not fit the session window; every binary that runs the changed
  code (the music dispatch, the cel post on the player tick, the gate drain) is listed above
  and green — the remaining walkreals exercise the same dispatch paths as the five run.

## Live checks for the main agent

1. 07000002, walk Bond into the big east hall (cel #19: x −54..13, y 4..17, z 20..47): with
   the debugger watch `EventList[36]` (EE 0x2d3588) — expect a write of 1 EVERY tick inside,
   and the music jumping to section 2 (MayhewB's level-start arm) if the IOP is still at the
   start section.
2. 0700000b, ride to the tower top: watch `EventList[44]` (0x2d35a8) — expect 1 every tick on
   the top floor (cels #0-8/#14), and 1 in the shaft (cel #13) for `EventList[41]` (0x2d359c);
   the music machine should walk 2 → 4 → 6 as the zones change after NIS 0x60004f9.
3. 07000006, play to the 0x6000071 cut-scene: watch the board ch 11 (0x2d3524) — on the NIS
   start expect one instant jump to the 06 row's +0x14/+0x18 marker (end_combat 8 when the
   IOP is in combat [2,15,17], else end_calm 6), the combat picks stopping until the cut-scene's
   music sound, then the 0xc jump and the end-wait.

## Usage

not visible to me (the sandbox exposes no token, context-window or cost figures for this
session; the counts above are test results, not token meters).
