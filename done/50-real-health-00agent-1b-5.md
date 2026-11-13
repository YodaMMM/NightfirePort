## Your task: `real-health-00agent-1b-5`, 0700001b's console phase can be survived on 00 Agent with real health

Branch from origin/s/real-health-00agent-16-1b-4, merge current origin/master, and read its docs/reports/real-health-00agent-16-1b-4.md, item 1 of "What's left". First re-derive the real branch-read cadence from the decode it found, and fix the determinism divergence it names: test it red on the old code, and keep the Rand count. Then re-balance the per-console caps against the real read times, keeping the lane kill shape at the 300 cycle with the fights fenced out of the hold chains. Then the Drake phase's spawn-band re-weight. The 16 half of that report is not this task. Helpers stay gated to 0700001b and difficulty 3, so other walks stay byte-identical. Real input only, no top-ups, never lower damage or health. Remove the ignore when green; if it is still red, push the ledger and next lever in its ignore message. Run the whole walkreal_0700001b_live binary, interactables_0700001b_live and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/real-health-00agent-1b-5`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
Main agent, fresh deaths on master 729f9694e (each ignored walk of the level, --ignored --test-threads=1; test names not captured):
walkreal_0700001b_live: Bond died at tick 11385 (frame 11385) at (46.01,-14.08,-3.00) armour 0.0
walkreal_0700001b_live: Bond died at tick 5619 (frame 5619) at (31.92,-1.68,-0.59) armour 0.0

---
claimed-by: sandbox4 at 2026-11-13T04:33Z

---
## Report (done, 2026-11-13T06:04Z)

Closed by the driver: the report is on `s/real-health-00agent-1b-5` (docs/reports/real-health-00agent-1b-5.md); the worker didn't run finish.sh.

---
## Report from the worker (done, 2026-11-13T06:10Z)


---
## Report from the worker (done, 2026-11-13T06:10Z)

# real-health-00agent-1b-5 — report (2026-11-13)

Branch `s/real-health-00agent-1b-5`, cut from `origin/s/real-health-00agent-16-1b-4`, merged
current `origin/master` (abc09337d) clean. **Status: the console phase is still red on 00 Agent
real health, but this round answered the -4 round's decode find and the determinism item, put
the -4 lever set in with the hold-chain fence, and left the ledger + the next levers in the
ignore message.** The walk stays `#[ignore]`d with the round-5 ledger.

## What's done

**1. The real branch-read cadence, re-derived and pinned (probe1b_cadence, GREEN).**

