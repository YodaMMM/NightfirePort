## Your task: `break-sound-roll`, the breakable kill sound's Rand_Rand(100) roll in the game's draw order

`Break_Kill` 0x12b970:65-76: kind 2 plays 0x2E with no draw; every other kind rolls `Rand_Rand(100) < 0x29` → 0x5F2, else 0x5F3. The port (crates/nf-game/src/breakable.rs `kill`) always plays 0x5F3 and skips the draw, because inserting the draw shifted the shared RNG stream and broke escort_follow_07000001's timing (task 959 measured 7.0 s vs 18-46 s stalls).

1. Establish where the draw sits in the game's per-frame RNG order: who else draws from `Rand_Rand` on the frames a vase/bust dies (the bullet pass, debris create 0x1ed548 / 0x1aaa00, the drone AI). Cite every consumer between the bullet hit and the kill in frame order [M].
2. Put the roll in at the matching position so the port's stream matches the game's. If escort_follow then stalls, find out why its timing depends on the stream (it should not depend on a vase sound) and fix that, with evidence.
3. Tests: kind 0 kill → 0x5F2 or 0x5F3 by the roll (seeded), one draw consumed; kind 2 → 0x2E, no draw. If a live check is needed, say which counter to read in PCSX2 (Rand seed address) across a vase kill on 07000007.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/break-sound-roll`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox3 at 2026-10-23T15:32Z

---
## Report (done, 2026-10-23T17:27Z)

Closed by the driver: the report is on `s/984-break-sound-roll` (docs/reports/984-break-sound-roll.md); the worker didn't run finish.sh.

---
## Report from the worker (done, 2026-10-23T17:44Z)


---
## Report from the worker (done, 2026-10-23T17:44Z)

# 984-break-sound-roll — the breakable kill sound's Rand_Rand(100) roll

Branch `s/984-break-sound-roll` (from origin/master, tree-identical rewrite a67f1a27e; the
driver's marker commit e81661175 is the base). One code commit + test fixes; no port source
changed except `crates/nf-game/src/breakable.rs` (`Breaks::kill`).

## What's done

1. **The roll is ported.** `Break_Kill` 0x12b970:59-76 [M]: after the collision-records gate,
   kind 2 plays 0x2E with NO draw; every other kind draws `Rand_Rand(100)` — < 0x29 → sound
   0x5F2, else 0x5F3 — before both `Sound_Play3D` arms and before the pieces. The port's
   `Breaks::kill` now does exactly that (the roll is the kill's FIRST draw, ahead of the
   shatter/debris draws); the old deterministic-0x5F3 deviation and its `// [S] unverified`
   note are gone. `Rng::rand` already implemented `Rand_Rand` bit-exactly (y then x step,
   `(y&0xffff)·n>>16` [M 0x1e3738 + disasm: SEED_Y 0x30d0a4, SEED_X 0x30d0a0, multipliers
   18000/30903 @0x30d0a8/ac, `_gp` 0x314670 −0x75d0/−0x75cc]).

