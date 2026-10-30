## Your task: `level-0a-walk-clip-flip-2`, finish the move-clip flip

Continue `origin/s/level-0a-walk-clip-flip` (merge `origin/master` first; read its report). Under NF_WALK_CLIP=1 walkreal_0700000a_live is 6/1/2: the pick-ref accumulator and the office-floor keyboard legs are green; the qworm ring (the alarm leg) is still red. Fix that leg (port or test, with cites), then do the flip: `walk_clip()` -> true, remove NF_WALK_CLIP and the dead push-walk branch. Run EVERY walkreal_* binary plus nf-game's collision/player tests; quote every total and list any walk whose route changed (leg + position delta).

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0a-walk-clip-flip-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox4 at 2026-10-30T12:44Z

---
## Report (done, 2026-10-30T14:09Z)


---
## Report from the worker (done, 2026-10-30T14:10Z)

# level-0a-walk-clip-flip-2 — the move-clip flip landed; the qworm ring reconciled (the hatch exit races the uncatchable press and wins by ~230 ticks); the 0a battery back to master's 7/0/2

## What was asked

Continue `s/level-0a-walk-clip-flip`: fix the last NF_WALK_CLIP=1 red leg
(`walk_office_floor_qworm_keyboard_hatch`, the alarm), then flip — `walk_clip()` → true, remove
`NF_WALK_CLIP` and the dead push-walk branch — and run every walkreal_* binary plus nf-game's
collision/player tests, quoting every total and listing any walk whose route changed.

## The qworm ring: what the trace showed, and the fix

`NF_ALARM_TRACE=1` on the failing flag run (the previous session's binary, 2026-10-30):

1. The east-side civilian spawner draws a DIFFERENT template under the clip walk: **g3, spot
   (9.8, 1.00, 21.3)**, spawned t13982, patrolling south to (9.9, 9.4) — where master's timing
   draws **g19, spot (15.3, 1.00, 5.1)** [L both traces this session]. The pick is the shared
   RNG's state at the draw, and the state differs because the walked lines differ (the install
   taps' spread draws, the spawner's per-tick abort draws — every attempt with a usable template
   consumes `Rand_Rand` [M spawners.rs:194]). Both draws are legitimate simulation; neither walk
   is "wrong".
2. g3's spot and patrol sit INSIDE the ring route's east L (the west-leg z 3.5 + the north-leg
   x 11.5): his unalerted sight (rng 12, half-fov π/2 [L the roster print]) catches Bond at
   t14589, d 6.9 — and once challenged the sight is combat (rng 17, half-fov 0 = all-around [M
   `FirstSightState` raises the range and zeroes the half-FOV, docs/systems/security-room-alarm-09.md]).
   The open corridor offers NO line-of-sight break within the ~295-tick challenge window, so —
   unlike g16 on the master-timing run, whose challenge aborted when the walk's corner broke LOS
   (seen false at t14471 → flee, no alarm [L]) — g3's challenge completes, he runs to the panel
   (9.5, −1.4) and presses (0x8a at t15149) [L].
3. The player cannot kill him: g3 carries props 0x53/0x54 [L the trace roster print] and a
   team-3 protected death is fail reason 3 → ch 0x60 → the mission fail [M
   `on_init_death`/`mission_fail_reason` in crates/nf-game/src/ally.rs]. The chase cannot reach
   him inside the press timer either (0x8a → the fail in 280 ticks [L t15149 → t15429]).
