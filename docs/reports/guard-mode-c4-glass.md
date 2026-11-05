# guard-mode-c4-glass — report

Branch `s/guard-mode-c4-glass`, cut from origin/master 2f0c88e2c. Status at the sandbox
deadline: **both code fixes landed and are proven; the 08 real-damage walk re-pin is
partially done — two Agent-difficulty walks still fail**, so the task is `blocked` and the
next session continues from this branch.

## 1. The `+0xc4` attack-type write at creation — DONE

**Game law** [M]: `NDrone2_GetDroneTypeAttackTypeFriend` 0x14cbd8:22-23 writes
`+0xc4 = GetDTYPENEW(drone, +0xc7, 0)` (the type's block-0 mode) *before* the mode byte
(:24-26). It runs for **every** drone: `NDrone2_DoModeSettingsNEW` 0x14c600:80 calls it, and
`DoModeSettingsOLD` tail-calls NEW (0x14cfa0:16) — so the decompile has two `+0xc4` writers,
not the one the port's old comment claimed (the astronaut's 0x14b300:725). The kind-0x18
astronaut arm then rewrites `+0xc4 = 0x1d` [M 0x14b300:722-727]. Readers: the abseil landing
`+0xc5 = +0xc4` [M AbseilSlide 0x169730:34-36; AbseilStepOff 0x1738e0:21] and the
GoToGoalPosition arrival [M 0x165640:78-79]. `GetDTYPENEW` 0x14c938: block 0 + behaviour
prop 0x10 (bit 16 of the behaviour word 0 — bitDescs 0x2c7250 word 0 / mask 0x10000 shift
16, decoded from work/elf/ACTION.ELF) → 0x10, dtype 0xa/0xb → 9/0x12/10, else the block-0
tail 0.

**Port fix** (`crates/nf-game/src/drone.rs`): `type_setup` now writes
`self.attack_mode = self.dtype_of(d, self.dtype, 0)` in the game's order (before the mode
byte, so the 07000007 block-0 prop writes land in sequence), plus
`if self.kind == 0x18 { self.attack_mode = 0x1D; }`, and the field comment is corrected.

**Disc census** [D `nf-extract drones`]: the five rope placements (07000011 p258/p280,
0700004a ×3) are all dtype 3, normal block 0, param 18 = 0xa8d2e0c1 (bit 16 CLEAR) → their
game attack type is **0** — the port's landed mode on 07000011 is unchanged, now *derived*
instead of hardcoded. The ambush-bit scan over the walk levels: 1/37 drones on 07000011 set
the bit (a stairwell walker, mode 2 — never reaches the mode-7 arrival arm), 0/26 on
07000008, 0/20 on 0700000a, 0/20 on 07000001.

**Test** `crates/nf-game/tests/attack_mode_c4_live.rs` (red-first verified: with the write
reverted the test fails on the ambush assert; green after):
- disc abseiler params → `attack_mode == 0` (and rope mode 0x1c);
- the same placement with param 18 bit 16 set → `attack_mode == 0x10` [M 0x14c938:10-14];
- the astronaut skin 0x05000078 (kind 0x18) → `mode == 0x1d` and `attack_mode == 0x1d`
  [M 0x14b300:723/725].

## 2. The guard rounds' glass pass armed from the record — CODE DONE, WALK RE-PIN PARTIAL

**Game law** [M]: `Bullet_update` 0x12e7b0 (disasm 0x12f030-0x12f058, hand-decoded from
work/elf/ACTION.ELF, LOAD vaddr 0x107100 → file off = vaddr − 0x106100) re-arms the flying
round's HITTEST+0x9a every tick: `+0x9a = ((weapon_data+0x6c & 0x800) == 0) << 5` — the
flight's contact walk passes surface-byte-0x10 (glass) contacts for every record without the
bit [M Collide_Update 0x1ea398:216-229]. Nothing here touches any guard AI state — the
round's *flight* stops on glass or passes it.

**Port fix** (`crates/nf-game/src/drone.rs`): all five guard-side spawn sites
(`close_combat_impact` 0x1789c0, `fire_weapon` 0x178f68, `spawn_copter_bullet`
0x17d828/0x17d9f0, the GT turret `GTEvent::Fire` 0x180498, the space-laser record-107
`Bullet_init`) now set
`glass_pass: u32::from_le_bytes([w[0x6c], w[0x6d], w[0x6e], w[0x6f]]) & 0x800 == 0` — the
per-round law `update_player_bullets` already computes for Bond's rounds (drone.rs
14891-92). No RNG draws added, removed or reordered (the computation is a pure record
lookup).

