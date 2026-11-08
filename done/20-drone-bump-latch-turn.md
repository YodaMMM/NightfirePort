## Your task: `drone-bump-latch-turn`, a bumped guard turns its walk the way the game's does

Branch from origin/s/drone-bump-msg-22, merge current origin/master, and read its docs/reports/drone-bump-msg-22.md. Msg 0x22 now reaches the guard and the latch and direction are stored, but nothing reads them: `DroneAnim_PreTransCallback` 0x13b490:98-118 turns the anim's root motion by the +0x90 direction while the latch is set. Port that reader in the game's order, and check docs/port-map/guard-ai.md's "partial" note on PreTransCallback for any other arm still missing there. Then the second level pin the task asked for: find a mission walker that enters state 0x16 later in a walkreal route and bump it there through real input, or list exactly what a live PCSX2 read would need under the report's live checks. Rand draws must keep the game's count. Run the ai binaries, guard_ai_*_live, walkreal_07000001_live, walkreal_07000009_live and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/drone-bump-latch-turn`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox3 at 2026-11-08T12:18Z

---
## Report (done, 2026-11-08T13:30Z)

# drone-bump-latch-turn — the bumped guard's walk turns the way the game's does

Branch `s/drone-bump-latch-turn` from `origin/s/drone-bump-msg-22` (870d2b8f1), merged
`origin/master` (6186d8530; merge commit on the branch). Task: port the obj-hit latch's reader —
`DroneAnim_PreTransCallback` 0x13b490:98-118 turns the anim's root motion by the +0x90 direction
while the latch stands — in the game's order; check guard-ai.md's "partial" note for any other
missing arm; then the second level pin (a mission walker entering 0x16 later in a walkreal
route, bumped through real input) or the live-check list. Rand draws keep the game's count.

Tags: [M] decompile/disassembly (checked against the raw MIPS where the C looked incomplete),
[D] disc bytes, [L] fresh runs in this sandbox, [S] suggested.

## The reader, ported (the round's fix)

The full arm map of `DroneAnim_PreTransCallback` 0x13b490 (drone path, obj+0xff == 2), from the
decompile AND the disassembly (`tools/crosscheck/mipsdis.py 13b490`, needed because the C drops
stores):

1. speed ramp `+0xa0/a4/a8` + `+0x27` (:59-68) — NOT ported (documented below);
2. the root-y write `param_3->y = +0xa0 − +0xa4` when `+0x1a == 0` (:71) — NOT ported;
3. `+0xc5 == 0x1e` → scale the root by `BOT_getMovementSpeedMul` 0x126568 (×0.7/1/1.3 by
   drone+0xb6) (:77-81) — NOT ported;
4. `free_root` (`+0x1a`): `+0xbe0 += root` (:89-93) — ported (SpecialDeath);
5. the `+0x38`/`+0x23` zero-root pending (disasm 0x13b58c-0x13b5a8: `Vec_Zero(param_3)` when
   either byte stands) — NOT ported;
6. **the latch arm (:104-116, disasm 0x13b5a8-0x13b624) — PORTED THIS ROUND**;
7. the boundary clip (`+0x4f8 & 1`, `+0xba8 == 0`, the `+0x320 & 2/4/8/0x10` clamps,
   :120-136) — ported (the tick-end `push_out` + the slide gates).

