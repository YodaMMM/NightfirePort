## Your task: `level-objects-uncited`, level-object functions the port never names

Never mentioned in `crates/*/src`: `Copter_FireMissile` 0x17d9f0, `SpaceLaser_Register` 0x1aa9e8, `Sunreflection_Update` 0x18db40, `Bubble_Update` 0x1aca48, `camera_fixed` 0x1ba4e8, `LS_LoadBonus` 0x1ce488, and the sub/creature family `Sub_Laser` 0x1acda0, `Sub_DeployTorpedoes` 0x1acf68, `Sub_Update` 0x1ad200, `Sub_ApplyDammage` 0x1ad570, `Sub_Collide` 0x1ad868, `Sub_Delete` 0x1ad870, `MiniSub_FireTorpedo` 0x181888, `MiniSubMove_Forward` 0x1819d8, `MiniSubState_Attack` 0x181b00, `MiniSub_Update` 0x181f40, `MiniSub_Collide` 0x182128, `MiniSub_Delete` 0x182268, `Creature_Delete` 0x135230, `Creature_Update` 0x135268, `Creature_Hurt` 0x1352d0, `Creature_Move` 0x135340.

For each: name its creator/caller and the placement kind or script event that reaches it, and which levels (on-foot vs driving — the driving missions are not ported yet) [D + M]. On-foot reach: port the simulation [M] with a test (draw-side parts — the sun reflection sprite, bubbles' sprites, missile meshes — are not yours: list them in the report as draw hand-offs with the hash, matrix and timing). Driving-only or unreachable: cite the proof in a comment next to the nearest related port code.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-objects-uncited`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox1 at 2026-10-26T13:26Z

---
## Report (done, 2026-10-26T14:41Z)


---
## Report from the worker (done, 2026-10-26T14:42Z)

# level-objects-uncited — report

Branch `s/level-objects-uncited` (from origin/master @ 7e35bd77). Task: the 22 level-object
functions never cited in `crates/*/src` — for each, name the creator/caller and the reaching
placement kind / script event and the levels; port the on-foot-reachable simulation with a test;
driving-only or unreachable ones get a proof citation next to the nearest port code.

Tags: [M] decompile/disassembly (`work/decomp/ACTION.ELF`, addr:line), [D] disc data, [L] live
PCSX2, [S] suggested. Research was split over three parallel subagent passes; every load-bearing
claim below was re-checked against the named decompile files.

## The disc census (new test: `nf-formats/tests/level_objects_census.rs`)

Walks every 07* archive (driving archives included) through the port's placement parser and pins
per-level counts for the task's kinds — the [D] reachability evidence:

- kind 0x2F (`Copter_Create` 0x17d340 [M 0x1d04f0 case 0x2f]): **07000005 ×5, 07000008 ×3,
  0700000B ×3, 07000012 ×2, 07000046 ×1**. `Copter_Create`:42-45 turns EVERY 0x2F placement into
  a path node; only param-0==0 records create the copter object — the docs' "2 copters on 05 /
  1 on 08" [D docs/levels/07000005.md:137, 07000008.md:45] are the param-0==0 subset.
- kind 0x38 (`Create_SpaceLaser` 0x1aa8b8 [M 0x1d04f0 case 0x38]): **0700001B ×1** (PL#108).
- kinds **0x36 (Sub) / 0x37 (MiniSub) / 0xE0 (Creature): zero rows anywhere** — asserted by
  `creature_kind_0xe0_never_ships` and the main pin.

## Per-function findings and what was done

**Ported (on-foot reachable, with tests):**

1. `Copter_FireMissile` 0x17d9f0 → `nf-game/src/copter.rs` `MissilePod` + `fire_missile`.
   Caller: `Copter_Update` 0x17dbb8:188-198 (state 2, every tick while `attackmode_159 & 1 == 0`;
   state 2 is entered by the not-yet-ported state-1 spot check — documented in the module).
   Creator: kind 0x2F placements (above). Levels: 07000005/08 (on-foot, ported) + 0B/12/46.
   Order pinned by tests: the `Rand_Rand(0x86e6)` pod pick is unconditional and precedes the
   cadence gate (`GameState+0x52 % (u32)(60·DiffMod)`, 1 missile per 60·DiffMod ticks [M :47-64,
   disasm 0x17da0c/0x17da30-0x17da9c]); even → slot 8 (`rec+0x50`), odd → slot 7 (`rec+0x4C`);
   homing (`Rand_Rand((u32)(8.0/DiffMod))==0`, normalize(player − pod+0xC0)) on GameState
   0x07000008 only [M :65-76]; weapon record 0x6d/109, 0x62/98 on 07000008 [M disasm 0x17db28/
   0x17db30]; muzzle = pod+0x80 + dir·(glist+0x38·1.5) [M :79-84]; `Bullet_init(-1, copter, pod,
   weapon_data+id·0x10c, pos, dir, 0xffff)` [M :95-97]. 4 tests in copter.rs.
2. `SpaceLaser_Update` 0x1aa3f8 + `SpaceLaser_Register` 0x1aa9e8 → `nf-game/src/space_laser.rs`
   `tick`/`register`/`LaserOut`. Caller: `control_funcs` row class 0x50 (vaddr 0x26fc50); Register
   is the tracking scripts' stream callback (`Script_Load` ..., 0x1aa9e8, ext [M 0x1aa3f8:100-108])
   and stores the REGISTERED SCRIPT OBJECT's position quad into ext+0x00 — the fire origin [M
   0x1aa9e8:12-20; corrects space-laser-update.md's "copies the laser's position"]. Creator: kind
   0x38, 0700001B PL#108 only. On-foot yes, but **inert on retail data**: the arm gate is param 7
   → channel 19 and no disc script ever writes ch 19 [D docs/levels/0700001b.md:71,73]; the tests
   pin the inert path (no cooldown decrement, no draws) plus the synthetic armed path (tracking
   script load mode 0 + register → whine → state 2 bullet record 107 [M :172-175, 0x2c6154 =
   weapon_data 0x2bf150 + 107·0x10c] + beam 0x20005cd alpha `Rand_Rand(0x80)+0x7f` [M :183-185] +
   emitter + the 10 s re-arm) and the kill gate (ch 14 → state 4 [M :86-90]). The script lifecycle
   is abstracted behind a `script_playing` input — the same pending script-player integration
   `space_missile.rs` documents; its pinned-decode approach applies. Sound ids 0x5de/0x46a recorded
   in the module doc (sound-coverage.md lists class 0x50 MISSING).
   **Test-data fix (decompile beats test):** the existing param-map test labelled param 5 "the arm
   channel" with synthetic (5,19) — the decompile disagrees (arm gate = ext+0x44 = param 7 [M
   0x1aa3f8:75-77]; ext+0x40 = param 5 is the state-3 re-raise write channel [M :200-204]); the
   test now uses PL#108's real values (99/14/19) and the fix is recorded here.