4. **The fix (the player's own answer): leave through the hatch.** The final
   `watch_and_clear` before the E is removed — hunting a protected presser from the hatch mouth
   burns the press timer — and Bond walks straight into the LOAD_LEVEL trigger box / E's the
   hatch. The first `level_exit` write wins and the press's later fail write is `is_none`-gated
   [M drone.rs:15145-15146 and the fail path's same gate], so the exit sticks. Measured: the
   walk completes at t15179 with **ch 0x63 FALSE** — the exit won by ~230 ticks; the press never
   even completed [L this session's run].

## The flip

- `walk_clip()` and the `NF_WALK_CLIP` env gate removed; `physics_tick` now runs
  `capsule_clip_scoped` unconditionally, and the floor-redo arm runs `capsule_clip` (the dead
  push-walk branches deleted; `capsule_push`/`capsule_push_scoped` remain for the ladder-climb
  path and the NPC/SWIM semantics [M player.rs:1022's CLIMB_RADIUS user]).
- **The first flip draft had a real bug, caught by the battery and fixed**: I hoisted the
  `move_v` computation to the tick top, where `pos − prev` is still (0,0,0) — the clip never
  fired and the 0a walks broke (4/3/2, the install walks failing at the ch-0x48 assert). The
  correct shape is the previous session's: capture `pre_move_pos` at the tick top, compute
  `move_v = pos − pre_move_pos` AT THE COLLISION BLOCK, after the walk's matrix·(vs,vf) add and
  the gravity integration. The report is honest that this session introduced AND fixed that.
- The corner test's ignore note updated (the flip landed; the exit-y settle gap remains).

## How it was verified (the totals quoted)

- `walkreal_0700000a_live` on the flipped default (--test-threads=1, slim ISO): **7 passed /
  0 failed / 2 ignored** — master's old default total, i.e. the flip preserves the level's
  behaviour, with the qworm walk 79 ticks faster than the push walk's 15258 (15179 measured [L]).
- **nf-game's whole crate** (the lib tests + 96 integration binaries — collision, player, world,
  drones, AI): **955 passed / 0 failed / 9 ignored** across 97 binaries — all green under the flip.
- **Every walkreal_\* LEVEL-walk binary** (24, --test-threads=1, slim ISO, 110-150 s caps):

| binary | total |
|---|---|
| walkreal_0700000a_live | 7 passed / 0 failed / 2 ignored |
| walkreal_07000001_live | 2 / 3 FAILED |
| walkreal_07000002_live | 1 / 0 ok |
| walkreal_07000003_live | TIMEOUT (>150 s) |
| walkreal_07000004_live | 7 / 0 ok |
| walkreal_07000005_live | 9 / 1 FAILED |
| walkreal_07000006_live | 3 / 0 ok |
| walkreal_07000007_live | 2 / 0 ok |
| walkreal_07000008_live | TIMEOUT (>110 s) |
| walkreal_07000009_live | 3 / 1 FAILED |
| walkreal_0700000b_live | 1 / 2 FAILED |
| walkreal_0700000c_live | 1 / 1 FAILED |
| walkreal_0700000d_live | 1 / 0 ok |
| walkreal_07000011_live | 0 / 1 FAILED |
| walkreal_07000011_route_live | 1 / 0 ok |
| walkreal_07000012_live | 0 / 1 FAILED |
| walkreal_07000013_live | 0 / 1 FAILED |
| walkreal_07000014_live | 2 / 4 FAILED |
| walkreal_07000015_live | 1 / 0 ok |
| walkreal_07000016_live | 3 / 5 FAILED / 1 ignored |
| walkreal_0700001b_live | 8 / 0 / 1 ignored ok |
| walkreal_07000041_live | 5 / 0 ok |
| walkreal_0700004a_live | 1 / 0 ok |
| walkreal_0700004a_satchel_live | 2 / 0 ok |

  Level-walk sum: **60 passed / 21 failed / 4 ignored**, two timeouts (03, 08).
- The fast suite runner (`tools/test/suite.py`) ran its full build (935 test binaries, 187 s)
  and 110 binaries of the run phase (**437 passed / 17 failed / 21 ignored**) before this
  session killed it for the deadline — the 2-core sandbox cannot fit the 935-binary run phase
  in the window. Its FAILED list is a superset sample of the walkreal reds above plus the
  interaction tests that ride the walks: `gt_los_live::the_open_front_spot_still_deploys_the_gun`,
  `mp_bots_live::bots_roam_and_hunt` (the bot closed to 2.7 and never fired), `fix_truck_live`
  (both ride tests), `kiko_button_real_walk_live`, `interactables_07000013_live::the_pickups_take_on_the_walk`,
  `interact_07000001_live` (armour + E-door), `guards13_wake_live`, `walk_07000014_live::the_facility_chain_runs_to_the_delta_sector_exit`,
  `level_end_live::the_walkin_leaves_a_closing_straddle_door_alone_on_07000016`,
  `probe_pylon_render::probe_render_pylon_only`. The remaining ~825 suite binaries did not run
  this session — that re-run is the first thing for the next session.

## The walkreal reds: what changed, mechanically (the next session's reconciliation list)

1. **The airborne hang (a mechanics regression, not a route nudge)**: under the clip Bond HANGS
   airborne where the push dropped him — 07000014's grapple release over the open gap wedges at
   (43.97, 2.67, 63.46) instead of landing on the trench floor −0.60 (4 tests, one shared
   assert), and 07000001's meet-plate walk hangs at (3.40, 7.00, 10.40) grounded=false over a
   floor 1.66 below. The suspect is the r-skin retraction pinning a FALLING move against a
   sloped/diagonal face (the per-tick gravity vs the radial retraction equilibrium); the fix
   needs the fall-contact class read against the game (the live checks below).
2. **The use-pick position deltas**: 07000012's console E (ch 0x49/0x59 never wrote — Bond's
   stood position at the console rests at the r-skin where the push rested closer), 07000011's
   spawn walk, 07000013's pickup walk — small walked-line shifts breaking tight picks.
3. **The guard chains**: 01's three named guards (the shoot loop's aim/positions shifted), 09's
   one walk, 0b/0c's legs, 16's five — the same two classes above, per the failure dumps.
4. The green set is meaningful: **0a (the task's level), 02, 04, 06, 07, 0d, 11_route, 15, 1b,
   41, 4a, 4a_satchel** — 12 of the 24 level walks unchanged, and nf-game's 955 unit/integration
   tests all green.

## What's left

1. **The walkreal reconciliation on the non-0a levels** (21 failed + 2 timeouts above): the
   airborne-hang mechanism first (07000014's grapple release, 01's meet plate), then the
   use-pick deltas and the guard chains — fix the port or re-derive the routes, with cites.
2. The full 935-binary suite re-run on this final tree.
3. The west-hall +0.0256 climb settle (the glide/corner exit-y gap) — unchanged, pre-existing.
4. The corner test stays ignored until that settle lands (note updated).
5. The spawner's RNG divergence between walk models is inherent (event-driven draws over
   different walked lines) — noted here so nobody chases it as a bug.

## Live checks for the main agent

1. (carried from level-0a-walk-clip-flip) **The glide wall's rest geometry**: fresh boot
   0700000a, press into the west hall's south wall at the z 16.29 line 5 s; record obj+0x30.z
   minus the wall's face plane (the corner report quotes ~0.011 inside [L]; the west-wall press
   reads face+0.45/+0.55 [L] — the per-contact key decides).
2. (carried) **The open-door leaf press**: fresh boot, walk dead-on into the open leaf of the
   (−20.7, 1.1, −5.4) door, hold 3 s. EXPECTED (r-skin general): the spine rests at leaf+r, no
   leak-through — decides whether the qworm jink-4 route bend is game-true.

## Usage

Not visible to me — this environment shows no token, credit, context-window or cost figures for
the session. No sub-agents were spawned this session, so there are no per-call footers to quote
either.