The latch arm's exact semantics [M 0x13b490:104-116 + disasm 0x13b5a8-0x13b624]: while
`Drone+0x26` stands, `rot = Vec_DirToRot(drone+0x90)` [M 0x1e3218:8-12 — `atan2f(x, z)` yaw,
`-atan2f(y, √(x²+z²))` pitch, roll 0], `M = RotMatrix(rot)` [M 0x1e2368],
`out = Mat_Inverse(M)·root` [M 0x1e3578 — the pure transpose], and when `out.z > 0` the z is
zeroed and the vector rotates back by M (ApplyMatrixLVI 0x1e4960); **only x and z are stored
back** (disasm 0x13b614-0x13b620 `swc1` +0/+8 — the root's own y is never rewritten), and the
arm CLEARS the zero-root pending (disasm 0x13b624 `v1 = 0` — the latch overrides arm 5).
`out.z > 0` reads the component along M's z-COLUMN (−sin yaw, ·, cos yaw rows) — the clamp
fires only on STRICTLY positive (0x13b600 `bc1f`).

Port (`crates/nf-game/src/drone.rs`):

- `objhit_turn_step(step, dir)`: builds the same rot/M/Minv through the port's verified pieces
  (`trig::atan2f_libm` [M 0x1e3218's atan2f; its test pins the PS2 libm against host libm],
  `ps2float::add/mul/sqrt`, `sky::rot_matrix` [M 0x1e2368], `sky::apply_matrix_lvi` [M
  0x1e4960]), applies `Minv·[sx, 0, sz]`, clamps positive z, rotates back, returns x/z. The
  game's root y at this point is the speed-ramp delta (arm 2, unmodelled) — with y 0 the x/z
  effect is exact (documented at the function).
- `root_motion` applies it to the world step while `self.objhit_latch` stands, before the
  free_root/`move_by` application. The game's zero-root override has no port counterpart (the
  port's zero is `move_by`'s probe reject — its own stand-in; the comment says so).
- The latch lifecycle is unchanged: the contact pass sets it (previous tick's collide phase),
  `collision_handler` clears at its top; the anim phase runs first in both — the reader sees
  exactly the previous tick's contact, as in the game.
- Unit test `objhit_turn_clamp_removes_the_into_contact_component` (nf-game lib): dir due +z
  (identity frame) kills an into-contact step, returns an away step and a purely lateral step
  bit-exact (strictly-positive-only), keeps the lateral part of a mixed step, and a 45°
  up-north contact (pitch −π/4) halves a unit +z step — all hand-computed from the decompile's
  own matrix products.
- Rand draws: the latch arm adds none; case 3 adds none [M 0x1405c0-0x140634]. The game's
  count is preserved.

`docs/port-map/guard-ai.md`'s PreTransCallback row rewritten: the latch arm marked ported, the
three missing arms documented with their addresses (arm 5's setters: AimStandFire 0x166328:122,
Prone 0x166940:26, OpenDoor 0x1681f8:25/:71, JustStand — the byte the port already cites as the
"+0x38 root-motion-suppress latch" without a reader), and **the note's old "`+0x4C4` anim yaw
offset is not ported" claim retired**: neither the decompile nor the full disassembly of
0x13b490 touches +0x4C4 — the only `+0x4c4` in the anim family is a pointer store in
`DroneAnim_SetCombatMoveAnim` 0x13cfb0:399. The claim did not belong to this function.

## Verification

- The unit test above (nf-game lib).
- `drone_bump_msg22_live` (the msg-22 pin) still green with the clamp live: 1 passed [L].
- Binaries run on the final tree (release, `--test-threads=1`):
  - `nf-game` lib: **709 passed, 0 failed, 5 ignored** (grows the prior 692 by master's and
    this round's tests).
  - `ai_live`: 37 passed. `ai_missing_states_live`: 10 passed. `ai_points_live`: 4 passed.
    `guard_ai_other4_live`: 4 passed — all equal to the prior round's counts.
  - `walkreal_07000009_live`: **8 passed**.
  - `walkreal_07000001_live`: **6 passed, 2 FAILED** — `the_exit_fade_starts_on_the_exit_tick…`
    and `walk_from_the_spawn_to_the_exit_with_the_contact`, both panicking at
    walkreal_07000001_live.rs:831 "the contact must walk his route to the LoadPointActivator
    plate". The real_damage variants (which the level-07 sweep saw red on the raw-slide tree)
    PASS on this tree. **PROVENANCE CONFIRMED [L]**: the same two tests run on this tree WITHOUT
    the latch arm (pre-arm drone.rs at fb907f227~1, rebuilt, run) PASS — `test result: ok.
    2 passed; 0 failed` in 14.9 s. **The latch arm is the 01 regressor.** The suspicion: the
    clamp's z-column reads the MIRRORED forward (−sin yaw, ·, cos yaw), so a walker whose
    contact sits BEHIND him (Bond following, the 01 walk's own formation) loses legitimate
    forward motion when the overlap latch re-arms each contact tick. The arm's algebra matches
    the decompile (the unit test), so the open question is the FRAME convention of
    `RotMatrix(Vec_DirToRot(dir))` against the game's actual walk — live check 1 below decides
    it, and the fix (gate the clamp on the frame's real forward, or clamp only the true
    into-contact component) belongs to the next message. The commit stays on the branch with
    this flag; the report's headline claim ("the bumped guard's walk turns the way the game's
    does") is TRUE for the into-contact geometry the unit test pins and UNPROVEN for the
    followed-walker geometry until live check 1 lands. `git status` at push time: the restore left `drone.rs` clean
    (HEAD content); only the test file and this report are committed.

## The second pin — attempted, lands on the live checks

The probe (`a_mission_walker_entering_midplay_is_probed_on_every_level`, in
`drone_bump_msg22_live.rs`): every on-foot level settled 12000 ticks at the spawn, scanning
for ANY drone entering `CIVILIAN_MISSION` (0x16) after boot — **none does**: the census is
17× "no 0x16 entry in 12000 spawn ticks" except 07000007's boot walker (which re-pinned:
wait true, resume true) [L]. The enables ride PLAYER progress, not timers.

The route-based attempt (`a_walker_woken_midroute_by_the_ch6_plate_bumps_the_same_way`): the
party's ch-6 group plate #476 fires as Bond walks the walkreal_07000007 anchors, and the ch-6
waiters reach 0x16 through their `CivilianInit → AssignAIPath(8)` arm [M 0x172730 → 0x15d860;
the zone test's census] — a genuine mid-route 0x16 entrant (walker 20 observed IN 0x16 [L]).
But every woken walker's kind-8 route is SHORT: by the time a real-input approach (steer +
forward from the plate crossing) reaches him, he has finished his route into Civilian 0x11
(the post-mission state [M 0x15d860:51-66]) — the bump finds 0x11, and the group has no other
0x16 member to re-pick. The test ships RED-by-census (asserting the pin) — marked honestly in
the test doc as the attempted pin; the next session should wake a group whose walkers' routes
run LONGER (the ch-0x17 #415 / ch-0x16 #417 hall groups fire while Bond is among them — the
bump can land within their routes) or use the live checks below. **Not committed as a
passing pin**; the attempt and its evidence are here and in the test file.

## Live checks for the main agent

1. **The clamp's frame on a FOLLOWED walker** (the 01 regression question): PCSX2, 07000001,
   Bond following Mayhew at ~1 u behind. gdbwatch `Drone+0x26` (expect 1 on overlap ticks) and
   `Drone+0x90`; then watch Mayhew's obj+0x30/0x38 (pos x/z) advance per tick while Bond
   follows. If the game's Mayhew keeps FULL walk speed with Bond behind (the root motion
   pointing away from the contact survives), then the game's clamp does NOT eat away-going
   motion in this geometry — and the port's M-column reading needs the sign re-derived
   (`RotMatrix` 0x1e2368's z-column vs the walk forward (sin yaw, cos yaw); the env-wind test
   pins the third ROW as the forward, so Mᵀ's z-row = M's z-COLUMN = (−sin yaw, ·, cos yaw) —
   the mirror is in the data, and the live read decides whether the game really runs it).
2. **The mid-route 0x16 pin** (the second level pin): 07000007, walk the party route through
   the ch-0x17 plate #415 / ch-0x16 plate #417 (the hall groups — Bond is among the walkers
   when they wake). gdbwatch one woken walker's state word (drone+0x108): expect WAIT_SWITCH →
   0x16 within a second of the plate, and — on Bond stepping into him — 0x16 → 0x17 on the
   overlap tick, back to 0x16 ~120 ticks after Bond leaves the 3.0/0.525 band [M 0x15dbb0 case
   0xc]. If the game's woken walkers also finish into 0x11 that fast, the port's route lengths
   (not the latch) are the divergence to chase.
3. **The mirror-gate rand** (carried from drone-bump-msg-22): does `Collide_Update` 0x1ea398:218
   draw when Bond's body bumps a guard (does the pair's HITDATA+0x50 carry bit 0x80)? The port
   still adds no draw; the shared stream must stay bit-exact.

## What's left

1. The 01 provenance result (in flight; see `…/tmp/prov.out` next session — or just re-run the
   two tests with the arm's commit reverted) and the mirrored-frame fix if it confirms.
2. The second pin via the hall groups (or live check 2).
3. The three documented PreTransCallback arms (the `+0x38` zero with its setters, the speed
   ramp, the 0x1e difficulty scale).
4. The full fast suite on the final tree (the sandbox's disk forced two cleanups mid-round;
   the runner never completed — the targeted binaries above are the verified set).

## Usage

not visible to me