**Unreachable / driving-only — proof citations (in drone.rs `attach_level_id` + this report):**

3. `Sunreflection_Update` 0x18db40 — `control_funcs` row 0x19/25 @0x26f9bc {update, 0, 0}. NO
   creator exists: zero `jal` targets, no data refs outside the row+symtab; the only class-0x15
   writer is AnimDebugCreate 0x1bcfa8 (a different row) — unreached on retail, nothing to port
   (behavior: viewer-pos + ext-quad offset, ×1.1 swell to 100 then ×0.909 shrink to self-delete).
4. `Bubble_Update` 0x1aca48 — row class 6 @0x26f8d8 {update, 0, 0}. No `Bubble_Create` symbol
   exists at all; zero jal/data refs. Ripples_Create 0x1acc90 is class 0x34 and spawns rings, not
   bubbles. Unreached (behavior: rise with damped x/z, pop at cel+0x54 surface plane, u16 life).
5. `camera_fixed` 0x1ba4e8 — dead: mode 4 of `Player_PositionCamera` 0x18f5d8:162-166; the only
   mode writer is `Player_SetCamMode` 0x195928 and the full call-site sweep (21 calls / 20 caller
   functions, correcting the audit's "19 sites") uses only {0,1,2,0xb,0xc,0xd,0xe,0xf} — modes
   4/5/6/0xa have no writer [M].
6. `LS_LoadBonus` 0x1ce488 — NOT dead: it is the save's 'BNUS' block loader. Dispatch: the
   runtime-built `SaveIFFBlocks` 0x2c74d8 (6 × {flag, fourcc, make, load}; 'BNUS' = flag 0x20 →
   make 0x1ce3b0 / load 0x1ce488, installed by __static_init 0x1cdd50:68-69); `LS_Load` →
   `LS_GetLoadStuff` 0x1cd2b0 walks it and calls load at entry+12. Reached from
   `Menu_UpdateMessageBox` 0x1f7de8 (the sole LS_Load/Save caller) with masks 0xff / 0x2d / 0x25
   — every load-game / MP-profile flow. Effect: `Menu_SetBonus(mask, slot, 0)` over the u64
   bonus[21] array 0x2e1b88 + the reward-counter upgrade re-derive [M 0x1ff898:69-107]. Port
   note: a save round-trip needs a 'BNUS' equivalent; no code change made (out of scope).
7. Sub family (`Sub_Create` 0x1ad6c0 class 74 / `Sub_Update` 0x1ad200 / `Sub_Laser` 0x1acda0 /
   `Sub_DeployTorpedoes` 0x1acf68 / `Sub_ApplyDammage` 0x1ad570 ← `Explode_Propagate` 0x17c518
   when hit-class == 'J' / `Sub_Collide` 0x1ad868 / `Sub_Delete` 0x1ad870 — both empty) and
   MiniSub family (`MiniSub_Create` 0x182130 class 77 / `MiniSub_Update` 0x181f40 — its own
   AINetwork chase; the "MiniSub calls Sub_Update" reading is FALSE, Sub_Update has zero call
   sites / `MiniSubState_Attack` 0x181b00 / `MiniSub_FireTorpedo` 0x181888 — weapon 0x2C2390,
   ±0.25 jitter, 50% homing via `Rand_Rand(0x5B7F)&1` / `MiniSubMove_Forward` 0x1819d8 /
   `MiniSub_Collide` 0x182128 / `MiniSub_Delete` 0x182268 — both empty): **driving-mission-only
   on retail** — the disc census shows zero kind-0x36/0x37 placements on every 07* archive,
   driving ones included [D], and the creators' only caller is the parsemap dispatch [M
   0x1d04f0:213-223]. No water/swim gating exists in either family (Sub's ram check is hull
   proximity: local Y ∈ (−0.9, 0.25), distSq < 64 → Player_Hurt(1.0) [M 0x1ad200:99-109]).
