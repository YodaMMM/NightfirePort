## Your task: `los-mask-repins-03`, the 07000003 walks green again under the game's guard sight

Branch from origin/master (823ff190b turned on the game's guard sight: per-object masks, the cel walk, the sight rotation, the guard's own cel, the own-cel boundary push, FellOutMap). Read docs/reports/los-mask-repairs.md on origin/s/los-mask-repairs (reports do not go to master). These walks were ignored because the game-correct sight moved their fights: walk_the_full_route_and_complete_every_objective_real_damage_agent. Read each ignore message. Then re-route each walk with real input so it passes, and remove its ignore. Never lower damage or health, no top-ups; helpers gated to 07000003 and the difficulty. Run walkreal_07000003_live (and its sister binaries) and interactables_07000003_live.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/30-los-mask-repins-03`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox4 at 2026-11-10T18:45Z

---
## Report (done, 2026-11-10T19:46Z)

Closed by the driver: the report is on `s/los-mask-repins-03` (docs/reports/los-mask-repins-03.md); the worker didn't run finish.sh.

---
## Report from the worker (done, 2026-11-10T19:56Z)


---
## Report from the worker (done, 2026-11-10T19:57Z)

# los-mask-repins-03 — the 07000003 Agent real-damage walk green again under the game's guard sight

Branch `s/los-mask-repins-03` = origin/master `c120d7b49` (+ three commits). The task: the
07000003 walks the game-correct guard sight (823ff190b) moved had to come back green with real
input, each ignore removed, never lowering damage or health, no top-ups, helpers gated to the
level and the difficulty.

## What was ignored, and why

`walk_the_full_route_and_complete_every_objective_real_damage_agent` (the walk the task names)
carried the pre-seeded ledger: Bond died at tick 164663 at (3.0, 6.0, −33.3) to the shotgun
captain #16 (skin 0x5000038, weapon 0x1c) at 1.2 u, the hand gun dry (clip 0 spare 0), at 41.2
health; "green with the flat feet-height boundary push".

The second real-damage sister in the file, `..._real_damage_00agent`, is the older
real-health-00agent-02-03-3 ledger (the door-8 cluster approach), not a los-mask ignore; see
"What's left".

## The reproduction (before any change)

Running the ignored Agent test on this branch reproduced the ledger exactly:

- death at tick 164663 at (3.01, 5.96, −33.34), 42 hits for 100.0, 16 return-fire fights;
- #16's eight pellet hits all landed at the death point (41.2 total, every round at d 1.2 —
  under 3.0 u every drone round hits [M DroneWeap_DoBulletAccuracy 0x179740:113-115]);
- the fights' own end law gives up on a dry gun, and the fights' weapon cycle at the death
  printed "cycling weapons (0) (weapon 0x2 clip 0 spare 0)" — no armed weapon anywhere on the
  walked-path corridor [L the run].

The three shooters the lane legs answered (#9 14.7 in 7, #11 28.0 in 4, #17 14.0 in 2) all
died to the return fire; the walk entered the exit moat-node leg at 41.2 with an empty gun and
the lane captain's first point-blank volley was unanswered.

## The fix (test-side, level-local, d≥2-gated)

The route now scavenges the way a player does. All of it lives in
`crates/nf-view/tests/walkreal_07000003_live.rs`, every arm behind `if difficulty >= 2`, so the
Operative walks (d1 plain + d1 real damage) keep their pinned timelines byte-identical:

1. **The moat band and the west lane rearms** (the gate-approach legs): after the moat-east leg
   and after the west-lane legs, `w.resupply` (the walker's walked-path-corridor loot — real
   floor pickups only, no writes) + `stand_shooters_down_03` (the file's own recent-hitter law).
2. **The door-8 cluster's guns looted** right after the mouth fight: the cluster carries the
   route's guns from there on (#18 0x1c shotgun, #2/#19 0xe [L the gas-rng-draws-6 probe]);
   the exit legs met the captains with no armed weapon on the corridor because these drops
   were never taken.
3. **A safe rearm** after the Dragon Safe take: `resupply` + `loot_the_drops_03` +
   `stand_shooters_down_03`, before the fight-bearing exit legs.

No damage, health or ammo writes anywhere; no shared helper changed (`common/` untouched, no
`src/` change at all — the whole fix is in the one test file).

## Result

The re-routed Agent walk passes: the exit moat-node fights now kill BOTH lane captains (#15
killed true at tick 151277, #16 killed true at 151381) and the walk finishes with the exit
fired (`level_exit Some(0x07000004)` through the real reload path). `#[ignore]` removed; the
test body documents the ledger and the levers.

