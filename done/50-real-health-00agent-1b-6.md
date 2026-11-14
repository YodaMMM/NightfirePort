## Your task: `real-health-00agent-1b-6`, 0700001b's 00 Agent real-health walk defeats Drake

Branch from origin/master. walkreal_0700001b_live::walk_arrival_to_drakes_defeat_on_real_health_00agent is ignored; the last round pinned the real branch-read cadence (probe1b_cadence) and fixed the walk's deploy/clock sampling during defense fights; the console phase is still red - the ledger and next levers are in the ignore message. Run walkreal_0700001b_live, probe1b_cadence and interactables_0700001b_live. Real input only, no top-ups, never lower damage or health; every lever gated to the level, its difficulty and !w.tops_up(). Remove the ignore when green, otherwise push the ledger and the next lever into it.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/50-real-health-agent-06`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

**Branch:** `s/real-health-00agent-1b-6`.

---
claimed-by: sandbox7 at 2026-11-13T12:14Z

11-13 (on master cdfc28056): round-5 lever 1 landed - cap = deadline - 800: m3/m4/m5 all disarmed, death t6580 -> t7494 (the wave 6-7 astronauts + one 17.64 world blast at the fixed 6580 tick). Reserve 1000: t7402 (disarmed); 1200: t6505 (starved). Next: thin the wave (lever 2) and the kind-107 blast compare.

Cooldown re-swept at reserve 800: 200 -> t6963, 300 -> t7494 (best, master), 400 -> t7138, 600 -> t6580; m3/m4/m5 stay disarmed in all. The cycle stays 300.

---
## Report (done, 2026-11-14T03:03Z)


---
## Report from the worker (done, 2026-11-14T03:03Z)

# real-health-00agent-1b-6 — round report

Branch `s/real-health-00agent-1b-6`, created from `origin/master` 48e457c57, merged with
`origin/master` c9b24271d (final tree merge commit 099e0de66). The 00 Agent real-health walk
`walkreal_0700001b_live::walk_arrival_to_drakes_defeat_on_real_health_00agent` is still red — the
console phase survives to the k6/k8 windows but Bond's health runs out there — and the walk's
ignore message now carries this round's ledger and the next levers. The wave and the health budget
are solved to the measure below; the remaining binder is the k6 chain's start timing.

## What's done

The round-5 ledger's levers were implemented and measured one at a time (16 full walk iterations,
each ~12 s wall, logs `/tmp/lever*.log`, master baseline reproduced first — it matched the prior
ledger exactly: ch19 1775, k5 3158 / k4 5174 / k3 6177 latched, m3/m4/m5 disarmed, five defense
fights ~4000 ticks with zero kills, death t7494 astro 6).

1. **The station-yield gate (round-5 lever 1) is WRONG — measured.** Suppressing the fights once
   Bond is within 4 u of the pending station removed every defense after the first crossing (the
   baseline's fights 2-5 all start near x32) and the walk died at 5649, 1845 ticks before the
   baseline. The station fights are the survival mechanism: `fight_opts_y`'s exposure-hedged cover
   floats break the shooters' 2-s sight clocks (the baseline's fight 1 ended after 53 ticks on a
   sight break). The lever survives only as the fight's own shape (below).
2. **The run A kill shape (round-5 lever 2) works once the fight can REACH the shooters.** The
   approach lane — 40% of the way from Bond toward the nearest shooter, |z| >= 1.6 on the
   shooter's side — plus the nearest-shooter fire prio (`fight_opts_y`'s r−50 / r−60 banding)
   plus a 2-kill exit thinned 7 → 4 with two kills a fight in 286-388 ticks (iters 3-4); one
   iteration latched k6 for the first time (iter 10: k6 7519, −255 vs the model, death 7631).
