# retune-nb-16-7 — 16 chasers-defeat 00 Agent real-damage walk, round 7

Branch `s/retune-nb-16-7`, from origin/master 7562f1e70, merged `origin/s/retune-nb-16-6`
79c09d3e5 per the task (16-6 was not on master). The merge resolved to master's side for
README (master deliberately dropped the driving credit line — 3dc1aff7e) and the gadget-census
test (master's 7562f1e70 is newer), and to the 16-6 side for the walk file's `#[ignore]` (the
ledger) and the docs. The task's `tools/test/suite.py` came in through the master merge.

Task: take the next untried lever from the ignore message (the ring-phase bleed), judge each
lever on its own real run. Real input only, no top-ups, never lower damage or health, never
change guard code. Helpers stay gated to the level and the difficulty (every new gate evaluates
`re_arm && station_armour && difficulty >= 3 && !w.tops_up()` — exactly the d3 real-damage
defeat walk; the d2 walk, the punch walk, the tops-up sister and the door tests are missed by
construction). The walk is still red; the refreshed ledger and the next levers are in the
ignore message.

## The stream moved again

The -6 ledger's numbers did not transfer: on the merged tree the re-measure died **t1027 AT
the ring east arrival** (17 hits for 100.0; the south-gallery pair #2/#5 dealt 86.0 [L run 1,
NF_DMG_TRACE]) where the -6 tree measured t1229 on the ring between ring 1 and ring 2. Master
gained 48bfea13f ("faster moving-collision rebuild and per-tick tables") after the -6 branch
measured, and the stream moved with it.

## Done — the measured lever session (each its own real run)

- **run 1 — the ledger's lever (the ring-1..4 bounded stands, the punch walk's ring-1
  gallery-rifleman lever extended to the defeat walk)**: the stands engaged at the ring east
  ARRIVAL and lost the race — Bond arrived at 33.5 hp and died mid-stand (t1027). Measured:
  the stands must start earlier than the arrival.
- **run 2 — the ring east split** (nav to the first ring-level node (14.22, 5.73, −81.07), the
  stands from there): the stands died INSIDE `switch_off_the_delta`'s cycling — the probe:
  gun 0x11 clip 2 spare 9 at the node, the P2K clip 7 carried 41 unfired (death t1019). The
  switch costs 61 ticks a try [M the holster/draw cycle the walker models] and is fatal under
  the pour.
- **run 5 — the Delta→P2K switch moved to the route head** (the corridor's quiet stretch; the
  run's first landed hit is t335 on the lane east) + the lane south dealer stand (dealer-first
  by the walk's own `player_hits` audit): the #2 kill at 27 u took 335 ticks (the P2K's
  2.1-a-round body taps, 10 rounds [L the kill trace]) for ~25 health — the walk reached
  t4313 and ENTERED the ring phase, but `shoot_bounded`'s 540-tick flank nav walked Bond east
  to the pocket's rim and the trench killed him (y −4.99).
- **run 6 — flank_hold_x 15.0 around my stands** (the walker's own opt-in, set and restored):
  the flank nav was blocked, but the blocked branch's 90-tick SIDESTEPS (±1.0 strafe) still
  walked Bond 16 u west off the lane into the trench rim (death t2815, y −6.68). Measured: the
  displacement is the sidesteps, not the flank.
- **run 7 — the walk-wide shooters_window (40, 7.5)** (the damage_log.rs opt-in field):
  MEASURED WRONG — it opened reactive fights on the GROUND legs (the ground hunter #1, y 0.8,
  in band there) and the fights' inputs shifted the whole stream (the #2 dealer's first burst
  moved t335 → t586). Reverted to per-leg, then removed entirely (see run 10).
