# real-health-00agent-06-8 — the report

Task: the 00 Agent castle walk (`walkreal_07000006_live::walk_the_castle_interior_to_the_party_real_damage_00agent`)
on real health. Continued the numbered chain: branches -06-5/6/7 were closed by the driver but
their branches no longer exist on origin (only the queue stubs survive, and -06-7's report is
lost), so this session rebuilt on the latest surviving branch `-06-4`, merged current
`origin/master` (784ba2d99), and took the stubs' named levers. Branch:
`s/real-health-00agent-06-8`. Only file touched: `crates/nf-view/tests/walkreal_07000006_live.rs`
(+ WORKLOG + this report).

## Status: GREEN — the ignore is removed. The whole 06 binary is 7 passed / 0 failed / 2 ignored
## (the ignored pair: the assertion-free probe and the d2 Agent sister, which is master's own
## ledger and a separate task). The walk: 14106 ticks, end health 6.2, 21 hits for 93.8,
## deterministic across two whole-binary runs.

## What this session found (all measured on the merged tree)

1. **The merged tree moved the wall.** Sessions -06-5/6/7 worked on master 22d5836b7-era trees;
   the guards' own sight rewrite [823ff190b "Guards see the way the game's do", 2026-11-10] and
   the glass/doors commits landed after them. On the merged tree the -06-4 ledger no longer
   applies: the walk dies at the door-#47 APPROACH, not the east strip.
2. **The court return-fire burns the clip on angles whose rounds never land.** The run00b/c
   traces: return-fire taps at err 0.0000 on the exposed part from d 8.1 (clip 20 → 13 → dry)
   while #4 walks down the recess shooting (24 hits for 64.2 before Bond reaches the door
   plane); the first damage Bond lands on #4 comes from the door plane (d 4.0, hp 10 → 8); the
   run00b death is t2989 at clip 0 spare 0 with #3 at full hp 20.
3. **The plain index-order pre-clear fights the wrong guard.** run00d: `clear_the_spot` picks
   the room's #3 first — his rays the doorway plane blocks (the probe's ray matrix: from the
   court to #3's post the chest rays pass the window glass, but from the door plane the flight
   hits the z −16 wall at 0.828) — and burns the whole budget on him while the threshold guard
   #4 (untargeted, full hp 10) chews Bond point-blank: 97.2 in 19 hits, death t2801.
4. **The probe** (`probe_the_guardhouse_door_rays`, assertion-free, `--ignored`, in the 06
   file) pins the geometry: the flight/aim/bullet queries agree on every lane; the closed leaf
   is visible to the flight; the court chest rays to the room pass the window glass; the room's
   own ammo box (weapon 0x18, 90 rounds) sits at (34.72, 40.0, −24.45) behind the door.

## The fix (two d3-gated levers, both file-local)

1. **The door-#47 approach leg keeps return fire OFF at 00** (`walk_from_the_guardhouse_court_at`):
   the player closes to the door under fire and fights from the door plane, where the rounds
   land. d1/d2 keep their court return fire — their timelines are untouched.
2. **`clear_the_spot` picks the recent hitter first at 00**: the guard who is actually shooting
   Bond, over the fights' own 240-frame window [M the return-fire pick in
   `damage_log.rs:698-703`], ties and no-hitter phases keeping the plain index order (stable
   sort). d3-gated via `sim.drones.difficulty >= 3`.

No `crates/*/src/` change; no shared helper change (`clear_the_spot` is 06-file-local); no
health/ammo/channel writes; the only inputs are the walker's own view drives and trigger taps;
the shared RNG untouched (the flag and the pick order draw nothing).

## Verification (this session, on the pushed tree)

| Binary | Result |
| --- | --- |
| `walkreal_07000006_live` (whole, 9 tests) | 7 passed, 0 failed, 2 ignored — twice (deterministic; the 00 walk's end state byte-identical both runs: t14106, health 6.2, 21 hits for 93.8) |
| `interactables_07000006_live` | 5 passed, 0 failed |

The full suite did not fit the window (the sandbox's fresh target needed a 4-minute rebuild of
the 06 binary alone, and the runner must run in the foreground here — background jobs are
killed between tool calls). The diff is one test file, so the lesson-10 set is the 06 binary
plus the interactables binary — both green. The next session should still run the fast runner
on this tree (`NF_ISO=<abs> nohup python3 tools/test/suite.py > suite.out 2>&1 &`) before any
further work, per the standing rule.

## What's left (next session's levers)

- The **d2 Agent sister** is still `#[ignore]`d with master's own ledger (the raw-plate contact
  shifts the balcony drop and the walk dries at the mouth, dying to strip post #25 at t17089) —
  that is task 50-real-health-agent-06's scope, not this one.
- The 00 walk's 6.2 end health is razor-thin. If any future master change shifts one hit, the
  margin is gone. The known residues in the 14106-tick run's ledger are worth one census pass
  (which guards dealt the 93.8) to name the next shave before it is needed.
- The queue stubs' unmeasured levers from the lost -06-7 session (the unseen-bail cap, the pair
  enable-delta census) are moot now that the walk is green on this tree, but re-run them if a
  future master merge re-opens the guardroom phase.

## Live checks for the main agent

None required — the wall settled in-sim with the ray probe, and the walk's green state pins
itself. If the main agent wants one: a PCSX2 watch on door #47's leaf angle during the court
approach would confirm the guards walk out through the leaf exactly as the port models it
(the port's #4 descends the recess 40.6 → 39.8 during the engagement; the disc census has him
at his post (36.15, 41.0, −19.36) until he spots Bond).

## Usage

Not visible to me: this environment exposes no token, credit or context-window figures for the
session.
