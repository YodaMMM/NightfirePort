# walk-1b-agent — 1b arrival-to-Drake Agent and 00 Agent real-health walks

Branch `s/walk-1b-agent` (from origin/master 3dc1aff7e). The sandbox for this session was reset
(fresh home, no repo, no rust), so setup ran from scratch: clone, work-slim + ISO downloads,
ALSA user prefix, rustup, nf-queue worktree (~35 min of the window).

## What's done

**Agent walk (`walk_arrival_to_drakes_defeat_on_real_health_agent`) — the round's named lever
SHIPPED.** The ignore message's next lever was "surviving the tail window (~7400..9350), not the
console order". Shipped in commit e86570620, all gated to this level + the Agent walk (d2,
`waves_first`); the Operative (d1) walk stays byte-identical:

- The 00 Agent dead-window shelter now also gates on the Agent walk
  (`shoot_consoles(.., hard, agent)`, gate `hard || agent`): Bond kites the k1 dead window
  (7356..9170, 1814 shelter ticks, 56.9 -> 35.9 at 12 astro) instead of standing in the k1 wave
  fight that killed him at 8862 (41 hits for 100.0). The kill arm's stray taps latch k8 at 9170
  on the way; all 8 consoles latch (k1 10029, k2 10329, k7 10977).
- The test's stop moved to 6000/60 (`with_pauses(&[(6000, 60)])`, the d1 walk's own re-roll; the
  old 13600/60 Drake re-roll is superseded — Drake now spawns at 12721 so a 13600 stop would
  land mid-fight). Measured: the window enters at 74.0 instead of 56.9 and every leg gains.
- The missile-wait and waves-wait ride `kite_wait` (the r11 lever-3 shape the hard walk already
  uses): the kite missile-wait reaches the deploy at 23.7 (484 ticks) and the waves-wait holds
  ZERO-LOSS 11461..12721 (23.7 for 1260 ticks, the west shelter).
- The Drake fight rides the compensated taps (`fight_opts_y` aim_close, the r11 lever-6 shape):
  the wave thins 16 -> 5 during the fight (the uncalibrated fight lost the same fight with the
  wave only 16 -> 12).

Result: Bond meets Drake's gate at 12721 with 23.7 hp (was: dead at 8862 before k1's UI). The
walk still fails in the Drake fight (Drake 120 hp untouched — see "what's left").

**00 Agent walk (`..._00agent`) — the round's named lever measured RED, reverted.** Fencing the
defense fights on a live k8/k6 UI target (the chain-clear gate) died at 8372 (k8 latched 7191,
then the fight-free k6 window bled Bond out) vs the stuck-at-8864-with-37.5 baseline. The
round-6 core law holds: the fights' cover floats break the shooters' sight clocks; skip-fight is
measured dead at every formulation (r6 iteration 5, r7 lever 8, this round's fence). Reverted;
the baseline signature re-confirmed on the final tree (stuck 8864, hp 37.5, astro 6).

Both ignore messages carry the refreshed ledgers and the next levers.

## How it was verified

Every lever was judged on its own real run of the real-input walk binary (no top-ups, no
damage/health changes, no guard-code changes). Key measurements ([L] this branch's runs):

- Pause sweep 5800..7800/60 without the shelter: all red; best 6000 (k1 9354 / k2 9492, death
  10002, 11 astro) — this is why the 6000 stop shipped together with the shelter.
- The shelter without the stop: all 8 latch, death in the missile-wait at 11110 (~62.5 hp in 498
  ticks, 15 astro, one 9.1 world-blast hit).
- The kite missile-wait without the stop: 7.9 -> 0.0 in 377 ticks (no target fits the 7.0 climb
  cap from the deep band, so the kill arm is inert there).
- Shelter + 6000/60 (shipped shape): console phase ends 30.7; missile-wait kite 30.7 -> 23.7;
  waves-wait zero-loss; Drake gate 12721 at 23.7.

Runs on the final tree (whole binary, `--test-threads=1`):

- `walkreal_0700001b_live`: **17 passed; 0 failed; 2 ignored** (123.04s) — the two target walks
  are the ignored ones (their ledgers are in the ignore messages). The Operative walk stays
  green; nothing else in the file moved.
- `walk_arrival_to_drakes_defeat_on_real_health_agent` (individually): reaches Drake's gate at
  12721 / 23.7 hp, fails the Drake-fight assert (:2039), 42.4s.
- `walk_arrival_to_drakes_defeat_on_real_health_00agent` (individually): the ledger's baseline
  signature (stuck 8864 / 37.5), 16.3s.

## What's left

- **Agent green**: the Drake-fight aim/line — the same open lever the d3 ledger names: rest the
  tap's level ray on Drake's box (or a shoot-and-scoot out of the west shelter's cover, whose
  sight-cut also cuts the wave). This round measured: Drake took 0 damage from 96 rounds in the
  compensated fight from the east-platform hold, and the hide-hold probe closed to the hub floor
  (32.0,-1.3,0.0) with the same zero. The hp budget is 20-24 at the gate with the wave down to
  5-12 by mid-fight — once the taps land, the fight is winnable on paper (6 laser taps = 120).
- **00 Agent green**: the next untried lever in its ignore message — a 60-tick stop sweep in the
  mid phase (6000..7500); the Agent sweep this round moved every downstream latch (k6 6822 ->
  7104, window entry 56.9 -> 74.0) and the hard stream has never been re-rolled.

## What was run, and what was not

- Ran: the whole `walkreal_0700001b_live` binary on the final tree (17/0/2 above) plus the two
  target tests individually, after every lever; `cargo test --offline --release -p nf-view
  --test walkreal_0700001b_live --no-run` builds nf-game/nf-formats/nf-audio/nf-view clean.
- Did NOT run: the full-suite fast runner (`tools/test/suite.py`). The sandbox reset consumed
  ~35 min on setup (clone, 1 GB ISO, rustup, crates) and the runner needs ~1.5 h; the remaining
  window after the lever work did not fit it. All changes are inside
  `crates/nf-view/tests/walkreal_0700001b_live.rs` (no shared helper, no src/ change), so the
  1b binary is the complete affected surface; the whole binary ran twice on the final tree
  (120.31s pre-ledger-edit, 123.04s final).

## Usage

not visible to me.

## ntfy failed

All three fallbacks timed out from this sandbox (ntfy.sh unreachable):
- `curl -sS -m 20 ... https://ntfy.sh/nightfire-yoda-8k3x` -> curl: (28) Connection timed out after 20002 milliseconds
- python3 urllib.request (timeout=20) -> the wrapping call exceeded its budget (no response in 60 s)
- `wget -q -O- --post-data ...` -> rc 124 (timeout kill)
- curl retry -> curl: (28) Connection timed out after 20002 milliseconds
