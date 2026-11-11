# los-mask-repins-16-2 — report

Branch `s/los-mask-repins-16-2`, base `origin/s/los-mask-repins-16` (7cb3c5fbc) merged with
`origin/master` 784ba2d99. Task: 07000016's three defeat walks — Agent d2 first, then 00 Agent
d3 on the same levers, then Operative d1 — one walk per tool call, helpers gated to 07000016 and
the difficulty, real input only, no top-ups, never lower damage or health.

## Done

**Operative d1 (`walk_the_chasers_defeat_completes_the_mission_on_real_damage`) is un-ignored
and GREEN** — end health 45.4, **armour 45.2** (the pit-room armour #648 take FIRED: the arrival
tightened 0.6 → 0.25; the 0.6 arrival measured 0.6 off and the take never fired, the run ended
armour 0.0 [L the los-mask-repins-16 ledger]), 57 hits for 54.6, 8 return-fire fights, mission
complete at tick 15600 [L the final-tree run].

**Agent d2 (`walk_the_chasers_defeat_completes_the_mission_on_real_damage_agent`) is
un-ignored and GREEN** — end health 6.9, armour 0.0 (the #648 straps spent in the pit), 89 hits
for 93.1, 11 return-fire fights, the lift-out hand-back on the cinema key (5.385, 0.90,
−158.309) [L the final-tree run; the numbers are byte-identical across every re-run after the
gates settled].

The levers that carried them (all in `crates/nf-view/tests/walkreal_07000016_live.rs`; no
`common/` file touched, no `crates/*/src/` code touched):

1. **The gallery stand extended to the defeat walks** (the task's item for d2), keyed
   `station_armour` — the punch walk (`station_armour=false`) never takes it, so its measured
   timeline is untouched. `answer_the_gallery_from_the_ring` answers the visible at-level
   (±2.0) gallery muzzles from where Bond stands on each ring/station leg: nearest first,
   bounded 1800-tick `shoot_bounded` stands, no walk, no flank (the far-exposed FIGHT arm
   measured worse: the 2152 death). The d1 green run's stands killed #4/#5/#2 at 22-35.9 u
   [L]. Two hard-won laws inside it:
   - **The pick runs FIRST, side-effect free.** The d2 green timeline's hook never picks (the
     reactive fights reach the gallery first) — an unconditional weapon switch before the pick
     cycled the chain's Delta there and shifted the whole stream (the measured regression: the
     walk died at 1405 on the ring north, green at 18240 before it).
   - **The stand pick's reach is per difficulty** (25 u at Agent, 40 u at Operative/00 Agent):
     at d2 the fights reach the pair before any stand is needed, and the stands' own ammo
     spend dried the chaser fight (the measured regression: the 40 u stands spent the route's
     drops, the chaser fight opened clip 7 spare 0, death 9602); at d1/d3 the stands ARE the
     answer.
2. **The d2 pick laws** (`level_fights` + `hurting_first`, the d1 walk's own proven set, keyed
   `difficulty >= 2`): the measured 2548 death's ramp fights engaged the y 6.7 gallery pair
   from MID-RAMP — fight#1 held Bond at (12.2, −89.0) for 240+ ticks under the pair's
   crossfire, health 74.8 → 26.5 in one fight, and Bond reached ring 1 at 5.5 with clip 3
   spare 0 [L the same trace]. With the laws the ramp legs keep MOVING and the pair is
   answered at level on the ring; `hurting_first` fights the muzzle that is actually hurting
   Bond (the pair dealt 42.0 + 32.8 of the run's 100 while the plain nearest-visible pick
   ground on the closer ring guards).
3. **The pit walk-in** (the task's item for d1 — and the d2 death reproduced it exactly): the
   squad kill loop stood at the shelter and ground two full 10800-tick ladders on the 100-hp
   melee clone #12 (28 hp a ladder, exposed FALSE at 5.5 u behind the launch structure) while
   the pit AR guards closed [L the 17138 trace]. The walk-in advances in bounded 8 u bites
   toward the member while he has NO exposed aim (a player closes until he can see him, then
   head-taps), the walk legs' own return fire answering the dealers between — the measured
   runs' first nav leg killed the whole squad mid-advance (#12 the 100-hp clone, #14/#15/#13
   with it), which is why the advance is bounded: a player does not stroll PAST a dead target
   out of the pit room (the stroll cost the lift-out hand-back its pit-floor position). The
   walk-in's wire probe excludes the event-gated pulse poly (gate 0xf3 — both pulses are sat
   out before the squad phase; its polygon still claims the whole launch floor geometrically
   between events and refused EVERY walk-in until filtered [L the diagnostic run]); the
   always-on wires stay probed.