8. `Creature_Delete` 0x135230 / `Creature_Update` 0x135268 / `Creature_Hurt` 0x1352d0 /
   `Creature_Move` 0x135340 — dead on retail data, proof strengthened two ways: (a) zero kind-0xE0
   placements in every 07* archive [D census test]; (b) the exhaustive obj+0xFF class-writer sweep
   finds exactly one writer of class 0x23 — `Creature_Create` 0x134ab8:63 — whose only caller is
   the kind-0xE0 arm [M 0x1d04f0:351-357]; `control_funcs` row 35 @0x26fa34 = {0x135268, NULL
   @0x26fa38, 0x135230} (there is no Creature_Collide). Creature_Update ticks only when its cel
   is visible (cel+0xA0), hurt = class-3 collider → `Player_Hurt(2.0)` gated by param 4 [M
   0x1352d0:9-24], move = a 6-waypoint spline loop rebuilt from `Rand_FRand` draws with the water
   kind's spawn gate on cel kind 0xC023 [M 0x134ab8:99-109].

## What was run

- `NF_ISO=... cargo test --offline --release -j2 -p nf-game -- --test-threads=1` → **905 passed,
  0 failed** (91 suites; includes the new copter/space_laser tests and the NF_ISO live/golden sets).
- `NF_ISO=... cargo test --offline --release -j2 -p nf-formats -- --test-threads=1` → all green
  except the **3 known slim-ISO `disc_golden` failures** (bigf_header_fields_behave_as_documented,
  bigf_tables_match_the_golden_manifest, directory_tree_matches_the_golden_manifest — pre-existing,
  the same set the previous session recorded).
- The fast runner (`tools/test/suite.py`) could not be used: this sandbox kills background jobs
  between tool calls (three nohup attempts died silently here and in the previous session's
  WORKLOG); the foreground crate runs above are the substitute. The final tree == the pushed tree.

## Draw hand-offs (not mine, listed per the task)

- Space laser beam: class-0x10 object, mesh hash 0x20005cd, `PositionBeam(origin → target)`,
  alpha byte = `Rand_Rand(0x80)+0x7f`, `Emitter_Create(0,0,0xc000010, target, 0x2d6210, …)` at
  the impact — emitted as `LaserOut::Beam/ImpactEmitter` with the origin/target the sim holds.
- Copter missile: a class-5 bullet on weapon records 109/98 — flight/mesh is the port's bullet
  machinery (`Bullet_init(-1, copter, pod, weapon_data+id·0x10c, pos, dir, 0xffff)`).
- Sunreflection (if ever reached): obj+0x30 = viewer+0xC0 + the ext quad; ×1.1/×0.909 scale pulse
  on obj+0xE8; draw flag obj+0xFA bit 0x20.

## Live checks for the main agent

1. 07000005 copter: force state 2 (poke obj+0xF4 = 2 and `attackmode_159` = 0) and watch the
   missile bullet spawn — settle the 0x17df34 delay-slot caveat: is `obj+0xF6` re-zeroed every
   state-2 tick (making the 5·FR·DiffMod attack timer dead) or only on the accum>100 path
   (Ghidra's reading)?
2. 0700001b laser: fieldprobe the arm channel 19 across a full Drake fight — confirm no writer
   ever sets it (the inert-laser pin rests on the [D] doc claim).

## Usage

not visible to me — this sandbox exposes no token/context/cost readout (no usage figures are
printed by the harness; nothing to copy).
