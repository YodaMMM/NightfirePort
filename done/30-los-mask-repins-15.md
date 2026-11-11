## Your task: `los-mask-repins-15`, the 07000015 walks green again under the game's guard sight

Branch from origin/master (823ff190b turned on the game's guard sight: per-object masks, the cel walk, the sight rotation, the guard's own cel, the own-cel boundary push, FellOutMap). Read docs/reports/los-mask-repairs.md on origin/s/los-mask-repairs (reports do not go to master). These walks were ignored because the game-correct sight moved their fights: walk_spawn_to_the_omega_sector_exit_agent, walk_spawn_to_the_omega_sector_exit_real_damage_agent. Read each ignore message. Then re-route each walk with real input so it passes, and remove its ignore. Never lower damage or health, no top-ups; helpers gated to 07000015 and the difficulty. Run walkreal_07000015_live (and its sister binaries) and interactables_07000015_live.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/30-los-mask-repins-15`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox3 at 2026-11-11T00:28Z

---
## Report (done, 2026-11-11T02:06Z)


---
## Report from the worker (done, 2026-11-11T02:06Z)

# los-mask-repins-15 — report

**Branch `s/30-los-mask-repins-15`, base origin/master `784ba2d99`.** (The driver header says branch
`s/los-mask-repins-15`; the task body and the queue claim file `claimed/30-los-mask-repins-15.md` say
`s/30-los-mask-repins-15`, and finish.sh uses the queue item `30-los-mask-repins-15`, so that name is used.)

## What's done

**Both named walks are green on the merged master line and both `#[ignore]`s are removed — with no
test-side re-route needed, because the red state the ignores describe no longer exists on this line.**
The one commit on top of master (`7a6ed7588`) touches only
`crates/nf-view/tests/walkreal_07000015_live.rs` (the two ignore attributes removed, their evidence
moved into the doc comments) and `docs/WORKLOG.md`.

- `walk_spawn_to_the_omega_sector_exit_agent` (the Agent top-up walk): the round-3 ledger's
  door[42] crowd (guards #4 and #19 standing 0.1–1.6 u off Bond, the leaf closed again by the
  #447 shot's start, the mine never going off in 3000 ticks) is gone. Bisect on the master line
  pins the change to **`c3e09cb2f` "Death and hurt planes are solid the way the game's are,
  without drawing them"** — the only level-agnostic sim commit in its window: the walk still
  fails at `6e24d0164` [L the 2026-11-11 filtered run] and passes at `c3e09cb2f` [L].
- `walk_spawn_to_the_omega_sector_exit_real_damage_agent` (the Agent real-health walk): the
  ledger's tick-9330 death at (-0.65, 41.40, 25.82) — "red with the flat boundary push too" —
  does not reproduce anywhere on the merged line: the walk already passes at `823ff190b` (the
  round-3 merge, this branch's base) [L the 2026-11-11 filtered run]. The ignore was written on
  the round-3 development tree and outlived its own red — an ignored test is never re-run.
- The d3 `walk_spawn_to_the_omega_sector_exit_real_damage_00agent` ignore is **untouched**: it is
  the `real-health-00agent-15` ledger, another task. It stays `#[ignore]` and, run under
  `--include-ignored` in the whole-binary check below, it failed exactly as its ledger says
  (its ignore message describes the budget overrun), so the ledger is still accurate.

## How it was verified

All runs on the slim ISO, release, `--test-threads=1`, 2026-11-11. The sim's guard-sight RNG is
seeded per sim with a constant (`Rng::new(0x1234_5678)` [M drone.rs `Rng::new`; scene.rs the
drones.rng construction]), so filtered runs and whole-binary runs see the same stream.

| run | result |
|---|---|
| walkreal_07000015_live whole binary, master, twice | 8 passed / 0 failed / 1 ignored (the d3 ledger), byte-identical outcomes |
| walkreal_07000015_live whole binary after the edit (ignores removed) | 8 / 0 / 1 — identical |
| interactables_07000015_live whole binary | 19 passed / 0 failed |
| bisect, filtered pair runs | 823ff190b: Agent FAIL, real-damage PASS · c120d7b49: real-damage PASS · 6e24d0164: Agent FAIL, real-damage PASS · c3e09cb2f: both PASS · master: both PASS |

## What's left

- **The full suite did not fit this sandbox.** `python3 tools/test/suite.py` needs
  `cargo test --no-run` over the four crates plus ~120 test binaries (~600 MB of links plus the
  per-binary codegen temps) on top of the ISO (~1.0 GB) and `work/`; this 9.9 GB disk hit
  `No space left on device` three times even after freeing ~5 GB of stale generations (the
  previous sessions' `target/` carried multiple fingerprint generations). The nf-game/nf-formats/
  nf-audio lib runs also died on ENOSPC. The last suites recorded the same limitation
  (los-mask-repins-16: "suite … if the sandbox allows"). My change is test-file-only — no shared
  crate or helper code changed — so the whole 07000015 binary + interactables are the full
  affected surface, and both ran green. The next session should run the full suite from a roomier
  sandbox.
- Live checks for the main agent (all on 07000015, Agent difficulty):
  1. At the door[42] fence-shot mark after the walk: list the live hostile positions — the ledger
     had #4/#19 at 0.1–1.6 u from Bond at the threshold; expect no guard within ~10 u at the
     mark (access type: read `drone.pos` each tick around the door[42] E press).
  2. The great-hall pit and the south corridor: which guards cross the hurt planes now that they
     are solid (`c3e09cb2f`) — expect guards to route around them; if a guard still stands in a
     plane's box, that would contradict the solidity and explain a residual fight shift.

## Usage

- input tokens: not visible to me
- output tokens: not visible to me
- context used: not visible to me
- cost: not visible to me