**Re-pin (the 08 station route first)** — `crates/nf-view/tests/walkreal_07000008_live.rs`:
the armed pass made the Agent (d2) and 00 Agent (d3) whole-mission walks die at the
platform mouth — exactly the shortcut's documented cost (the old deviation comment cited
"#23/#19 volleys through the top-station glass" at tick 25178; measured now: d2 died at
tick 25209, 33 hits for 100.0, #23 point-blank 5 × −4.2 at d 1.6). Re-pin so far: a
"stairwell mouth" `fight_close_guards` stand at difficulty ≥ 2 (a player arriving at the
stairwell's foot fights the wave from the foot's cover before stepping onto the exposed
platform) and the "lower level east" platform-mouth stand gate extended 3 → 2. Result:
**Operative (d1) real-damage and the plain walks pass (4 passed in the binary's trio run)**;
the d3 death moved from 25209 to 27933 (a later spot) but **both Agent variants still fail**:
at the mouth stand guard #19 held "out of sight" through six 900-tick stands (the foot's
sight line into the platform is blocked) and the stand cost ~1000 ticks for 3 kills
(#20/#23/#24 down).

### What's left (the next session picks up here)

1. Finish the 08 Agent re-pin: the mouth stand cannot see #19/#20 from the stairwell foot —
   move the stand onto the platform ("the lower level east" arrival now fires for d2/d3;
   verify whether the d2 run survives with it) or fight from the "gondola mouth" spot; then
   re-run the whole `walkreal_07000008_live` binary and pin the remaining death (d3 tick
   27933 at (111.0, 13.5, −179.7) — need the killer list from that run).
2. Re-run every `walkreal_*_live` binary (lesson 7) plus `npc_doors_live`,
   `fix_guard_misc_live`, `abseil_damage_*`, `rope_guard_hitbox_*`, `glass_pass_every_level_live`,
   `rocket_glass08_live`, `rocket_stop_plane08_live`, `stairwell_guards_07000011_live`,
   `abseil_death_live` (the last two + abseil + lib already re-run green — see below), and
   the full suite: `NF_ISO=$NF_ISO cargo test --offline --release -j2 -p nf-game -p
   nf-formats -p nf-audio -p nf-view -- --test-threads=1`.
3. A guard-side glass-crossing unit proof (a guard round reaching Bond through an intact
   pane) would pin the flight change directly; the walk deaths above are the live evidence.

### Verification run this session (each with pass counts)

- `attack_mode_c4_live`: 1 passed (red-first verified by reverting the write).
- `nf-game` lib: **679 passed, 5 ignored** (after both fixes).
- `abseil_death_live`: 5 passed.
- `stairwell_guards_07000011_live`: 3 passed; `abseil_elevator_live`: 3 passed.
- `walkreal_07000008_live` real-damage filter: 4 passed / 2 failed (the two Agent variants,
  above). The full suite did not fit before the sandbox deadline.

## Commit discipline

Only these files were committed: `crates/nf-game/src/drone.rs`,
`crates/nf-game/tests/attack_mode_c4_live.rs`, `crates/nf-view/tests/walkreal_07000008_live.rs`,
`docs/WORKLOG.md` (appended only), this report. `git status --short` checked before every
commit; no game bytes, tokens, URLs, env scripts, logs or dumps committed.

## Live checks for the main agent

- `--bond 108.6,28.4,-121.65,270,0` (level 08, after the stairwell descent): Bond at the
  platform mouth — guard bursts from the west pair should now strike through the intact
  panes (sparks on Bond, not on the pane), and the pane still breaks on aimed fire.
- `--bond 15.1,33.5,-104.5,315,10` (level 11 stairwell top): a landed abseiler (after the
  rope chain) should stand in the generic-guard mode (attack type 0) and fight Bond from
  the stairwell.

## Usage

Environment-visible token figures for this session, copied exactly:

- Explore subagent 1 (attack_mode research): `Tokens -> prompt:122512 completion:6836
  total:129348` (from the subagent's final report footer).
- Explore subagent 2 (glass_pass research): `Tokens -> prompt:125216 completion:11430
  total:136646` (from the subagent's final report footer).
- My own session's input/output tokens, context used and cost: **not visible to me**.
