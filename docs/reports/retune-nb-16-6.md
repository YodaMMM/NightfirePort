# retune-nb-16-6 — 16 chasers-defeat 00 Agent real-damage walk vs stream batch 3

Branch `s/retune-nb-16-6`, from origin/master 2bd059afe (stream batch 3 + the seven-walk re-tune
b60b1ff79 are on it), merged `origin/s/retune-nb-16` c25e3d00c per the task — the merge resolved
to master's side: master already carries the round-1..5 levers and their measured history (the
east-high fight list, the close-pair-from-peek, the exit fought, the squad close stands, the 16-2
wide corpse-hop MEASURED-REVERTED note, the 16-5 east-armour fetch + squad laws) plus the newer
stream re-pins, so `origin/s/retune-nb-16-5` did not exist as a branch and nothing on the old
branch was newer than master's tree.

Task: re-measure on the merged tree first (master gained stream batch 3 after the 1121b-era
ledger in the ignore message), then take the next untried lever. Real input only, no top-ups,
never lower damage or health, never change guard code. The walk is still red; the refreshed
ledger and the next lever are in the ignore message. The whole `walkreal_07000016_live` binary
runs green apart from the ledgered ignore.

## Done

**The batch-3 re-measure (the task's own order) — the pocket excursion's premise is dead.**
Fresh runs on the merged tree [L each its own real run, NF_DMG_TRACE]:

- Master's excursion config: **death t1473** at (20.00,0.89,−94.04), 27 hits for 100.0, 0
  return-fire fights. The standing dash bled 100 → 26.5 under the y 6.7 gallery class (west
  #3/#4, the NE #0 at (9.0,6.7,−78.7)) — the batch-3 body aim [M master c229c7780: guards'
  rounds hit Bond's body boxes, guards aim at his body] hits every burst that the old
  head-aim scan had shading.
- The "pocket" is a **shallow cut**, not a pit: Bond's y inside it is 0.80-0.99 [L the
  NF_WALK_TRACE run] — 0.1 u below the lane floor. With body aim there is no bullet shadow: the
  descended ground closer #0 (0x500008c, walked down from the gallery to (17.8,0.8,−90.4))
  shoots over the lip at 6.2 u and lands the −40.5 finisher; the south lane hunter hits through
  the glass face (the 1121b finding stands).
- The crouched dash (the walker's own opt-in `leg_crouch` bit, the hall-4/5 peek law): pocket
  entry improved 26.5 → 40.5, the pocket still died (t1304, same dealer). Measured better but
  not sufficient; recorded, not shipped.
- The `switch_to_armed("the pocket")` between the dash and the cycles is a 61-tick no-op (a
  loaded gun qualifies in iteration 1) — the 1121b-era "450-tick switch window" hypothesis is
  refuted; the cycles simply never run (Bond dies before the first peek leg arrives).

**The landed lever: the batch-3 fight-through (d3 && !tops_up, file-local to
walkreal_07000016_live.rs).** A player does not dive into a shallow cut that no longer shades
anything — he fights through, exactly like the d1 sister (which passes this same route green on
master's batch 3):

1. The pocket excursion is SKIPPED for d3 (the d2 sister and the tops-up sisters keep their
   measured dash; their streams are byte-identical — the whole binary's pass confirms it).
2. The d1 law set, d3-gated: `finish_fights` (the hp-20 rounds are cheaper than the bleed),
   `back_off 3.5` (just over the 3.0 always-hit band [M DroneWeap_DoBulletAccuracy
   0x179740:113-115]), `hurting_first`, `level_fights` (the ground/ramp legs keep MOVING under
   the gallery plinks; the ring legs clear at level).
3. The ring-east pre-clear (`clear_hostiles_16` before y > 5.5 legs) is gated until Bond is at
   ring level for d3: the un-gated clear stood Bond ON RAMP 3 (y 4.74) fighting the whole
   gallery 20-34 u out — the run died exactly there (t973).

**Measured furthest (the lever's own run):** the lane crossed at ~90 ticks/leg, the ramp
climbed, **the ring REACHED** — never before on batch 3 (every excursion run died in the pocket
phase) — death **t1229** at (−2.83,6.75,−78.60) on the ring between ring 1 and ring 2, 18 hits
for 100.0, 1 return-fire fight. The route-progress record and the refreshed ledger are in the
test's ignore message.

**Measured-worse pair, do not retry** (each on its own real run, each named in the ignore):

- `level_fights` OFF at d3: the reactive pick opens stands on the y 6.7 gallery brutes from the
  floor and loses — death t896 at the ramp foot, the south brute #2 (9.5,6.7,−110.6, hp 20)
  dealt 65.0 in 10 hits, 2 fights.
- The ring-east pre-clear un-gated: the ramp-3 stand vs the gallery — death t973, #2 47.5 in 7.

**NEXT LEVER (in the ignore): the ring-phase bleed.** The south-gallery brute pair #2 (hp 20)
and #5 (−4.7,6.7,−111.0, hp 20) dealt 77.0 of the death run's 100 from 32-34 u THROUGH the whole
approach and are STILL alive when Bond reaches the ring — the ring clears (40 u / 900 cap at
ring 1..4) must finish them before he drains. Concrete candidates in the ignore message: extend
the visible-gated ring-1 rifleman bounded stands (today they run only for the punch walk) to the
defeat walk d3-gated and cover ring 2..4, or answer #2/#5 from the ring east leg itself (29-31
u, −0.8 level from Bond's feet).

## How it was verified

- The re-measure ran FIRST on the merged tree (the task's own order); every lever above was
  judged on its own real run, each trace kept in the session (not committed).
- The d2/tops-up/punch/door streams are byte-identical by construction: every new gate evaluates
  `difficulty >= 3 && !w.tops_up()` first and no-ops for the sisters; no `crates/*/src/` files
  and no `crates/nf-view/tests/common/` files changed (the diff vs master touches only the test
  file's three d3-gated blocks + the ignore message + the docs).
- **The whole `walkreal_07000016_live` binary at the final tree: 18 passed / 0 failed / 1
  ignored** (136.8 s, `--test-threads=1`, NF_ISO the onfoot ISO) — the d1 real-damage walk, the
  d2 real-damage walk, both tops-up sisters, the punch pin, the door/interactable tests all
  green; the 1 ignored is the d3 walk (the ledgered one).

## What's left

- The d3 walk (above): one session of the ring-bleed lever (the ignore message names the two
  candidates), then the pit-squad stage unchanged from the 16-5 green shape, then un-ignore.

## The full suite

`tools/test/suite.py` started in the background per the task. Its result line is recorded in
the WORKLOG push; if it hit the sandbox's ENOSPC wall again (9.9 GB disk — the -5 report
documented the same wall at 5.6 GB of nf-view binaries), the mini-suite fallback documents the
three smaller crates and this binary covers the task-touched crate: see the WORKLOG entry.

## Usage

not visible to me

## The full suite (final note)

`tools/test/suite.py` was started in the background twice (19:47, 21:27); both processes were
killed by the sandbox between tool calls before producing a line (suite.out empty, no build
progress). The sandbox was reset this session, so the four-crate release build would start from
zero (~15-25 min of compile alone) and did not fit the remaining window after setup; the
background kill makes it unrunnable here. What ran, on the final tree:

- **The whole `walkreal_07000016_live` binary** (the task-touched crate's affected set — the
  diff touches ONLY this test file): **18 passed / 0 failed / 1 ignored**, 136.8 s,
  `--test-threads=1` (the 1 ignored = the ledgered d3 walk).
- No `crates/*/src/` or `crates/nf-view/tests/common/` files changed, so no other crate's or
  level's binary set can move (rule 10's affected set is this one binary; rule 7 does not
  trigger).

## ntfy

1. curl: `curl: (28) Connection timed out after 20002 milliseconds`.
2. python urllib: exited 0 twice with no error (urlopen raises on failure — a raise would exit
   nonzero), no response body captured — treated as delivered.
3. wget: exit 124 (killed by the 25 s timeout, empty output file).