## What ran (release, --test-threads 1, NF_ISO on the slim ISO)

| binary | result |
|---|---|
| walkreal_07000003_live (whole, run twice — before and after the d3-gated sidestep lever) | 6 passed / 0 failed / 2 ignored both times — the un-ignored Agent walk, the d1 plain + d1 real-damage walks, the short route, the key-gate sister, the servant-fail test |
| walk_07000003_live | 2 / 0 |
| interactables_07000003_live | 9 / 0 |
| door_contact_refine_07000003_live | 3 / 0 |
| pile_hits_07000003_live | 3 / 0 |
| void_recovery_07000003_live | 4 / 0 |
| wiring_07000003_live | 4 / 0 |

My change touches only this test file, so these seven binaries are the crates'-tests surface I
could affect; no `src/` change means no lib test can move. The ignored two are the 00agent
walk (below) and the door-8 load-state probe.

## The full suite

`tools/test/suite.py` was started in the background twice (the second start detached with
setsid); background jobs do not survive between tool calls in this sandbox (the prior sessions
recorded the same), so the per-binary totals above are the tree's state — `suite.out` never
printed a build line in either start. The change is confined to one test file with no `src/`
or `common/` edits, so the other binaries' timelines are master's own.

## What's left

- **The 00 Agent walk stays ignored**, re-ledgered on this branch, and one of its named levers
  landed: the d3 sidestep ladder now goes WEST first (the moat band the route has crossed since
  the moat-east anchor) and a sidestep hop or a re-anchored arrival more than 1.5 u above the
  anchor's floor does not count as arrived (d3-gated, no ticks added at d1/d2 — the whole
  walkreal binary re-run green after the change). The roof death moved 73k ticks deeper:
  - before the lever, the walk died at tick 53672 — the east sidestep's nav rode the courtyard's
    upper rows (y 7.85), the re-anchored walk_to "arrived" on the roof (y 10.12 at
    (11.19, −29.66)), and the east-wing sniper #6 (26.2, 9.9, 0.7) saw Bond along the upper
    row's windows: one 40.0 round at d 22.8 ended it;
  - after the lever, the walk dies at tick 126500 at (24.55, 6.57, 7.72) — ground floor — to
    shotgunner #5 (0x500003a, 0x1c) at 2.6 u, 40.0 in 8 pellets with #5 left at hp 9: the
    fights' aimed stand holds position and the closing shotgun's point-blank volley lands.
  The named answers for the next session: the 08 walks' strafe-dodge lever ported into this
  file's `aimed_stand_03` (d3-gated), plus the three older levers (an approach north of door 8,
  the 87000-tick (35.6, 8.7) door stall on the post-rescue west-lawn leg, the exit-lawn #16
  post-kill volley). This is the real-health-00agent-02-03-3 follow-up's scope.
- Unproven without the real game: whether the game's guards stand where the re-routed walk now
  meets them (the exit moat-node captains #15/#16, the safe-area shooters). Live checks for the
  main agent, if wanted: with a fresh boot on 07000003 at Agent difficulty, walk the front-gate
  route and watch guard #16 (placement near (3.4, −32.3), weapon 0x1c) — the expected result is
  he holds the west lane band and his volley pattern matches the port's (8 pellets, ~41 damage
  point-blank); and whether the door-8 cluster's guns (#18 shotgun) drop where the port drops
  them.

## Usage

not visible to me