4. **The armour #648 arrival tightened 0.6 → 0.25** (the task's item): the take fired on the
   first tightened run and d1 finished with armour 45.2 (it absorbed through the pit and still
   had 45.2 left).
5. **The ramp-top sprint** (`station_armour`, difficulty 1 and 3 only — d2's green runs
   without it): the ring east leg's clear ran at the leg's START with Bond still on the ramp
   top (y 4.77) — the clear's level band picked the ring guard #0 at 5.4 u above him and the
   stand held Bond there under the gallery's plink, dead 3 u short of the ring floor [L the d3
   916 trace]. The sprint walks the leg FIRST and the clear + gallery answer follow on the
   floor. d1's regression without it: the walk SURVIVED (59.4 health) but the squad phase
   ended 6.5 u off the lift-out key and the hand-back assert failed — the sprint is part of
   d1's measured green timeline.

**00 Agent d3 stays ignored, with the ledger and the next levers in its ignore message** (the
task's provision). The d2 laws carried it 1020 → 1292 (the ramp-top sprint, the stands killed
#4, the pick laws) but the Hard quartet still kills on the ring: tick 1292 at (6.84, 6.81,
−75.07), 20 hits for 100.0 — #2 (skin 0x50000b2, weapon 0x1a, hp 20, (9.5, 6.7, −110.6)) poured
47.5 in 7 hits (7.0 a hit, Accuracy Hard [M 0x179740:74-88]) from 30-36 u and was NEVER visible
from the spots Bond's fights displaced him to (the reactive pick skipped him every tick); the
fights' drops slide below the walk-over take radius and the ring stands ran dry twice (clip 1-2
spare 9 [L the 1148/1292 traces]). TRIED this round: the d2 laws + `finish_fights` +
`bridge_wedged_nav` (d3-only gates now), the stands to 40 u, the sprint, the drop re-read walks
(arrive 0.3), `switch_off_the_delta` (the P2K over the Delta's gravity arc). NEXT LEVERS (in
the ignore message): cross the ring east → ring 1 stretch faster (skip the ring east leg's own
walk after the sprint so the ring 1 hook fires from the punch-proven sight line (6.0, 5.9,
−78.5) where #2 IS visible [L the capsule-join trace t893-1891]); or a pre-ring stand from the
ramp top's east edge; or the unseen-fights creep (needs `hurting_first` OFF — the pick chain
skips the unseen fallback while `hurting_first` is on).

## How it was verified

- `walkreal_07000016_live` whole: **17 passed, 0 failed, 1 ignored** (the parked 00 Agent
  sister) — release, `--test-threads=1`, `NF_ISO` the slim ISO, final tree. The 17 include the
  punch walk `the_chasers_punch_lands_the_record_damage` and both tops-up defeat walks — every
  shared-helper change here is gated (`station_armour`, `!w.tops_up()`, the difficulty), and
  their timelines are unchanged.
- `interactables_07000016_live` whole: **13 passed, 0 failed**.
- Determinism: the d2 green numbers (end health 6.9, armour 0.0, 89 hits for 93.1) reproduced
  byte-identically on the first run after each gate settled and again in the final-tree whole
  run; d1's summary (45.4/45.2, 57 for 54.6) likewise.
- No `crates/*/src/` code changed, so no other binary's coverage is owed. The fast runner
  (`tools/test/suite.py`) cannot run in this sandbox: background processes die between tool
  calls (three attempts documented — nohup, setsid, and with the rustup PATH; the first also
  hit a missing `cargo` on the nohup PATH). The per-binary coverage above is the substitute,
  same as the base task recorded.

## What's left

- **00 Agent d3** — the one still-ignored walk. Its ledger, the tried levers and the next
  levers are in its `#[ignore = ...]` message on the branch; the first lever (skip the ring
  east leg's own walk after the sprint so the ring 1 hook fires from the punch-proven sight
  line) is a small, well-evidenced change.

## Live checks for the main agent

None this round — every fix is harness-side and carried by trace evidence ([L] lines above).
The #648 take question from the base report is settled: the tightened arrival fires the take
(d1's armour 45.2 at the finish).

## Usage

The environment did not expose any token, credit or context-window figures for this session —
not visible to me. (No usage numbers were shown by the sandbox, the CLI, or the git host.)
