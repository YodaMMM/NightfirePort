## Your task: `level-0d-real-health-crossfire-2`, finish 0700000d's walk to the C5 door on real damage

Continue origin/s/level-0d-real-health-crossfire: branch from it, merge origin/master in, and read its report docs/reports/level-0d-real-health-crossfire.md. That session fixed the crossfire's port-side over-hit [M] and moved survival from tick 4177 to 9412. Two things still drain the health pool: the last roof guard's fire when Bond arrives on the deck, and #10's shots during the climb. For each, check the wake, the sight and the hit rate against the game [M the sight, accuracy and damage paths; D coll-scan for the rays]. Fix the port only where it differs. Otherwise change the walk's tactics: kill the roof guard before the deck arrival, climb between #10's shots, or take cover. Un-ignore walkreal_0700000d_live `walk_arrival_to_the_c5_door_real_damage` on real input only. Never lower the damage or health to make it pass. Run walkreal_0700000d_live, enemy_accuracy_live and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0d-real-health-crossfire-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox7 at 2026-11-03T08:24Z

---
## Report (done, 2026-11-03T10:38Z)

# level-0d-real-health-crossfire-2 — report

Branch `s/level-0d-real-health-crossfire-2` (cut from origin/s/level-0d-real-health-crossfire a37cdff7, origin/master merged in twice: 5ec77aea at the branch start, then 63e76f06 mid-session — the stun-anim guard work came in with it).

## Status: the un-ignore is NOT achieved. The two remaining drains are now MEASURED to the hit,
## the port matched the game in every path this session could check [M]/[D], and the sanctioned
## tactic changes were tried on the real input path and RETIRED with measurements (one of them
## made things worse). The twin stays `#[ignore]`d with the new measured reason; the walkreal
## tree is byte-identical in behaviour to the incoming branch (death at tick 5964, exactly).

## What's done

1. **The death anatomy, measured to the hit** ([dmg] log, real input, real health, the full route
   from the load): 10 hits for exactly 100.0, death at tick 5964 at the deck arrival
   (198.94, 16.03, −260.18):
   - **#10 — skin 0x5000093, the BAY guard (214.7, 12.1, −289.5)** (not a roof guard: placement
     #292, wait ch 0x2d [D the audit's kind-0x0f table]) — 3 hits (14+16+16 = 46) at ticks
     3242/3650/3980, every one while Bond stands at the LADDER TOP (213.1, 17.4, −234.5), d 55.3.
   - **#29 — skin 0x5000087, the west roof** — 4 hits (14+16+16+6 = 54) at ticks
     5423/5603/5783/5964 — the EXACT 180-tick burst cadence [D weapon 0x1e; M
     DroneWeap_NextBulletTime 0x179d40] — during the late wire ride and the landing.
2. **#10's fire is legal in every checked path.**
   - *Wake*: #10 wakes with the platform-drop group (the prior session's probe part E; unchanged).
   - *Hearing CANNOT be the path* [M]: the P2K shot's hearing entry weighs outer radius 40 [L
     SFXOutputData 0x403] < the 55.3 distance; `Sound_Alertness` 0x1ddc48 is a pure radius law
     (outer/inner falloff, no occlusion term at all); and the alert-broadcast receiver gate
     returns 0 for any drone more than 50.0 from Bond [M NDrone2_ReactToDroneAlertMsg 0x1437a0:77-79,
     the +0x1a0 field = |opponent − drone| per Drone_GetOpponentInfo 0x138f68:67-71] plus the
     own-alertness > 0.7 gate [M 0x1437a0:99-104]. The port carries all three laws
     (drone.rs `hear`/`enemy_alerts`, cited inline). So #10 acquired the ladder top BY SIGHT.
   - *Sight is geometrically legal* [D coll-scan]: the eye→ladder-top line from (214.7, 13.6,
     −289.5) passes the canopy corner (the canopy y 18.6 spans x 130..190 × z −270..−305 [probe21
     roof map]; the line at z −270 reads y 14.95, x 214.4 — south-east of the corner, no block).
   - *Hit rate is the game's own law*: the climber reads STILL on the climb (the zero-on-climb
     rule [M Player_Update 0x193600:129-134 — the previous session's port fix]), so #10's rolls
     are 45% a round [L tuning: base 100−5·acc, Easy ×0.6]; 3 hits over ~1100 standing ticks is
     the expected count, not an over-hit.
3. **#29 cannot be shot from anywhere the route can stand — the slab-edge law ([0d:ray], a new
   audit line in `shoot_far` printing the same eye→chest ray through `segment_hit`,
   `segment_hit_static` AND the bullet-law `segment_bullet_scoped` every 120 ticks).**
   - The ladder-top eye (213.1, 18.1, −234.5) is 4.9 BELOW the west-roof surface; the eye→chest
     ray to #29 dies at fraction **0.972-0.974 on the roof slab edge** (all three probes agree).
   - The deck eye is 7.5 below it over ~50 u — worse. The wire (y ~17.2) crosses the roof plane
     ~3 u east of the guard — the same slab. [D coll-scan: the guard's own DESCENDING rounds
     clear its roof edge because its eye rides 1.5 u above the slab — the mutual-sight
     asymmetry is real geometry, not a port path difference. The two port bullet paths (player
     `update_player_bullets`, drone `Bullet::update`) run the SAME `segment_bullet_scoped`
     with the same pass laws [M Collide_Update 0x1ea398:216-229, Collide_Intersect
     0x1ebf78-0x1ebf80] — no player-vs-drone path divergence exists in the port to fix.]
   - The prior session's open question (its live check #3, "does the REAL game's deck-arrival
     round reach?") is answered on the port side: no — and consistently.
