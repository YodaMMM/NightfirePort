# guard-alt-voice-set — the six talk sites take the game's +0x108 serial; the reachability correction and the serial hand-out verified live

Branch `s/guard-alt-voice-set` from `origin/master` (22d5836b7). The changes touch
`crates/nf-game/src/drone.rs` and add one test file; the evidence is DISASSEMBLY-based (the
slim `work/decomp/` has no main-binary C — the work below disassembles
`work/ee_snapshot.bin`, file offset = EE address, with capstone MIPS64-LE; every [M] below
cites the instruction addresses).

## 1. The record, the counter, the reset

- `Drone_SM_InitObject` 0x175838: `lw $v1, 0xe0($t1)` — the SMI record is obj+0xE0; the
  counter is the halfword at **0x299310** (= DAT_00299090 + 0x280; `lui 0x2a; addiu -0x71f0`
  then `lhu/sh 0x280($a3)`) [M 0x175848-0x175874]; the serial stored at SMI+0x108 is
  **counter + 1** [M 0x175878 `sw $a2, 0x108($v1)` with a2 = v0+1] — the first object of a
  level takes serial 1 (odd). The `beqz $t1` early-out skips the increment for a null object
  [M 0x175840]. SMI+0x10C/110/114/118 take obj+0xF4, +0x120/+0x12C = 0, +0x130 = 0x175778
  [M 0x175880-0x1758a0].
- The reset: `Drone_LevelReset` 0x13a5a8 zeroes it [M 0x13a614 `sh $zero, 0x280($s0)`] — the
  serials restart every level load.

## 2. Who takes a serial, in what order

`Drone_SM_InitObject`'s only caller is `NDrone2_PostLoad_Init` 0x14d890 [M the single `jal` at
0x14d8f0; the full-binary scan for the address and the branch targets found no other caller].
PostLoad_Init's callers (the branch/jal scan):

1. `Drone_PostLoad_Init` 0x136938 — the level load: the loop walks the drone list at
   0x299094 from its head [M 0x136b80-0x136ba8], and `LList_Add` 0x1cca28 APPENDS (the head
   stays the first-created node [M 0x1cca48/0x1cca6c decode]) — **the serials follow the
   creation order, 1-based**.
2. `DroneSpawner_SpawnDrone` 0x137cb0 — the spawn takes the next serial [M 0x137e04].
3. `Drone_CoderCreate` 0x13a108 — the NIS spawn takes the next [M 0x13a19c].
4. `BOT_init` 0x1249e8 — MP only.

**The spawner templates have no object until their spawn** (the port's own drone.md model,
confirmed by the spawn-time PostLoad call) — so the load-time serials cover exactly the
non-template drones, in placement order; every spawn (spawner or CoderCreate) takes the next.
On 07000014 this is THREE load-created guards (verified live below) and the rest at spawn.

## 3. The six sites, each compared with the binary

| site | read | bit clear (base) | bit set (alternate) |
| --- | --- | --- | --- |
| `NDrone2_AttackTalk` 0x13eac0 | 0x13ecb4 | 0x11b + Rand(7) | 0x122 + Rand(7) |
| `NDrone2_HitTalkExtra` 0x13ed38 | 0x13ee5c | 0x133 + Rand(5) | 0x138 + Rand(5) |
| `NDrone2_SeenPlayerTalk` 0x13f1d0 | 0x13f2f4 / 0x13f328 / 0x13f35c / 0x13f380 | 0xf3 / 0xf7 / 0xfb / 0x10d (+Rand(2)) | 0xf5 / 0xf9 / 0xfd / 0x10f (+Rand(2)) |
| `DroneFunc_InjuredTalk` 0x13fa20 | 0x13fb10 | 0x133 + Rand(5) | 0x138 + Rand(5) |
| `DroneFunc_RunForCoverTalk` 0x13fd08 | 0x13fdf8 | 0x11b + Rand(7) | 0x122 + Rand(7) |

- The AttackTalk site is the **type-7** arm (the jump table at 0x2f4a30, entry 7 → 0x13ec78),
  level-gated to 14/15/16, odds word never set → the line speaks whenever the busy/health
  gates let it [M 0x13eccc-0x13ed10].
- The SeenPlayerTalk sites are per-LEVEL (the table at 0x2f4b20: 0x14 → 0x13f2d8, 0x15 →
  0x13f30c, 0x16 → 0x13f340): the first `Rand(2)` picks the level's own pair (1) vs the common
  tail pair (0), the second `Rand(2)` picks the offset within the pair, the bit picks the
  pair's base [M 0x13f2d8-0x13f390].
