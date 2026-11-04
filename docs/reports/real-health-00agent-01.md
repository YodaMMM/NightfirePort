# real-health-00agent-01 — the 01 00 Agent walk

**Status: DONE.** The 00 Agent (difficulty 3) sister of `walkreal_07000001_live`'s real-damage
walk is landed, un-ignored, and green, with `Drones::difficulty == 3` asserted at load. This
session re-verified everything from a fresh clone after two sandbox resets, ran every
`walkreal_*_live` binary whole, and captured the suite totals that fit in the window.

## What's done

- The sister test
  `walkreal_07000001_live::walk_from_the_spawn_to_the_exit_with_the_contact_real_damage_00agent`
  (crates/nf-view/tests/walkreal_07000001_live.rs, landed in commit 0c0de2e9) walks the whole
  route at difficulty 3 on real input from the level start, no top-ups. The load asserts
  `drones.difficulty == 3` ("the sim runs the picked difficulty [M GameState+0x28 0x2a3790 →
  Drones::difficulty]"). Final run: exit to 07000002 at tick 10428, end health 22.6, 26 hits
  for 77.4, 9 return-fire fights.
- The tactical piece that made d3 survivable is the lip law: the hall holds a fourth guard
  outside the named loops' AND (the d3 run's #5, skin 0x5000038, shotgun, 24 hp, 0.9 u off the
  aperture's north side), and the doorway step ate his volley point-blank (the 4119 death:
  -14.40 x2 at 0.9 u, the x2 TooClose band [M Drone_ModBulletDamage 0x139f50]). The test's
  d3-gated lip block lures him across the aperture (the guards hunt the gunshot [M
  DroneFunc_SetNoiseHeard]) and drops him from the hold line before the step.
- The stairwell pop-pocket work this branch reused is level-01-agent-stairwell-doors' (merged
  on this branch as 9970721e): cross the ch-0xF slab deliberately, back out in the poppers'
  10-tick reaction window [M DroneFunc_ReactionTime 0x14ac60:15-24], fight from the mouth at
  6.5+ u, never nav-approach a guard parked west of x -27. No changes to `tests/common/` were
  needed by this task (helpers untouched, so nothing to opt in).

## How it was verified (this session, fresh clone + full rebuild)

- `walkreal_07000001_live`: **8 passed / 0 failed / 0 ignored** in 74 s on this tree — the d3
  sister included. This is the task's own binary.
- `difficulty3_07000001_live` (the pins the brief points at): **4 passed / 0 failed** — the
  spawn set, the difficulty rows, and the no-delta-at-difficulties pins all hold with the
  sister in place.
- **Every `walkreal_*_live` binary in the workspace ran whole** (78 binaries):
  **218 passed / 0 failed / 10 ignored** (the 10 ignores are pre-existing on master). The
  long-level binaries time-starved at 3-way concurrency were re-run solo and green:
  walkreal_0700000a_live 12/0, 0700000d_live 1/0, 07000011_live 3/0, 07000012_live 3/0,
  07000016_live 14/0, 07000041_live 11/0. Full per-binary table in
  `walkreal_results.tsv` (session scratch, summarised here).
- Suite totals captured before the window closed:
  - `nf-formats` + `nf-audio`: **229 passed / 4 failed / 3 ignored**. The 4 failures are all
    disc-image dependent: the 3 `disc_golden` tests (the documented slim-ISO allowance) plus
    `identity_disc::the_real_disc_passes_the_identity_check`, which expects the full
    1,191,072-sector NTSC-U image while the onfoot ISO here has 518,435 sectors — an
    image-size environment failure, not a tree failure.
  - `nf-view`: partial — 48 test binaries completed with **0 failed** before the window
    closed (alphabetical, 0*–front_end_chain), including `walkreal_07000001_live` and
    `difficulty3_07000001_live`.
  - `nf-game`: not reached. The box is 2 cores and the session restarted twice (fresh clone,
    ISO re-download, full rebuild each time); lesson 10's "the full suite did not fit"
    applies. The task's diff is a test-file addition only — nothing under `crates/*/src/`
    changed — so no non-walkreal binary can be affected by it; the binaries that DO exercise
    the level (the two above) ran green whole.

## What's left

- Nothing blocking. If a later session wants the missing totals: run the nf-view remainder
  (~900 short binaries, alphabetical after `front_end_chain`) and `nf-game` on a wider box.

## Live checks for the main agent

None required — the change is test-side only (no `crates/*/src/` edit, no visual change).
The sister's route is already covered by the existing walker renders on this branch.

## ntfy failed

(none yet — filled below if all three notification attempts fail)

## Usage

not visible to me (no token, credit or context-window figures are exposed in this sandbox).