- **run 8 — the peek stand + the per-leg wide window**: the peek stand (a file-local,
  NO-displacement aim+tap stand — the walk's own chaser-wait idiom) replaced `shoot_bounded`
  in the lane stand; the wide window on the level-phase legs let the reactive fight KILL the
  gallery brute #5 mid-nav for 14.0 (the cheapest answer this stream has measured) — then the
  node stand that followed stood still under the strafed-north rifleman pair's combined pour
  (#3 25.6 u + #4 21.3 u + #1, 3.5 a hit each) and bled 44.0 → 0 in 130 ticks (death t1598).
- **run 9 — no ring-level stands** (the reactive fights as the ring answer): the fight CHAIN
  re-picked the far riflemen and the #3 fight stood under the multi-guard pour — the same
  t1598 death. Measured: ANY stand at ring level eats 0.35–0.67/tick from 3–5 pourers; the
  ring phase cannot be fought standing.
- **run 10 — the final composition**: the early switch + the lane south dealer stand (the #2
  kill) + the default machinery (the 20 u window, the legs moving). The node arrival improved
  **47.5 → 58.0** [L run 10 vs run 1]; the ring phase's own pour (~30–58 for the phase [L
  runs 1/10]) ate it — death **t1620** at (10.70, 6.75, −77.94), ~10 u past the ring east
  arrival onto the ring.
- **run 11 — the ramp-2 re-stand** (the cheap hp-10 kills): shifted the stream and died
  EARLIER (t1428 at the ramp 1 arrival) — the run-7 lesson (a stand call shifts the stream)
  repeats. Reverted.
- **run 12 — the determinism check**: byte-identical to run 10 (t1620 at the same spot) after
  the revert. The final tree's composition is runs 10/12.

The ignore message carries the refreshed ledger: the landed levers, the measured-worse set
(do not retry: the ring-level stands, the ramp stands, the walk-wide window, the sidestep
displacement, the lane flank nav) and the next levers — (a) the fights' SPEED at the ring (the
dropped 0x1a SMGs deal 7.0 a round vs the P2K's 2.1; the ring clears' own loot walk already
fetches close drops — a d3 arm should prefer the owned 0x1a after the loot, which the clear's
`switch_to_a_deep_firearm` keeps off because the P2K's 36 rounds pass the ≥10 check);
(b) the fights' trade discipline (`abort_on_wound` breaks each fight after a volley so the
legs move between cheap kills); (c) the approach cut pushed further (a #5-then-#3 batch in the
lane stand loop).

## How it was verified

- Every lever above was judged on its own real run through the real input path (NF_DMG_TRACE),
  the traces kept in the session (not committed). The determinism of the final shape was
  verified by the identical repeat (run 12 = run 10).
- No `crates/*/src/` files and no `crates/nf-view/tests/common/` files changed: the diff vs
  the merged master touches only `crates/nf-view/tests/walkreal_07000016_live.rs` (the
  d3-gated blocks + the ignore message) and the docs. The sisters' streams are gated untouched
  by construction, and the whole binary's pass confirms it.
- **The whole `walkreal_07000016_live` binary at the final tree: 18 passed / 0 failed / 1
  ignored** (140.1 s, `--test-threads=1`, NF_ISO the onfoot ISO) — the d1 real-damage walk,
  the d2 real-damage walk, both tops-up sisters, the punch pin, the door/interactable tests all
  green; the 1 ignored is the d3 walk (the ledgered one).

## What's left

- The d3 walk (above): one session of the ring-phase speed/discipline levers (the ignore
  message names the three candidates in order), then the pit-squad stage unchanged from the
  16-5 green shape, then un-ignore.

## The full suite

`tools/test/suite.py` was started in the background per the task (`setsid nohup`, the
first minutes of the session): the sandbox killed it between tool calls before a single line —
`suite.out` empty, no build progress, no python/cargo process left (the same wall the -5/-6
reports documented; this sandbox kills every background job between tool calls). The
foreground four-crate build (~15–25 min of compile from zero) did not fit the remaining
window after the lever session. What ran, on the final tree:

- **The whole `walkreal_07000016_live` binary** (the task-touched crate's affected set — the
  diff touches ONLY this test file): **18 passed / 0 failed / 1 ignored**, 140.1 s,
  `--test-threads=1`.
- No `crates/*/src/` or `crates/nf-view/tests/common/` files changed, so no other crate's or
  level's binary set can move (rule 10's affected set is this one binary; rule 7 does not
  trigger).

## Usage

not visible to me