- The InjuredTalk/RunForCoverTalk functions open with the `MPSettings+0x184` gate
  (`0x2a4924` [M `lw $v0, 0x4924($v0)` + `beqz`]) — dead in both modes ([L]
  ee_start_05.bin @0x2a4924 = 0, the port's established `mp_gate`); their tables are updated
  with the parity for the record.

**Reachability correction** (the task's "compare each with the decompile" found this):
HitTalkExtra's types-5..7 arm is **game-dead** — PainTalk 0x13eeb8 calls HitTalkExtra ONLY
from its type-0x10 arm (the type table at 0x2f4a80, entry 15 → 0x13ef50; every 5..7 entry →
0x13ef0c, the default 0xf/0x17 pain grunt). The type-5..7 check inside HitTalkExtra is the
shared body's vestige. The LIVE parity sites on 14-16 are SeenPlayerTalk (every type) and
AttackTalk (type 7). The port's `hit_talk_extra` mirrors the game's dead arm faithfully (its
5..=7 arm was already unreachable — the port's pain_talk routes only 0x10 there).

## 4. The port's changes (crates/nf-game/src/drone.rs)

1. `sm_serial: u16` on `Drones` — the game's counter, zeroed at the reset (the constructor),
   incremented per init. The load serials are handed out in the spawner-setup block (where the
   template set is known), skipping the templates, in list order; the CoderCreate arm and the
   spawner `Action::Spawn` arm take the next serial.
2. `walk_anim` reads the serial directly [M 0x13ccc0's `lw $v1, 0x108($s0)` + the %3 chain]:
   `(self.sm_id % 3)` — behavior-neutral (the old `sm_id + 1` equals the serial the port now
   stores for load drones).
3. The six sites use `self.sm_id & 1` (bit set → the upper base). The draw counts are the
   game's own — the parity never adds a draw. The "stands in" comment and the
   out-of-pass/fail note are removed; the SeenPlayerTalk 14-16 arms (previously "not
   modelled") are added with the game's two-draw shape.

## 5. Verification

- **nf-game lib 691 passed / 0 failed / 5 ignored** — including the new
  `alt_voice_set_follows_the_serial_parity`: HitTalkExtra odd → 0x138..0x13c / even →
  0x133..0x137; SeenPlayerTalk (level 14, 64 draws) every line's half matches its serial's
  low bit and both halves occur; AttackTalk type 7 odd → 0x122..0x128 / even → 0x11b..0x121.
- **The live serial hand-out verified** [L the run]: 07000014 loads THREE drones (#0 serial 2,
  #1 serial 3, #22 serial 1 — types 6/6/0x10) — the parity pair (2 even, 3 odd) exists in real
  play.
- The voice + ai binaries (single-threaded): `guard_barks_07000013_live` **4/0**,
  `escort_follow_07000001_live` **1/0**, `fix_06_driver_live` **2/0**,
  `hostage_killer_07000002_live` **5/0**, `ai_traces_live` **4/0**.
- **The full fast-suite run did not complete in the window** (tools/test/suite.py started in
  the first minutes; no total line by the deadline — no FAILED/TIMEOUT lines either; the
  final-tree re-run did not fit). The nf-game build + lib + the five binaries above are the
  quoted runs; no other level's code path changed (the sm_id renumber is behavior-neutral for
  load drones, and the talk arms only add the game's own draws on 14-16).

## 6. What's left

1. **The live parity pin on 07000014**: the test exists
   (`guard_voice_parity_07000014_live`) and is `#[ignore]`d with the measured state — the
   stealth opening + one-hit kills produced no 14-16 line. The next run walks the sighting
   legs (the mezzanine mouths' lane after the shaft — walkreal_07000014's legs 1e+) where
   guards SEE Bond (SeenPlayerTalk fires on every type) and asserts both halves against the
   serials.
2. The full suite on the final tree.

## Live checks for the main agent

1. **A 07000014 serial on hardware**: with the debugger, break `Drone_SM_InitObject` 0x175838
   during the level load and read the counter at 0x299310 across three consecutive
   non-null-object calls — expect 0 → 1 → 2 → 3 with the stored SMI+0x108 = 1, 2, 3 (the port
   measured [L] the same via its own counter: the three load drones at serials 1-3). Then
   break a SeenPlayerTalk parity read (0x13f2f4) during play and confirm the guard's
   SMI+0x108 low bit picks the pair the port's test asserts.
2. **A type-7 guard on 07000014**: confirm at least one of the level's guards (load or
   spawner-spawned) carries voice_type 7 — the AttackTalk parity arm only fires for that
   type (the port's three load guards are 6/6/0x10 [L]).
3. **The 0x10 SeenPlayerTalk tail arm** (0xe9 + Rand(3) [M 0x13f3cc-0x13f3d4]) — the port
   still returns for type 0x10; confirm the game speaks it on 14-16 and whether the port
   should model it (out of this task's parity scope).

## Usage

not visible to me — this session exposes no token, credit, context-window or cost figures to copy.
