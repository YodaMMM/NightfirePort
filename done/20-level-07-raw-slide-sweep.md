## Your task: `level-07-raw-slide-sweep`, the doorway raw-slide fix keeps every level's walkers on the game's paths

Branch from origin/s/level-07-actor-stuck-doorway-2, merge current origin/master, and read its docs/reports/level-07-actor-stuck-doorway-2.md. The fix removed the probe from undrawn walkers, matching the game's rule [M 0x139268:70-93], and the report warns that a level whose walker relied on the probe to stop at render geometry with no AI wall would now walk differently. Run every walkreal_*_live and npc_doors_live one level per tool call, and record each level's result in the report. For any walker that now leaves its path, find whether the port is missing an AI wall the game has there [D the level's AI wall set] and fix that, never bringing the probe back. Run fix_guard_misc_live, the route and door binaries and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-07-raw-slide-sweep`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox3 at 2026-11-08T10:48Z

---
## Report (done, 2026-11-08T12:16Z)


# level-07-raw-slide-sweep

Branch `s/level-07-raw-slide-sweep` from `origin/s/level-07-actor-stuck-doorway-2`, merged
`origin/master` (7876b7feb). Task: sweep every level's walkers after the doorway raw-slide fix
(the probe removed from undrawn walkers [M 0x139268:70-93]) — find any walker that now leaves
its path, decide whether the port is missing an AI wall the game has there, and fix it without
bringing the probe back. Also `npc_doors_live`, `fix_guard_misc_live`, the route and door
binaries, nf-game, and the fast runner.

Tags: [M] decompile/disassembly, [D] disc bytes, [L] fresh runs in this sandbox, [S] suggested.

## Headline: the sweep FOUND the warned regression — three confirmations

The doorway-2 report warned that a walker relying on the removed probe would now walk
differently. It does, and the sweep caught it with provenance:

1. **07000001 `walk_from_the_spawn_to_the_exit_with_the_contact_real_damage` (×3 variants)
   Bond dies** [L this sandbox]: the hall ammo-resupply fight (drone #11) chews 82.0 in one
   exchange (100.0 → 88.5 at 4393 … → 6.5 at tick 7305) and Bond dies at tick 7498 at
   (6.24, 6.44, 7.71). The plain (top-up) walks still pass.
   - On the BASE branch (79d9a0218, the raw-slide fix, pre-merge) the same test dies at the
     SAME tick 7498 with the same health path [L, the base worktree binary].
   - On `origin/master` (7876b7feb) the same test PASSES (7.65 s) [L, the master worktree
     binary].
   - ⇒ **The death is a regression of the doorway-2 lineage** (its two arms: the raw undrawn
     slide in `drone.rs update_anim`, and the `hit_door` skip in `Route::follow`
     [M 0x15a2c8:67-69]) — NOT of the master merge. An undrawn walker that the old probe
     used to hold back now walks on and re-opens the hall fight geometry.
2. **07000002 `walk_the_castle_all_objectives_real_health_agent` Bond dies** at tick 6915 at
   (−30.98, 6.46, 8.40) [L]. Master PASSES the same test (40.3 s) [L] ⇒ same lineage
   regression. (The `00agent` variant is the documented-ignored death,
   real-health-00agent-02-03-04's report.)
3. **`npc_doors_live::the_01_escort_driven_through_the_shoji_opens_it_and_crosses`** — the
   driven Mayhew **crossed the CLOSED, never-activated shoji** [L]: `crossed true`, closest
   approach 1.33, door state 0x2 (CLOSED), pos01 0.00 for 1800+ ticks. A closed door no
   longer blocks a driven escort. Two candidate mechanisms, both in the doorway-2 lineage:
   - the `hit_door` skip (`nav.rs follow`): while the port's `hit_door` binding stands the
     whole blocked/bounds test is skipped [M 0x15a2c8:67-69] — if the port's binding
     lifecycle has a hole (set on one doorway, not cleared on the path Mayhew takes), the
     skip becomes permanent and he creeps through every boundary afterwards, closed doors
     included; or
   - the raw undrawn slide itself (`drone.rs update_anim`): `pos += sin/cos` with NO probe
     and no door read [M 0x139268:70-93] — the arm matches the game as decompiled, but the
     game's DRONE-side door blocking (whatever stops an undrawn slider at a closed door —
     the door prop's own collision, or the drone never being undrawn while driven) is not
     modelled in the port's slide arm.
   Provenance for THIS test (master vs base) was not re-run in the window; the two confirmed
   01/02 regressions above already implicate the lineage.

Same-family failures the window did not provenance-check (suspected, not asserted):

- **07000003** `walk_the_full_route_and_complete_every_objective`: "servant 0's captor (chase
  3): drone 2 still alive after 10800 ticks (health 12)" — the chase never finishes the
  captor [L]. The `real_damage` variants PASS; the plain walk fails.
- **07000008** `..._real_damage_00agent` and `..._real_damage_agent`: Bond dies at ticks
  27791/27268 at the same spot (111.06, 13.54, −179.71) [L]; plain + default `real_damage`
  PASS.
- **0700000a** `walk_office_floor_qworm_keyboard_hatch` (all 4 variants) +
  `bond_stands_at_the_mouth_bridge_edge`: "[walk] the runner chase timed out" then
  "level_exit reached at +0 ticks" at the ring's FIRST waypoint — the mission fails the
  moment the ring starts [L]. A runner whose walk changed (raw slide) is the prime suspect:
  the chase stalling AND Bond's ring start drifting into a LOAD_LEVEL.

## The per-level sweep (individual runs, one binary per tool call, this sandbox)

| binary | result |
|---|---|
| walkreal_07000001_live | FAILED 5/3 — real_damage ×3 die (above); plain walks pass |
| walkreal_07000002_live | FAILED 2/1 — agent dies (above); real_health passes; 00agent documented-ignored |
| walkreal_07000003_live | FAILED 5/1 — plain walk chase kill (above); real_damage ×2 pass; 00agent documented-ignored |
| walkreal_07000004_live | ok 10/0 |
| walkreal_07000005_live | ok 14/0 |
| walkreal_07000006_live | ok 6/0, 1 ignored |
| walkreal_07000007_live | ok 6/0 — the fixed level itself: the waiter crosses the doorway green |
| walkreal_07000008_live | FAILED 4/2 — 00agent + agent die (above) |
| walkreal_07000009_live | ok 8/0 |
| walkreal_0700000a_live | FAILED 8/5 — runner chase + level_exit (above) |
| walkreal_0700000b_live | ok 7/0 |
| walkreal_0700000c_live | ok 3/0, 1 ignored |
| walkreal_0700000c_barrelrow_live | ok 1/0 |
| walkreal_0700000c_eastroute_live | ok 1/0 |
| walkreal_0700000d_live | ok 2/0, 1 ignored |
| walkreal_07000011_live | KILLED exit −9 mid-run (sandbox memory pressure — see suite note); not completed individually |
| npc_doors_live | FAILED 5/1 — the shoji crossing (above) |
| fix_guard_misc_live | ok 6/0 |

Not run individually in the window (the fast suite covers them; see its partial results):
`walkreal_07000011_route_live`, `walkreal_07000012_live` … `walkreal_0700004a_satchel_live`
(the 07000012-16, 1b, 41, 4a, 4a_satchel level walks) and the probe* walkreal set.

## The fast runner (partial — still running at the deadline)

`python3 tools/test/suite.py` (build ok: 1164 test binaries in 1739 s), run on THIS tree
(base+merge — the final tree; no code changes were made after the merge). Snapshot at report
time: 360 binaries run, then:

- FAILED `searchlight_live::the_beam_passes_through_bond_and_a_zero_alarm_writes_nothing` —
  a test ADDED by master's own 7876b7feb; its failure on this tree is a fresh merge
  interaction suspect (the base branch had no searchlight changes) — next session: run it on
  master and on the base to place it.
- FAILED `disc_golden` ×3 — the allowed slim-ISO failures.
- FAILED `identity_disc::the_real_disc_passes_the_identity_check` — a FOURTH disc-dependent
  test; ISO-content-dependent, needs eyes next session (the slim ISO may lack files it wants).
- FAILED `probe_pylon_render::probe_render_pylon_only` — not investigated yet.
- CRASH `probe_lane_gap`, `net_desync_report_live`, `mp_bot_weapons_live` (exit −9) — killed
  by the sandbox OOM while my parallel provenance builds/runs competed for the 4 GB; these
  three need a clean single-run re-run before they count as failures.

## Diagnosis direction for the fix (next session)

The two lineage arms are separable: patch one arm off (re-run the 01 fight + the shoji test),
then the other — that pins WHICH arm causes each failure, cheaply (both tests run in <60 s;
the lib rebuild per flip is ~2-3 min). Then:

- If the `hit_door` skip is the shoji culprit: audit the port's `hit_door` set/clear lifecycle
  against [M 0x153358:149-176] (the pair arm binds; the far-side arrival and the open-door
  resolution clear) and close the hole. The skip itself stays [M 0x15a2c8:67-69].
- If the raw slide is the culprit: the game's undrawn slider must still respect whatever stops
  it at a closed door — check the door's contribution to the movement pass
  (`NDrone2_Collision` 0x140e90:169-174 reads bounds; doors may sit in that set) and give the
  port's tick-end `push_out` the door term, rather than re-probing.
- The AI-wall question the task asks ([D the level's AI wall set]) is NOT yet the explanation
  for any observed failure: the shoji is a DOOR (state 0x2), not a missing AI wall, and the
  01/02/08 deaths are fight-geometry changes, not walkers off-path. No level needed a new AI
  wall in this window.

## Live checks for the main agent

1. **07000001 hall ammo-resupply fight** (fresh boot, agent difficulty): reach the hall
   resupply with real health and record the health exchange with drone #11 — if the game's
   exchange leaves Bond ≥ ~80 (not 88.5 → 6.5), the port's undrawn #11 walks somewhere the
   game's does not; the spot and Bond's health after the fight are the pin.
2. **The driven escort vs the closed shoji (07000001 door 1)**: drive Mayhew at the closed,
   never-activated shoji exactly as `npc_doors_live` does — the game must NOT let him cross
   (closest approach ≥ door thickness). If the game's driven Mayhew DOES cross, the port's
   test expectation (and its closed-door blocking for driven escorts) needs the decompile
   re-read instead.
3. **0700000a office floor**: walk the office floor to the ring start — watch whether the
   runner NPC reaches his chase position in the game (the port's chase "times out"), and
   whether a LOAD_LEVEL fires at (9.9, 1.3, 5.3) when Bond stands there after the chase leg.

## What's left

1. Pin which lineage arm causes each of the three confirmed regressions (the two-flip
   isolation above), then fix per the diagnosis direction — no probe back.
2. The 0a runner-chase family and the 03 chase-kill and 08 agent deaths: same isolation.
3. Re-run the three OOM-crashed binaries cleanly; finish the suite; the fourth disc test.
4. The remaining individual level walks (11_route, 12-16, 1b, 41, 4a, 4a_satchel) — the
   suite's lines for them are the backup evidence.
5. `searchlight_live` and `probe_pylon_render` provenance.

## Usage

not visible to me