- `crates/nf-view/tests/probe1b_cadence.rs` (new, this task's own file — not a shared helper):
  two full level loads in ONE process, 7000 ticks each, the launch order driven through the real
  AND #202 inputs (ch 193/194 held — the mux writes ch 19 every tick [M triggers.rs case 3]),
  records per-tick `(state, clock)` for missile 5 and every missile's start.
- **The p0 clock advances EXACTLY +1.0 per sim tick while state 1 stands** [M 0x1aa23c
  `SpaceMissile_Update` runs `Script_Update` per tick; L the probe's two identical traces].
- **The start law [L]**: seed 0 exits on the first gated tick (ch19 + 1); every other seed exits
  within ±1 tick of ch19 + seed·60 — the exact tick depends on the f32 countdown accumulation
  (seed 105 → +6300, seed 40 → +2401; COUNTDOWN_STEP 1/60 [M 0x1aa1f0-208]).
- **The -4 "1/4 rate" was a stale-anchor artifact, not a throttle.** The probe shows no
  1/4 clock anywhere in the port's missile path; the modeled read deadline
  S_k + P0_LEN[k−1] − 1 [M 0x1aa23c-258; D P0_LEN] stands and `read_deadline_in` keeps it.

**2. The determinism divergence, explained (non-repro in-process).**

- The two loads' ch19 frame, all eight missile starts and m5's whole clock trace are
  **byte-identical** [L the probe]. std HashMap iteration order (per-instance RandomState)
  cannot be the leak — each load builds fresh maps.
- **The -4 run A vs run B "divergence" (m3 4175 vs 4375) was an observation-window artifact**:
  the walk's per-tick deploy/clock sample block is bypassed while a defense fight ticks
  (`fight_opts_y` runs its own `w.tick` loop), so a missile start that lands INSIDE a fight is
  recorded only at the fight's end. This round's lever run reproduces it exactly: m3 recorded
  **4375** while the defense fight 3457..4374 covered the model-consistent ~4175 [L the run;
  ch 19 never dropped — this run logged every AND(193,194) falling edge with a per-tick
  detector, zero drops]. No sim code changed; the sim never diverged.

**3. The -4 lever set re-implemented with the hold-chain fence (the walk file only).**

- The mid-phase defense (00 Agent): cooldown 1200 → **300** (the -4 measured edge), fight cap =
  the pending console's `read_deadline_in` − 400 clamped 300..1400, the **hold-chain fence**
  (`answered[pi] == 0` — never start a defense fight once the pending console's UI chain has
  begun; the console5 k5 lesson, today's slack gate made exact), the **off-plane lane kill
  shape** (the hold at |z| 1.6 on the nearest shooter's side — the -4 run A shape). All inside
  `walkreal_0700001b_live.rs`, gated by the file's own `hard` flag — no shared helper touched,
  every other walk byte-identical.

**4. The lever run's ledger [L this run]**: death **6580** at (30.93,−6.19,−1.71) — 33 s past
master's 5619 — but **m3 took the LAUNCH arm** (deploy [0,0,3,2,2,1,0,1], mission status 2):
the fights ran their full caps WITHOUT thinning (astro 5 after both: 3457..4374 cap 917,
4716..5574 cap 858 — the -4 run A thinned 7 → 3), so the schedule slipped — k5 3158 (−416, as
baseline), k4 5574 (**+800** past 4774), k3's holds answered 5828/5936/6044, all past m3's
~5975 read → the channel write landed 6565 (+391) after the read → m3 launched.

## What's left

1. **The cap-math flaw is the next lever**: cap = deadline − 400 grants the fight the chain's
   own time — the fence blocks fights DURING a chain but not fights that consume the slack a
   chain still needs. Chain time measured ~900 ticks (k4: approach end 4374 → latch 5574). Next:
   cap = deadline − CHAIN_RESERVE (~1000), and no fight once Bond is within ~4 u of the pending
   station (the approach is the real cost — the -4 run A fought BETWEEN consoles, this run
   fought ON the approach).
2. **Make the fights thin like run A**: this run's |z| 1.6 lane held behind the panel rows where
   the wave posts (z ±2.7-4.3) out-range Bond; try the lane on the wave's approach side and prio
   the nearest CROSSED shooter.
3. **The Drake phase's y > 9 spawn-band re-weight** — after the console phase survives (the -4
   item; untouched this window).
4. The live checks carried from -4/-2 stand (the k6/k8 hub sight-lines render, the k1/k2
   above-deck answer, the 16 gallery foot access, the missile p0 clock over 60 frames — the
   probe answers the port side 1:1; the PCSX2 confirm is still worth one look).

## How it was verified

- `interactables_0700001b_live`: **9 passed / 0 failed** [L this run].
- `walkreal_0700001b_live` whole binary with `--include-ignored --test-threads=1`: **17 passed
  / 2 failed** in 311.6 s [L this run]. The two failures are the two `#[ignore]`d real-health
  walks: `..._00agent` (this task's ledgered walk — death 6580, the lever ledger above) and
  `..._agent` (the d2 sister — death tick 11385 at (46.01,−14.08,−3.00), **byte-identical to
  the driver's fresh master-baseline death**, so this round's edits moved nothing on that
  walk). The d1 top-up-walk tactics red passed on this tree.
- `probe1b_cadence`: **1 passed / 0 failed** [L this run].
- `nf-game` whole crate (`cargo test --offline --release -p nf-game -- --test-threads=1`):
  **1233 passed / 0 failed** across its 117 test binaries [L this run].
- No `crates/*/src/` files changed — the branch's code diff is the walk test file, the new
  probe file and the two docs; no shared-helper sweep required.
- **The full 4-crate suite did NOT fit this sandbox's disk** (9.9 G total: the suite's
  ~1110 nf-view test binaries + nf-game's 117 + the ISO + the registry left the first build
  attempt at 100 % full — two build attempts died on `ld terminated with signal 7` mid-link
  before `cargo clean` freed 6.1 G). The required binaries ran directly; the suite runner's
  all-binaries build cannot fit this disk. A dedicated suite pass needs a bigger sandbox.

## Usage

- Context window, token and credit figures for this session: not visible to me (no such
  counters are exposed in this environment; nothing to copy).
