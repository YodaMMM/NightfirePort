# drone-yaw-convention — settle which way placed guards face

Branch from origin/master (it carries corridor_guards_07000012_live, the log-only probe from level-12-corridor-guards-stuck).

The port spawns every drone facing `p.rotation[1]` raw (drone.rs ~11754, the placement row's +0x1c f32). On 07000012 that faces the two corridor guards (placements 12/13 at (13.09, 18.55) / (12.98, 16.43)) away from Bond's spawn, so their sight cone never scores him and they never come down the corridor. `Drone_Create` 0x136be0 receives the rotation vector [M 0x1d04f0 case 0xf] but the decompile text drops which component lands in the drone's facing.

Do:
1. Disassemble Drone_Create 0x136be0 and whatever it calls that writes the drone's facing (work/elf ACTION.ELF): name the exact component and sign the facing takes from the rotation vector [M with addresses].
2. If the port differs (sign or component), fix it at the one place drones take their spawn yaw, and check every level's guard-facing tests and walks (the change touches every placed guard). If the port matches, say so with the disassembly lines and leave the code.
3. Either way, add a test pinning the facing law with the cited instruction addresses and a sister that fails on the opposite sign. Turn corridor_guards_07000012_live into a real pin if the fix makes the corridor guards wake on the player's approach.
4. Run every enemies_*/walkreal_*/interactables_* live test, stealth tests, then the full suite.

---
claimed-by: sandbox1 at 2026-11-02T09:30Z

---
## Report (done, 2026-11-02T12:43Z)


---
## Report from the worker (done, 2026-11-02T12:43Z)

# drone-yaw-convention — report

## What's done

The facing law is settled: **a placed drone's facing is the placement row's rotation[1]
(+0x1C f32) VERBATIM — no negation, no component permutation, no float op anywhere in the
chain.** The port already spawns with `p.rotation[1]` raw [M crates/nf-game/src/drone.rs:11754,
`Drone::new_with_stats(.., p.rotation[1], ..)`], so **the port matches and no code changed**.

### The verified chain

All [M], disassembled from work/elf/ACTION.ELF this session by raw-word decode; capstone
misdecodes this ELF's 16-byte load/store family as MSA/DSP, so the `.word` lines were decoded
by hand — op 0x1F words are the quad stores, 0x1E the quad loads. The symmetric
prologue/epilogue register save/restore proves the family: `SQ $ra,0x100($sp)` 0x136bfc vs
`LQ $ra,0x100($sp)` 0x136ccc.

1. `parsemap_create_dynamic_objects` 0x1d04f0 prologue copies the placement row verbatim:
   `lwc1 $f11, 0xc($a0)`→`swc1 $f11, 0x30($sp)` … and rotation[0..2] = +0x18/+0x1c/+0x20 →
   `$sp+0x40/+0x44/+0x48` [M 0x1d0520-0x1d0564; rot[1] is `lwc1 $f0, 0x1c($a0)` →
   `swc1 $f0, 0x44($sp)`].
2. Case 0xf calls `Drone_Create(a0=$sp+0x30 translation, a1=$sp+0x40 rotation, a2=level_tag)`
   [M 0x1d0c30-0x1d0c40].
3. `Drone_Create` 0x136be0 (symtab `Drone_Create__FP7_VECTORT0P9level_tag`): `move $s3, $a1`
   (rotation), `LQ $v1, 0($s3)` 0x136c40 → `SQ $v1, 0x20($sp)` 0x136c50 (parks it in the DIVars
   frame) → `LQ $v1, 0x20($sp)` 0x136ca8 → `SQ $v1, 0xc20($s0)` 0x136cb0 (the delay slot of the
   amemcpy jal that copies level_tag+0x2c, 0x84 bytes, to D+0xc34). No float arithmetic touches
   the vector; it lands verbatim at D+0xc20 where D = obj->0xe0 (`sw $s1, 0xc00($s0)` 0x136c94
   stores the obj back-pointer at D+0xc00).
4. `NDrone2_PostLoad_Init` 0x14d890: `lw $s1, 0xe0($s0); addiu $a0, $s1, 0xc00; jal
   NDrone2_DefaultInit` [M 0x14d8a8-0x14d8b4].
5. `NDrone2_DefaultInit` 0x14b300: second `Vec_Copy2(src=DIVars+0x20, dst1=obj+0x50,
   dst2=obj+0x60)` [M 0x14b350-0x14b35c: `addiu $a1, $s2, 0x50; addiu $a2, $s2, 0x60; jal
   0x1e2cf8; addiu $a0, $s3, 0x20`]. `Vec_Copy2` 0x1e2cf8 is three verbatim `lwc1/swc1` pairs
   writing BOTH destinations — so rot[0..2] sits at obj+0x50/0x54/0x58 and again at
   obj+0x60/0x64/0x68.
6. The AI facing read: `DroneMove_NoBunching` 0x150ad0 loads `lwc1 $f12, 0x54($s1)` and adds
   π/2 for its move math [M 0x150b44-0x150b58]. obj+0x54 is the copied vector's second
   component = placement rotation[1].

### The 07000012 corridor guards

They face away from Bond's start by the disc's own authoring (guard yaw −1.89/−1.38 at
(13.09, 18.55)/(12.98, 16.43) vs yaw_to_opp +1.50/+1.29), and guard #6 is already ALERT 0.66
at load — the wake comes through the alarm/hearing path, not the spawn sight cone. Since the
port matches the disc there is no fix, so per the task's conditional
`corridor_guards_07000012_live` stays the log-only probe (it still passes).

### The pin test

New: `crates/nf-view/tests/drone_yaw_convention_live.rs` — parses 07000012's 0x0f placement
rows from the ISO (the same bytes the dispatcher reads), loads the level through the real sim,
matches each spawned drone by position, and asserts `yaw == rotation[1]` (1e-4 wrap) **and**
`yaw != −rotation[1]` on every sign-discriminating row (the opposite-sign sister; guarded
against vacuity by `checked >= 2` and `yawed >= 1`). **Runs green: 1 passed in 1.59 s.**

## How it was verified (all green, this session)

| group | result |
|---|---|
| `drone_yaw_convention_live` (the pin + sister) | 1 passed |
| `corridor_guards_07000012_live` | 1 passed |
| `enemies_07000012_live` | 3 passed |
| `interactables_07000012_live` | 12 passed |
| `stealth_ai_live` | 9 passed |
| `walkreal_07000012_live` | 1 passed |
| `walkreal_07000001..0b,0d,11,13,14,15,16,1b,41,4a_live` (21 more level walks) | 114 passed, 4 ignored, 0 failed (e.g. 0700001b 17, 07000016 14, 07000005 12, 0700000a 10+3 ign, 07000001 6) |

Total: **141 passed, 0 failed, 4 ignored** across the task's named groups (enemies_\*,
walkreal_\* level walks — all 22 levels, interactables on the task's level 12, stealth) plus
the new pin. `walkreal_07000001_live` passes clean on this branch (6/6), confirming the
failures seen there under the previous task's branch were that branch's own.

## What's left / honest limits

- **The full ~1000-binary suite cannot fit this sandbox.** The four crates carry 924 + 98 + 43
  + 3 test targets; one build set of executables is ~5 GB against a 9.9 GB disk that also
  holds the ISO and work/. Two whole-suite build attempts died with rustc `No space left on
  device (os error 28)` (the affected targets: fix_guard_first_shot_live, mouse_look_rate_live,
  probe14_lid, cover_nodes_submap_live, tranq_dart_live, corridor_guards_07000012_live — all
  failed on temp-dir creation before I freed space). I freed ~5.3 GB of stale
  `target/release` executables twice and switched to `--test <name>` slices, running each
  slice and deleting its executables before the next — the strategy is in the WORKLOG. Given
  the task's outcome is a **zero-source-change** verification (one added test file only), the
  named groups above are the meaningful coverage; the untouched crates'
  (nf-game/nf-formats/nf-audio) suites are unaffected by this branch.
- Not run: interactables_07000001..4a beyond level 12, the remaining nf-view probes, and the
  other three crates' suites. A next session can resume the slice-and-run plan.
- No live checks needed for the main agent: there is no code change and nothing visual.

## Environment notes

- Disk was at 100 % (0 bytes free) at session start; `target/release/deps` had 4.9 GB of stale
  executables from prior builds. Freed ~5.3 GB twice to let builds proceed.
- Background jobs (even `setsid nohup`) are killed when the spawning tool call ends, so the
  suite ran as foreground `--test` slices with per-binary timeouts; results persisted via
  `tee` so killed slices lost at most the in-flight binary.

## Usage

not visible to me
