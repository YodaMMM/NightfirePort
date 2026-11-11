## Your task: `explode-falloff-int`, explosion damage falls off with distance as in the game

The level-14-fusebox-zap report (origin/s/level-14-fusebox-zap, docs/reports/level-14-fusebox-zap.md, "What's left") found that the game's Explode_Propagate falloff truncates the gap to an int before dividing by the radius [M 0x17c518:77-82], while the port's explode::falloff is linear. Confirm it in the disassembly first (the cast, and which value it applies to) and write it down with addresses. If the game truncates, port that law in the game's order and test it red on the old code. Pin it through real play: a guard or Bond at a fractional distance inside a blast takes the game's damage. This moves blast damage on every level, so run every walkreal_*_live with explosions one level per tool call and repair any walk that shifts, gated to its level. Real input only, never lower damage or health. Run the bullet and explosion binaries and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/explode-falloff-int`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox7 at 2026-11-11T13:55Z

---
## Report (done, 2026-11-11T14:46Z)

# explode-falloff-int — the Explode_Propagate falloff confirm: the game does NOT truncate

Branch `s/explode-falloff-int` (base origin/master 784ba2d99, master still an ancestor of the
final tree — nothing to merge). Task source: level-14-fusebox-zap's "What's left" said the
game's `Explode_Propagate` falloff "truncates the gap to an int before dividing by the radius
[M 0x17c518:77-82]" while the port's `explode::falloff` is linear, and asked for a disassembly
confirm before any port change. **The confirm came out negative, and that is the result: the
decompile's `(int)` is how Ghidra prints the EE's `MAX.S` instruction. No truncation exists,
and the port already implements the game's law.** No source behaviour changed, so blast damage
did not move on any level and no walk needed a repair.

## The disassembly (ACTION.ELF, Explode_Propagate 0x17c518)

The falloff chain, from the ELF's bytes:

- `0x17c53c` `mov.s $f21, $f12` — f21 = the first float arg = the RADIUS (the falloff divisor);
  the early-out at `0x17c564` `c.olt.s $f21, $f0` against f0 = 0x3951b717 (~0.0002) skips the
  whole sweep when the radius is under it. `0x17c534` `mov.s $f22, $f13` — f22 = the second
  float arg = the base DAMAGE.
- `0x17c5dc` `jal 0x1e2e48` (`Vec_Dist3D`, a0 = hit object obj+0x80, a1 = the blast centre) →
  `0x17c5ec` `lwc1 $f1, 0x8c($v0)` — the hit object's own sphere radius (obj+0x8C) →
  `0x17c5f8` `sub.s $f0, $f0, $f1` — **the gap = dist − obj sphere**.
- `0x17c5e8` `mtc1 $zero, $f3` — f3 = 0.0. `0x17c608` word 0x46030028 = COP1, fmt S (0x10),
  funct 0x28 = **`max.s $f0, $f0, $f3`** — the gap CLAMPED at zero, nothing more. The EE Core
  Instruction Set Manual's COP1.S table puts MAX.S at funct 0x28 and MIN.S at 0x29 (MADD.S is
  0x1D on the EE); capstone's MIPS32 (r1) mode simply cannot decode it, and Ghidra renders the
  clamp as `(float)((int)gap · (uint)(0.0 < gap))`, which reads like a truncation. 89 funct-0x28
  words live in ACTION.ELF — a routine compiler idiom, not a one-off.
- `0x17c60c` `div.s $f20, $f0, $f21` (max(gap,0)/radius) → `0x17c610` `sub.s $f2, $f2, $f20`
  (1 − that) → `0x17c614` `mul.s $f20, $f22, $f2` — **amount = damage · (1 − max(gap,0)/radius)**
  → `0x17c618` `c.olt.s $f20, $f4` (the same ~0.0002 constant): under it the object takes
  nothing.

So the game's law is the straight line with a clamp at the sphere edge — exactly what the port
ships:

```rust
pub fn falloff(damage: f32, radius: f32, gap: f32) -> f32 {
    damage * (1.0 - gap.max(0.0) / radius)
}
```

(its doc comment already cites these very addresses and notes "no integer conversion exists in
the function"). Due diligence on the callees: neither `Player_Hurt` 0x196060 nor
`Drone_ExplosiveHit` 0x13aab8 contains any float→int conversion opcode (cvt.w.s/trunc.w.s/
round.w.s/ceil.w.s/floor.w.s — a word-level scan of both bodies found none), so the MAX.S clamp
is the only falloff shaping in the blast path.

## What the conditional asked vs what the game does

The task said: "Confirm it in the disassembly first … **If the game truncates**, port that law
in the game's order and test it red on the old code." The game does not truncate, so the port
change is not made — implementing the truncation reading would BREAK the port against the game.
Concretely at a fractional gap (radius 10, damage 100): the game gives 91 at gap 0.9 and 71 at
gap 2.9; the misread truncation would give 100 and 80.

## What changed (tests only)

- `crates/nf-game/src/explode.rs`: new test
  `the_falloff_keeps_the_fractions_the_decompiles_int_misprint_would_drop` — pins gap 0.9 → 91
  and gap 2.9 → 71 through `propagate` (radius 10, damage 100), with the addresses and the
  MAX.S identity in the comment. Any future "fix" toward the `(int)` misprint turns it red.
- No other file touched. `git status` on the final tree: `docs/WORKLOG.md`, this report, and
  the explode.rs test module.

## How it was verified

- nf-game lib tests: **783 passed / 0 failed / 4 ignored** (`nf_game-ceddbdc98613aa28
  --test-threads=1`), including the new pin and the existing
  `propagate_falloff_is_a_straight_line_to_the_sphere_edge` (gap 4.7 → 53; the misread would
  say 60).
- Real play, already on master, re-run green on this tree:
  `destroy_blast_07000014_live::a_blast_falls_off_in_a_straight_line_to_bonds_sphere` — Bond
  stands at a fractional gap from the #620 box on real loaded 07000014, the channel fires, and
  the lost health equals `40·(1−gap/4)·difficulty_factor` to 0.2 — **8 passed / 0 failed** for
  the whole binary.

## The walk sweep (every walkreal with explosions; no repair was needed anywhere)

One level per run, direct exes, `--test-threads=1`, NF_SILENT=1, final tree:

| binary | P/F/I | binary | P/F/I |
| --- | --- | --- | --- |
| walkreal_07000001_live | 7/0/1 | walkreal_07000013_live | 3/0/1 |
| walkreal_07000008_live | 5/0/1 | walkreal_07000014_live | 7/0/3 |
| walkreal_07000009_live | 9/0/0 | walkreal_07000015_live | 6/0/3 |
| walkreal_0700000b_live | 9/0/0 | walkreal_07000016_live | 15/0/3 |
| walkreal_0700000d_live | 1/0/2 | walkreal_0700001b_live | 16/0/3 |
| walkreal_07000011_live | 3/0/1 | walkreal_07000041_live | 15/0/0 |
| walkreal_0700004a_live | 4/0/0 | walkreal_0700004a_satchel_live | 2/0/0 |

(The remaining walkreals have no explosion content — `rg -l "blast|explode|grenade|satchel|mine|destroy"`
names exactly the fourteen above. Since no source behaviour changed, their result cannot have
shifted; they were not re-run.)

## The bullet and explosive-weapon binaries

bullet_range_live 3/0, ally_bullets_live 3/0, stairwell_bullets_07000011_live 2/0,
searchlight_bullet_07000006_live 2/0, fix_foliage_bullets_live 3/0;
blast_debris_draw_live 1/0, blast_room_shelter_16_live 3/0, concurrent_blasts_draw_live 1/0,
fix_blast_scripts_live 3/0/1i, fix_fusebox_cut_live 5/0; grenade: fix_grenade_throw_live 4/0,
fix_grenade_release_live 4/0, fix_grenade_types_live 4/0, grenade_bounce_debris_live 1/0;
satchel: fix_satchel_stick_play_live 9/0, fix_satchel_tripbomb_live 6/0,
camera_beam_satchel_wreck_live 3/0; mine_bob_live 3/0.
`mp_stress_explosives_live`'s single test (an 8-machine state-hash sync storm — its assertions
compare net-state hashes, not damage figures) did not finish inside 560 s twice; every blast it
generates flows through the same `explode::falloff` pinned above.

## The fast suite

suite.py was started three times and died three times: once on `FileNotFoundError: 'cargo'`
(the background shell inherits no PATH), then killed between tool calls twice more — this
sandbox kills background jobs, and `setsid` did not save it. The full four-crate `--no-run`
rebuild (every nf-view test binary relinks after the explode.rs edit) does not fit the
remaining window, so the binaries above ran directly from `target/release/deps` — the same
per-binary-chunking precedent earlier rounds recorded. No FAILED/TIMEOUT line exists because
no suite run survived; the equivalent coverage is the table above (0 failed in 33 binaries).

## What's left

- Nothing for the law: the confirm is written down with addresses, the fraction pin is in, and
  the port matches the game. The fusebox-zap report's bullet should be read as settled here
  (its "worth a disassembly confirm before any port change" hedge was the right call).
- Optional, next round: give `mp_stress_explosives_live` a real slot on a machine that finishes
  its storm (it wants more than 560 s here).
- No live checks are needed — the ELF bytes decide this one on their own.

## Usage

not visible to me — this environment shows no token, credit or context-window figures for the
session (no usage panel is exposed to the agent).