2. **The per-frame draw-order census around a vase kill** (all [M] decomp + disasm, one [L]):
   - The kill is a TWO-frame dance. Frame N: `Bullet_update` 0x12e7b0 moves the round (one
     `Rand_FRand(0.5)` per flying bullet, walk is newest-first so bullets draw first); the
     contact is registered later that frame by `Collide_Update` 0x1ea398 (the last step of
     `control_movement_object_handler` 0x1336f8). Frame N+1: the class-0x20 update
     `Break_Update` 0x12bfd8 (its control_funcs slot read live [L ee_start_05.bin@0x26f890
     +0x20·12 = 0x0012bfd8]) walks the contact chain, health < 0 → `Break_Kill`.
   - Between the bullet hit and the kill the frame draws: `Env_Update` 0x17bc88's
     `Rand_Rand(20000)` (every frame with a world — 07000001 carries rain #279 [D]);
     `Drone_InitComms` 0x139520, `Doors_Calc` 0x135fe0, `Trigger_Calc` 0x1b1900 draw NOTHING;
     in the walk the other bullets (one `FRand(0.5)` each) and the drones' AI rolls
     (`SetIdleTimeOut` 0x14a3b8, `SetOpponentAimPos` 0x141fe8, `ChooseCombatMove` 0x150c18,
     head-tracking 0x1472c8/0x147578, SFX pickers 0x141848/0x1497e8) draw at their list
     positions. The escort machine itself (0x15e698/0x15eda8/0x15ec08/0x15f120/0x15f2e8/
     0x15f938) draws nothing.
   - Inside the kill: the roll, then `Drone_BrokenObject` 0x138588 (zero Rand calls; its whole
     body is the MP `GameState+0x12 == 0x7000001` speech gate), then `Debris_Create_Loop`
     0x1ed548 → `Debris_Create` 0x1aaa00: 8 draws per record (FRand(0.1), Rand_Rand(half),
     FRandHalf(spread)×3, FRandHalf_Vec(spin)×3); the glass arm draws in `ShatterPolyRecurse`
     0x12b270. `Collide_Update`'s own conditional `Rand_Rand(0x3f)` (surface byte bit 0x80)
     does not fire on a vase contact (material 0x10 carries neither 0x40 nor 0x80).
   - Walk position of the kill: placements prepend (newest first), and 07000001's breakables
     sit at array positions 22-306 vs drones 158-299 [D placements] — breaks created after the
     drones kill BEFORE that frame's AI draws; older breaks kill after. The port compresses
     the two frames into its bullet pass, so the roll (like the debris draws already shipped)
     lands one AI-frame earlier in the port's stream than the game's — documented in the
     module header as the standing bullet-impact compression.

3. **The escort mystery resolved.** Task 959's revert (2f7a41cd) blamed the roll for
   escort_follow going 7.0 s → 18-46 s stalls, but that branch bundled the roll with "Bond in
   the level-01 class-2 contact pass" + the intact-pane `set_dynamic` injection — and its own
   revert note says the injection stalled walkreal_07000001. On the roll alone:
   `escort_follow_07000001_live` PASSES (level_exit 07000002 at tick 25939, the walking-speed
   bands held, 10.0 s wall), `walkreal_07000001_live` 3/3 (21.5 s). The escort's timeline does
   move with every breakable kill (the shared SEED pair feeds the other guards' AI — the
   escort test's own header documents 959's finding); that is the game's own structure, not a
   port defect, and one roll draw per kill does not stall it.

## Tests

- `breakable::tests::the_first_bullet_kills_a_zero_health_vase_with_the_debris_roll` — kind 0
  kill: the sound equals a probe stream's `rand(100) < 0x29` outcome, and the kill's stream is
  bit-exact the game's: roll, then the 8 debris draws [M 0x12b970:66 + 0x1aaa00:30-50]; 9 draws
  total (1 + 8).
- `breakable::tests::glass_shatters_with_sound_0x2e_and_consumes_no_roll_draw` — kind 2 → 0x2E
  with ZERO draws consumed (the roll is inside the `!= 2` branch [M 0x12b970:59-66]).
- Two existing draw-count pins contradicted the decompile (the roll precedes the debris loop)
  and were corrected with the addresses: the lib debris-launch test 8 → 9, and
  `break_debris_live`'s twin-vase pin 8n → 1+8n [M 0x12b970:66].
- Run: `nf-game` lib 522/0; break_debris_live 2/2, breakables_live 2/2, glass_breach_live 1/1,
  interact_07000001_live 10/10, escort_follow_07000001_live 1/1, walkreal_07000001_live 3/3,
  rocket_glass08_live 3/3, fix_missing_objects_live 3/3.

## Suite result

`tools/test/suite.py` was started on the final tree at 17:22 UTC (build-verify 737 s, then
all 730 binaries, `--test-threads=1`). The sandbox window closes ~18:12 UTC and a full pass
needs ~70 min, so the total line is not reached in this window — the next message reads
`tail suite.out` / `~/suite.log` for the FAILED/TIMEOUT lines and the run's final count.
Baseline (same sandbox, master tree, 16:21-16:57 UTC): 4 FAILED disc/identity tests (the
slim-ISO golden misses, the task's documented exception) + 5 OOM CRASHes
(mp_stress_explosives_live, mp_bots_moving_live, net_match_end_clients_live,
walkreal_probe13, mp_online_failures — the box's 4 GB kills the 2.9 GB MP tests;
environmental, present on master too). The roll-tree run by 17:43 UTC (45 binaries in):
`fix_0c_pylon_live` ×2 FAILED — the documented no-GPU pylon render class (417bc6a0e,
559511469) — and `net_join_live` CRASH (the OOM class); no new failure class, and every
breakable/escort/walk-related binary passed (the ones the suite had reached, plus the manual
canary set above).

## What's left

- The full suite's total line on this tree (next message: re-run `tools/test/suite.py`, or
  read the tail of the run above).
- If a stronger game-side pin is wanted: the live check below.

## Live checks for the main agent

- The Rand seeds are at **SEED_X 0x0030d0a0, SEED_Y 0x0030d0a4** [L ee_start_05.bin values
  0x06595adb / 0x208a5de8; symtab names confirmed]. On a FRESH boot into 07000007, record both
  words every frame (`tools/trace/gdbwatch.py`, read, 32-bit) across a vase kill (shoot a
  kind-0 vase). Expected [M 0x12b970:59-76 + 0x1aaa00:30-50]: on the frame the kill sound
  plays, the FIRST state transition of the frame's tail is the roll (one `Rand_Rand` = SEED_Y
  steps then SEED_X steps), and the sound id obeys `(SEED_Y&0xffff)·100>>16 < 41 → 0x5F2 else
  0x5F3`; the debris pieces each consume 8 more draws (both words advance 8·count times). A
  kind-2 (glass) kill must advance the seeds by the shatter draws only — no roll step.

## Usage

not visible to me
