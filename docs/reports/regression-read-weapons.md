# regression-read-weapons — report

Branch `s/regression-read-weapons` (from `origin/master` 2f0c88e2c). 2026-11-05, finished ~19:00 UTC.

## What was done

Read every commit since 2026-10-20 touching the weapons-and-damage files, asked for each
diff what a player would see change, and checked that claim against the game's own code
before trusting it. No suspect survived; **nothing broke, so nothing needed fixing and no
fail-first test was owed.** Every commit range read is listed below with its verdict.

**File mapping.** `crates/nf-game/src/weapon*.rs` does not exist — the weapon logic lives in
`crates/nf-game/src/gun.rs` (the `weapon*.rs` stand-in; `weapon_data` records, trigger,
reload, zoom). The census is therefore `gun.rs`, `bullet.rs`, `grenade.rs` plus `hurt.rs`
(the damage half of the title: `Player_HandlePain`'s glow/health bookkeeping). Casing,
driving and gamepad were excluded as instructed; `casing.rs` commits were not read.

## The census (every commit read, with verdicts)

30 non-merge commits + 7 merge commits (each merge's delta vs its first parent maps 1:1
onto the audited branch commits — no hidden conflict content). Newest first. "checked"
means I re-derived the claim from the decomp/disassembly myself, not just from the commit
message.

| commit | date | what a player would see | verdict |
|---|---|---|---|
| 58d7f21f5 | 11-04 | none (test pin) | clean — checked: `Bullet_init` 0x12c5a0's cone block draws 3 unconditionally even at s=0 (the C at LAB_0012c7f4: FRand_MVar2+2×FRand every round); first tag v0.1.31 |
| df4012581 | 11-04 | rounds pass through glass for weapons whose record +0x6C bit 0x800 is clear | clean — checked in **disassembly**: 0x12f048 `lw $v0,0x6c($v1)`, `andi 0x800`, `sltiu`, `sll 5` → `sh $v0,0xea($s4)` arms mask 0x20 exactly when the bit is clear; first tag v0.1.31 |
| 21ffecc6c | 11-03 | a laser-burned-out satchel scrambles its digits, never beeps, never explodes | clean — checked: `Bullet_DoTrails` 0x12d890:424-426 redraws the fuse `Rand_Rand(0x1518)+0x3c` per tick when BU+0x102 < 1, skipping the beep/contact walk |
| c5635aafe | 11-01 | a player round from range detonates a stuck tripbomb/satchel | clean — checked: `Bullet_handle_object_destruction` 0x12cd98 gates on victim record `+0x68 & 0x4000` && `+0x70 & 0x80204`, laser tail `& 0x80000` |
| 03718421f | 11-01 | the space-laser impact def fix; the watch burn segment never passes the record range | clean (test-only in census files; the reach law is 34ba24ddb's, re-proven here) |
| 1ea4f6245 | 10-31 | re-applies the wall-wedge merge (health<1 kill back) | clean — checked: `Player_HandlePain` 0x1902c8:131-134 zeroes health below 1 before `Player_SetHealth` |
| 3c6e86592 | 10-31 | revert of the wall-wedge merge (removes it) | clean — net zero with 1ea4f6245 |
| 85735a9e4 | 10-31 | none (comment citation line fixes) | clean (comment-only) |
| ce22c9942 | 10-31 | pump Frinesi/snipers no longer refire while their fire script plays; the reload key reloads full-length; a run-out timer waits for a fresh press | clean — checked: `Player_WeaponFiring` 0x1a5460:601-648 — `if (fVar19 <= 0.0) return;` before the decrement, the state-9 + `0x8000` refusal, and states outside {0, 9, 0xB} refusing |
| 0617ba3e1 | 10-31 | player shots bloom per shot of the pull (capped at the clip); snipers exact only while aimed; zoom ramps ×1.025/tick on the axis, eases ×1.15 back; no trigger under water; Q-Claw assists on grapple points only | clean — checked: the player arm at 0x12c5a0 (`(short)(int)((float)n·+0x98)` bloom, the `+0xdc+0x60 & 1` aim bit vs the `+0x68 & 0x400000` flag), wiring in scene.rs:5159 feeds `spp − pending` = the game's `sppode − BL+0x936` after-decrement arithmetic; first tag v0.1.28 |
| 1b1d9988e | 10-31 | the red damage glow is a byte (no fraction, 128 per-hit cap) | clean — checked: 0x1902c8:126-130 `·21.333334`, the 128 select, the 255 select, `(char)(int)` store |
| bc10f1fae | 10-31 | the thrown satchel's mesh/digits keep the spawn alignment through the arc | clean — checked: `Mat_Align2Dir(obj+0x90, BU+0x00, …)` once at 0x12c5a0:~185; flight rewrites obj+0x90 only for spin/guided flags, absent on 0x3a/0x3b (+0x6C = 0xc08 [L]) |
| 065d6e53f | 10-31 | (drone-fire commit; its census-file hunk is the health<1 kill) | clean — same 0x1902c8:131-134 proof |
| 9153ebb2e | 10-31 | satchel digits placement/white tweak (draw-side files); the defused comment corrected | clean (comment + out-of-scope draw files) |
| 07c4710aa | 10-30 | bullet flight candidates scoped to the straddle cels (no cross-room ghost hits) | clean — the arming block (disasm 0x12f030-0x12f058) + `Collide_Pick` 0x1eb320 → `Collide_StraddleCels` 0x1eac98 chain |
| 2a305e510 | 10-27 | hurt objects build obj+0x90 with View_RotTransMatrix | clean by citation ([M 0x17e870:17, 0x1344e0:26-29]) + its disc census test green |
| 0078af5eb | 10-25 | rounds no longer stop mid-air on pass-flagged foliage/walls | clean — same disasm arming block; `Collide_Update` 0x1ea398:216-229 walks past pass-flagged hits |
| 364bdd228 | 10-24 | satchel landed stick normal + countdown digits/beep reach the draw lists | clean (draw-side fields; [M 0x12ce80:256-266, 0x12d890:435-441]) |
| 65941841d | 10-24 | none (test literals → `..Bullet::TAIL`) | clean (test-only) |
| 62a88ee67 | 10-24 | the satchel charge: fuse family arms, laser burn, landed state, ride on movers | clean — checked: `Bullet_init` fuse block (210.0 base, 300.0 for `+0x70 & 0x10000`, the `family == 0x3b` arm `((kind−0x3b)·5+5)·0x3c`, kind 0x6c → 900, burn 100 at BU+0x102; the game's 0x69 → 5.0 and 0x37 → 420/7777 arms are ported in the drone.rs spawn path); first tag v0.1.20 |
| 00f5508de | 10-23 | none (comment citation fix) | clean |
| 34ba24ddb | 10-23 | the laser watch's beam reach drops the 210.0 stand-in for the record's 4.0/8.0 | clean — the death bound is `weapon_data+0x1C` (see 31236aa08's proof); census pins 4.0/8.0 [L]; first tag v0.1.18's successor line |
| 31236aa08 | 10-23 | bullets die at their record's real range (+0x1C), not at 210 units | clean — checked: `Bullet_update` case-1 death compares record `+0x1C` against the travelled accumulator (0x12e7b0 C line 158); BU+0xFC is the fuse clock, not a distance; first tag v0.1.18 |
| 0fdae9614 | 10-22 | none | clean (comment scrub) |
| 117b077bc | 10-22 | none | clean (comment scrub) |
| 272cea833 | 10-22 | none | clean (comment scrub) |
| 8b79e076a | 10-21 | none (comment on CompHitData sort) | clean |
| 67e8a915c | 10-21 | none (doc of an unreached port) | clean |
| 3b6592800 | 10-21 | none (citation address fix `Dist2Tri_PointGeom` 0x1dfac8) | clean |

Merges touching the census files (deltas vs first parent all map onto the above):
3c7d4cc34, a2f9c090b, 1e6c464ba, 4415b4070, 2dea42943, a028ea46b, e06a9626a.

## Breaking commits

**None.** Every player-visible claim made by these commits matched the game's own logic
where I could re-derive it, and the positive controls (below) are green. The one
deliberate deviation found is documented in-code and pre-dates the window: the
player-thrown mine 0x37 keeps fuse 420 instead of the game's 7777 for a class-3 owner
([M 0x12c5a0:223-228]) because the port has no detonator yet (decided 2026-10-13,
bfa9b2818 — outside this task's window). If the detonator lands, that fuse must flip to
7777. No breaking commit ⇒ no first-tag attribution was needed.

## How it was verified

Reading was not trusted alone. Positive controls through the real path, all green
(`NF_ISO=$NF_ISO cargo test --offline --release -- --test-threads=1`):

- `nf-game` lib: **679 passed, 0 failed, 5 ignored** — carries every pin the window added
  (cone 3-draws, player bloom/aim gate, zoom ramp, refire/run-out, fuse family, defused
  redraw, round_pick, pain_glow byte).
- The window's own live/integration binaries: `weapon_handling_live` **3/0**,
  `tripwire_shot_live` **4/0**, `satchel_laser_scramble_4a_live` **1/0**,
  `laser_watch_0c_live` **3/0**, `bullet_range_live` **3/0**,
  `fix_foliage_bullets_live` **2/0**, `bullet_range_census` (nf-formats) **1/0**,
  `fix_06_fire_live` **3/0**, `drone_weapon_drop_live` **3/0**.
- Shared-machinery guards (`npc_doors_live` **6/0**, `fix_guard_misc_live` **6/0/1 ign**)
  and level walks `walkreal_07000005_live` **14/0**, `walkreal_0700000d_live` **1/0/2 ign**.
- `nf-game` whole package: **107/107 binaries ok**. `nf-formats` + `nf-audio`: **9 ok** with
  exactly the 3 allowed `disc_golden` failures on the slim ISO (BIGF goldens:
  `/DRIVING/MIS01.MUS`).

The 1/240-vs-1/60 HeadlessApp proof was owed only for a proven suspect; no suspect was
found, so there was nothing to pin at the alternate tick rate. The shipped pins above all
run the fixed 60 Hz Sim path.

## Commit ranges read

- `git log --since=2026-10-20 --no-merges -- crates/nf-game/src/{gun,bullet,grenade,hurt}.rs`
  → 30 commits, each diff read in full (the census files; ~810 changed lines).
- `git log --since=2026-10-20 --merges` on the same paths → 7 merges, each diffed against
  its first parent to confirm no unique conflict content.
- Decomp files opened: `Bullet_init` 0x12c5a0, `Bullet_update` 0x12e7b0,
  `Bullet_DoTrails` 0x12d890, `Bullet_handle_object_destruction` 0x12cd98,
  `Player_WeaponFiring` 0x1a5460, `Player_HandlePain` 0x1902c8.
- Disassembly opened (capstone, ACTION.ELF): 0x12f020-0x12f090 (the flight pick/mask
  arming block) for the glass-pass and pass-flagged-hit claims.

## What's left

- Nothing for the audit itself: the window is clean as far as the game's own logic can
  settle it.
- The full 4-package suite did not fit this sandbox (nf-view alone has 970 test binaries,
  715 of them ISO-loading live tests; the sandbox has a 9.9 GB disk and a 10-minute
  per-call cap — the first whole-suite attempt died on "No space left on device" and was
  re-run package-by-package). Everything that exercises the audited code ran green; the
  rest of nf-view was not run.
- The pre-window mine-fuse 7777 flip (above) belongs to whichever task ports the
  detonator.

## Live checks for the main agent

None required — no visual claim of this task needs a render, and no address stayed
unresolved long enough to need a debugger watch. (If a cheap one is wanted: with the
laser watch 0x4E equipped, fire at a wall ~5 u away — the scorch should appear at the
aim point and the beam should visibly stop at 4 u [L weapon_data 0x4E +0x1C = 4.0].)

## Usage

not visible to me.