3. **Measured shape laws** (each backed by a failed iteration):
   - `max_y` 12 is BAND-UNSAFE (iter 9): the k4 screen stance had already lifted Bond to feet
     7.375 (capsule top 9.175 [M Player_DealWithObjHit 0x194458: the feet..+1.8 r 0.55 capsule]),
     and the fight's cover ring (±2.5 in z) walked him under the live BigLaserBeamHURT #259 band
     (y 9.0-9.9, z −0.4..0.4 [L the level's hurt dump in the log]) — two −50.0 deliveries, hp
     100 → 0.0 in a 14-tick fight. The band-safe body cap is `max_y` 7.0: feet 7.1, top 8.9 < 9.0,
     reachable aims ~8.4-8.8 — exactly the prior ledger's "wait the band shooters down to the
     <=8.3 aim heights" band. The laser's 20.0 [D weapon_data 0x33 +0x0c] one-shots every
     astronaut (health 20/15/12/10).
   - The defense cooldown must count SIM FRAMES, not loop iterations (iter 8): `fight_opts_y`
     burns its whole window inside ONE loop iteration, so the iteration counter read ~0 right
     after a fight and the defense slept ~300 real ticks while the station grind drained 13.9 hp
     in 52 ticks. Now frames-based.
   - The chain fence admits fights inside a STARTED chain on the missile's own LIVE clock:
     `read_deadline_in` [M 0x1aa23c-258; D P0_LEN] — the measured live k4 read sits ~1200 ticks
     past the modeled S_k + P0_LEN − 1 (the p0 starts ~1200 after the MissileDeploy 0→1), so the
     modeled margins print late while the disarm still holds (k4 printed +297/+400/+423 in iters
     7/12/16 and deploy[3] stayed 2 every time — the live clock is the game's truth).
   - The pre-chain fights stay gated at slack 1200 (iter 7's 900 gate CASCADES: the extra fight at
     3734 shifted the route, the k4 fight then collided with the f4200 spawn wave, −45.0 hp in one
     window, death 6784).
   - The lane must stay PINNED to the pending console's station ±5 u (iter 11: the raw approach
     lane is a chase trap — the fights dragged Bond to x39.4/z 10.8, the chains starved, m6/m8
     took the LAUNCH arm, deploy [1,1,2,2,2,3,1,3], mission status 2).
4. **Final shape (iter 16, the pushed tree)**: pinned lane, slack 1200 pre-chain, live-clock fence
   >900 in-chain, fight cap 300, frames cooldown, melee override <12 u, 2-kill exit, max_y 7.0.
   Every lever sits inside the `hard` (difficulty 3) branch of `shoot_consoles` in
   `crates/nf-view/tests/walkreal_0700001b_live.rs` and is gated on `!w.tops_up()`; the
   difficulty-1 and Agent-difficulty-2 walks keep the master path byte-for-byte.

**Result [L iter 16]**: k5 −18 / k4 −457 (green by the model margin) / k3 +398 (live-saved),
m3/m4/m5 ALL DISARMED (deploy [0,0,2,2,2,1,0,1]), the wave thinned 8 → 3, hp 57.7 at t7014 —
death t7442 (−52.7 inside the last 128-tick fight). The 6580 kind-107 blast (17.64) landed inside
the measured runs as before; the schedule stays fixed.

## What's left (the next levers, in the ignore message)

1. The melee override is a FIGHT LOOP: with the wave parked within 12 u the fights re-fire every
   300 ticks with no 2-kill progress (astro 3 → 3 → 3, no k6 hold EVER answered in iters 12-16)
   and the k6 chain starves. Fix: a no-progress exit (a fight that lands no kill inside its window
   ends at ~150) and the k6 screen resumption.
2. The thinned wave's survivors snipe from 12-25 u with ESTABLISHED sight (the melee trigger never
   fired; −41.8 hp in 362 ticks from two shooters, iter 13). The fight trigger needs a
   closing-threat band, not `lined >= 2` alone.
3. The Drake phase's y > 9 spawn-band re-weight, after the console phase survives.

## How it was verified

- Master baseline reproduced exactly before any change (the ledger match above).
- 16 full-walk iterations, each measured against the previous (the iteration log line now prints
  the fight window, cap, chain_clear, astro before/after, hp before/after and the lane).
- The three required binaries ran whole on the final tree (pass counts below).
- The full suite did not fit (disk wall, below); the three pushed crates ran whole and the three required nf-view binaries ran whole on the final tree merged with origin/master.

## Binaries run (final tree, merged with origin/master c9b24271d)

- `walkreal_0700001b_live` (whole): **18 passed, 0 failed, 1 ignored** (216 s) — the difficulty-1
  walk and the Agent (difficulty 2) walk pass on this tree; the 00 Agent walk stays ignored (still
  red).
- `probe1b_cadence` (whole): **1 passed, 0 failed** (13 s) — the cadence/determinism pin holds.
- `interactables_0700001b_live` (whole): **9 passed, 0 failed** (39 s).
- `nf-game` + `nf-formats` + `nf-audio` full test suites (rule-5 line, `--test-threads=1`):
  **144 passed, 3 failed, 0 ignored** — the 3 failures are exactly the 3 `disc_golden` tests
  (`bigf_header_fields_behave_as_documented`, `bigf_tables_match_the_golden_manifest`,
  `directory_tree_matches_the_golden_manifest`), which are the allowed slim-ISO failures.
- **The full four-crate suite run did not fit**: the sandbox disk filled at 100% during the
  suite's build phase (`os error 28`); clearing `target/release/build/nf-view` (5.3G) freed the
  space but left no window to rebuild and run the remaining ~30 nf-view test binaries before the
  deadline. No `crates/*/src/` code was changed this round (test file only), and the three crates
  whose artifacts survived ran whole as above.

## Suite total line

The suite.py process was killed by the disk-full failure before its total line; the per-part
totals above are the run's result (144+1+18+9 = 172 passed, 3 failed = the 3 allowed disc_golden,
1 ignored = the 00 Agent walk under repair).

## Live checks for the main agent

- The kind-107 blast source compare from the prior ledger still stands: in PCSX2 at 00 Agent,
  shoot console 1's screen from the station and watch the health trace for a 20.0-class blast at
  the screen (the fixed 4658→6580→8502→10424 schedule, 1922 ticks apart). If the game's player
  takes it too, the route must dodge the fire ticks; if not, the port's blast delivery is a port
  gap. Address pointers: the band's hurt record #259 (0xe4) and the panel resume debris chain
  (GAPS G-047).

## Usage

not visible to me
