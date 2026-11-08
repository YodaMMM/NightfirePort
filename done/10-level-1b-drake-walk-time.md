# 0700001b: the Agent real-health walk to Drake's defeat never finishes

walk_arrival_to_drakes_defeat_on_real_health_agent in walkreal_0700001b_live runs past 590 s even alone and has never produced a result. Find where it loops (log the walk's leg and Bond's state per 600 ticks), fix the port if a game rule is stuck (a console, a wave that never thins, Drake's state), or re-route the walk as a player would; make it finish under 300 s or report why it cannot.

Reproduce on the real level first through real input, not a teleport. Compare with the decompile, fix, and test red on the old code. Run the walkreal_* binary of every level the change touches, one level per tool call. Branch from origin/master, push to `s/level-1b-drake-walk-time`, report in docs/reports/level-1b-drake-walk-time.md. No draw changes: name what is wrong if it is a draw issue.

---
claimed-by: sandbox5 at 2026-11-08T22:49Z

---
## Report (done, 2026-11-08T23:16Z)


---
## Report from the worker (done, 2026-11-08T23:17Z)

# level-1b-drake-walk-time — report (2026-11-08)

Branch `s/level-1b-drake-walk-time`, cut from `origin/master` b059df948. Task: the Agent
real-health walk `walk_arrival_to_drakes_defeat_on_real_health_agent` (`walkreal_0700001b_live`)
runs past 590 s even alone and has never produced a result; find where it loops (log the walk's
leg and Bond's state per 600 ticks), fix the port if a game rule is stuck, or re-route as a
player would; make it finish under 300 s or report why it cannot.

**The loop is found and fixed. It was the walker harness, not the port — no game rule is stuck.**
The walk now always produces a result (20.8 s at Agent), with the full ledger; it still cannot
*pass* on real health at Agent, and the report below carries the measured why.

## What's done

**1. The per-600-frame leg/state log (the task's instrument), kept in the file.** A test-local
`walk_log` module (no shared-helper changes): each route leg calls `leg(name, sim)` per tick;
it prints the leg's enter/exit with wall seconds and, every 600 frames, Bond's pos/substate/
health/armour/clip, the live-astronaut count, Drake's state, `MissileDeploy`, the mission
channels (19/46/10/09/0e/0b) and the objective records. Stderr only — no sim state, no RNG
draws, wall clock never touches Sim.

**2. The reproduction (the 590 s run, logged).** One full run of the ignored Agent walk on
master b059df948, real input from the load, no top-up [L this session's run]:

- `first-wave` frames 168..1774 — Bond full (100/50), the wave dies, ch 19 up. 1.3 s wall.
- `consoles` frames 1774..~11400: latches k5@3093 / k4@4317 / k3@5511 / k6@7083, each 457-691
  ticks inside its missile's p0 read [D P0_LEN; M 0x1aa23c-258], k8@8973 late by +1199; the wave
  ring grows 3→6→9→11→15 as the missile scripts spawn their waves [D spawners #165-181]; Bond's
  50 armour gone by f4317, 100 health ground to 0 by f11385 (43-class ledger, −3.5 a hit [M
  0x1d3a80 case 0x700001b Normal 0.7 × weapon 0x6a 5.0]); m1/m2/m7's consoles never latch →
  `MissileDeploy [3,3,2,2,2,2,3,2]`.
- **The loop, frame ~11400 onward: Bond dead (hp 0.0, clip 0) and the console leg never ends.**
  `shoot_consoles` had no alive/fail guard, so each outer iteration re-entered the tail-console
  wave fight — 15 astronauts stay inside 25 u of a body that cannot shoot (the fight's own done
  gate cannot fire), k1 never latches, and the fight burns its 2800-tick cap before the outer
  loop continues: 12000 × 2800 ≈ 33.6M ticks ≈ unbounded wall. That is the 590 s+ no-result run
  the task names. (The prior round had already measured the shape: "a dead Bond inside
  shoot_consoles ground the loop for 480+ s — worth an alive-guard in a future round";
  docs/reports/real-health-00agent-16-1b-2.md.)

**3. The fix (harness-side, opt-in by construction).** `shoot_consoles` now takes the leg's
`alive` predicate and a `stuck` gate — `!alive(s) || s.drones.mission.status() > 1` — checked
at the top of the outer loop and added to the inner wave-fight's done gate. Rationale, all
measured: the console channels' only writers are Bond's own shots [D the console UI scripts'
t682 write], a launched missile's state is final [M 0x1aa2cc], and the wave only grows, so past
a death or a launch every further tick is grind; a passing walk never sits at status > 1 here
(the route's own fight-gate assert pins `status() <= 1`), so the guard is inert on green runs —
verified: the whole binary below is green. The three real-health ignores now carry fresh
per-tree ledgers (the file's convention from the prior round).

**4. Why the walk cannot *finish* (pass) under 300 s at Agent — the game's own rules, not a
stuck port.** Every candidate "stuck rule" checked against the decompile-side pins this file
already carries:

- **A console:** the race is not stuck — k5/k4/k3/k6 latched with 457-691-tick margins while
  Bond lived; the tail group (k1/k2/k7) failed because the wave grind cost the health, not
  because a channel lost its writer.
- **A wave that never thins:** the wave thins to real damage — the first wave died in 1.3 s of
  sim (frames 168-1774) to the walker's real fire; the console-phase wave grows because the
  missile scripts spawn while Bond fights [D], and the walker's station-level fights cannot
  answer the above-deck band (the prior round's measured law).
- **Drake's state:** never reached — Drake's chain (ch 46 → ch 10) sits behind missile 1's
  disarm script [D 0600091c], and m1 launched (deploy 3) before its console latched.

What remains is the difficulty itself, measured across three rounds now (the 41-4a tactics
round: waves-first scoring, the prio-aware shelter, the Drake-close banding — best 32-36 hits
for 100.0 in the Drake stand; the 16-1b-2 round: the console phase is deadline-locked, every
same-station fight repositions Bond and slips the one-tick branch reads; this round: the fresh
ledger above). The wall-time budget was never the blocker — the Operative happy path completes
the whole mission well inside 300 s (the whole binary ran 85.5 s including it, below); the
blocker is surviving the wave ring on 150 ehp with the walker's holds, which no shipped tactic
has yet beaten.

## How it was verified

- The 590 s+ reproduction and the post-fix runs both went through the real input path
  (`cargo test --offline --release -j2 -p nf-view --test walkreal_0700001b_live <name> --
  --ignored --exact --nocapture --test-threads=1` on `NF_ISO`): no teleports, no
  health/ammo/channel writes, no top-up (`w.top_up_health = false`), the load's own difficulty
  (asserted in the body [M GameState+0x28]).
- The fix's effect, measured: the Agent walk 590 s+ (killed by the outer timeout, no result) →
  **20.8 s, FAILED at the death frame with the ledger** ("console 1 latched by the real shot" —
  the first never-latched console); d1 15.3 s (death f9081); d3 8.5 s (death f6251, the accuracy
  rows ordering the deaths [M DroneWeap_DoBulletAccuracy 0x179740:74-88]).
- Whole `walkreal_0700001b_live` binary, `--test-threads=1`: **16 passed / 0 failed / 3
  ignored, 85.5 s** — the Operative full-mission walk (`walk_arrival_to_drakes_defeat`, the
  other `shoot_consoles`/`play_the_mission_opts` caller) green with the guard in place.
- No shared helper changed (all edits are in `walkreal_0700001b_live.rs`); no other level's
  binary is touched by the diff, so no other `walkreal_*` run is owed.

## What's left

1. **The Agent pass itself** — the route-level re-plan the prior report named, still the next
   lever: (a) a chosen intercept spot the wave is fought at BEFORE it reaches the console
   stations (the wave pursues Bond's live position, so an intercept that ends away from the
   stations keeps the undisturbed latch geometry), or (b) an undisturbed k3/k6/k8 rush with the
   wave fought only on the crossing legs (the tail's ~1500-2200 margin absorbs fights; the
   k3/k6/k8 waits do not). This session adds the harness guard and the fresh ledger but not the
   route; the ignores carry the numbers.
2. **The Drake-stand survival** (behind lever 1): the 41-4a round's three tactics iterations
   stand; Bond's dps/reload cadence and the waves' sight-lines remain the named work.
3. The leg log stays in the file (stderr only); if it should ship quieter, gate it behind an
   env check in a future round.

## Live checks for the main agent

Carried unchanged from docs/reports/real-health-00agent-16-1b-2.md (they decide lever (a) vs
(b), and still stand on this round's ledger):

1. **0700001b, the k6/k8 hub waits' sight-lines**: a `--bond 32,-5.4,0` render at the death
   spot looking up at the wave band — is any undisturbed hold near the hub (±5 u, y −6..+3)
   out of the wave's lines while the k6 UI (station y ~3.5) stays shootable?
2. **0700001b, the k1/k2 station's above-deck answer**: a `--bond 20,-4.4,0` render looking up
   — is there any body-fit point at y 3-6 near x20 from which the y 4.9-10.6 wave band is both
   shootable and cut?

## Usage

not visible to me (no token, credit, context-window or cost figures are exposed anywhere in
this sandbox session; nothing in the environment reports them).