4. **The sanctioned tactic experiments — run on real input, measured, retired:**
   - *The ray-gated sweep* (skip the passes whose static eye→chest ray dies, pass over to the
     next ray-clear guard): TWO runs. The first exposed an inverted gate predicate
     (`segment_bullet_scoped` returns Some = BLOCKED, None = CLEAR — the gate kept the blocked
     picks), fixed, re-run — and the gate still fails in practice: a guard whose static ray
     carries gets ENGAGED by the pass's fire, steps into the slab shadow (the game's
     reposition-when-engaged cover law), and the chase lands nothing in the 260 cap while the
     missed passes widen the engagement. Run results: the gated sweeps produced a 3-shooter
     burst wall at tick 3709 (#10 + #28 + #18 in one tick) and death ON THE WIRE at 4313;
     the corrected gate still died at 5270. The blind 4-pass sweep (2 kills at +201/+186,
     ~520 ticks wasted, one hunter left) is the measured-best — RESTORED.
   - *The deck-arrival kill attempt* (260 ticks at the ray-dying guard): REMOVED — it never
     landed in any run and every standing tick is a burst window. The deck legs stay MOVING.
   - *The fixed 500-tick deck-settle wait*: replaced with a grounded poll (cap 500, min 30) —
     a player moves when he lands.
   - Net behaviour: identical to the incoming branch through the ride (the same 10 hits, the
     same death tick 5964 — verified by re-run after the revert).
5. **The audits that found NO port difference** (so no port code was changed this session):
   the hearing pool (`Sound_Alertness` 0x1ddc48 / `DroneFunc_HandleSoundAlerts` 0x148a30 vs the
   port's `hear` — the falloff, the 0x248/0x11c-0x11e specials, the 0.01 share, the 60-frame
   cooldown-or-louder law all match); the bullet pass laws (both paths, above); the
   accuracy/damage paths (unchanged from the prior session's audit). The only code change in
   the crate under test is the new [0d:ray] audit line + the route's retired-experiment
   comments; no nf-game source change at all.

## How it was verified

- `walkreal_0700000d_live`: **1 passed, 1 ignored** (the non-damage twin green with the full
  mission in 18 s; the real-damage twin carries the measured ignore above).
- `enemy_accuracy_live`: **5 passed, 0 failed** (409 s).
- `nf-game`: **1041 passed, 0 failed** (the merged master's +1 test).
- The real-damage twin was run 4 times this session (the baseline + one per experiment); the
  final tree reproduces the incoming branch's death exactly (tick 5964, 100.0 through 10 hits).

## The full suite

`tools/test/suite.py` was started at 09:34 (it died: `cargo` missing from PATH under nohup —
the prior session's same trap) and re-started at 09:36; THIS sandbox kills background jobs
between tool calls (measured three times: two cargo runs and one suite run vanished mid-build;
only foreground commands survive), so no `suite.out` total line exists. The three batteries the
task names ran green in the foreground instead (above). The suite on the final tree is owed.

## What's left (the next message's levers)

1. **The ride's 54 is the hard wall.** #29 (or whichever west-roof guard survives the sweep)
   fires on the rider from a spot the route cannot shoot back at, and the ride's own mechanics
   fix the exposure window. Untried levers, in order of promise:
   - **Kill the sweep's THIRD guard before it steps**: the passes that land do so in ~200 ticks
     while the engaged guard is mid-step. A 5th/6th pass (the box re-picks the survivor each
     time) keeps rolling the dice — the current cap is 4 passes; the death runs never showed
     the 3rd roof kill landing, so this likely just buys more engagement.
   - **The un-engaged ride**: skip the sweep entirely (keep #16's kill) and ride against the
     UN-engaged roof — the pre-crossfire runs measured 32-48 inbound on the ride from the
     untouched roof, WORSE than 54 — this lever is measured dead.
   - **A higher eye before the grab**: no reachable point in the route exceeds the ladder top's
     eye. [S] unverified: whether the grab itself (the jump's apex) lifts the eye above the slab
     line for a shot DURING the jump-grab window — needs a probe.
2. **#10's 46**: the only lever is a shorter ladder-top stand (the sweep waste), and the sweep
   waste is what keeps the THIRD roof guard from engaging — the two costs trade against each
   other. A real fix needs the game-side answer (live check #1 below).
3. **The full suite** on the final tree.

## Live checks for the main agent

1. **Does the real game's #10 (the bay guard at (214.2, 12.2, −289.4), skin 0x5000093) shoot a
   player standing at the ladder top (212.5, 17.4, −234.5)?** PCSX2, 0700000d, stand there after
   the platform kills without firing a shot at the roofs: if the real #10 holds fire, the port
   over-acquires somewhere this audit could not see (the sight geometry says it SHOULD see —
   the port's shot pattern matches the geometry, so this check separates geometry-law from
   level-design intent).
2. **The mutual-sight asymmetry** (the prior session's check #3, now sharpened): stand on the
   deck at (199.25, 14.16, −260.16) and fire at the west-roof guard at (151.4, 22.7, −248.5) —
   the port says the round dies on the roof slab edge (0.974); if the REAL game's round lands,
   the port's player bullet path is missing something the drone path has (the two share
   `Bullet_update` in the decompile, so a live difference would be a real find).
3. **The engage→reposition law on the west roof**: fire one aimed round at a west-roof guard
   from the ladder top and watch its next 300 ticks (gdbwatch on Drone+0x54/+0x30) — does the
   REAL guard step to a new spot (the port's measured behaviour) and does its new spot break
   the ladder-top ray the same way?

## Usage

not visible to me — this environment exposes no token, credit, context-window or cost figures
for the session (no usage panel or API counters are reachable from the sandbox shell).
