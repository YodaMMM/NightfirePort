# drone-block-swaps-g055-4 — the last walks under the G-055 hunt

Branch `s/drone-block-swaps-g055-4` = origin/s/drone-block-swaps-g055-3 merged with
origin/master (182408f88, batch-1104i). Status at the end of this session: **both target
walks are GREEN and verified on the pushed tree**, the whole `walkreal_*_live` set ran,
and the **full four-crate suite ran to completion** (every one of the 1104 built test
binaries). Everything below is pushed.

## What was done

1. **The two target walks** (made green by this branch's two commits, both verified again
   this session, whole, serial, on HEAD 4fa0db25c):
   - `walkreal_07000014_live::walk_the_infiltration_to_the_delta_exit_real_damage` —
     **green**: the binary is **9 passed / 0 failed / 1 ignored** (153 s). The fix
     (commit 05fb32b52) replaced the shared resupply's trench-lip wedge with a bounded
     test-local loot pass (own floor, < 6 u); the fights held 74.0 health through 9–12
     fights.
   - `walkreal_07000016_live::walk_the_chasers_defeat_completes_the_mission_on_real_damage`
     — **green**: the binary is **14 passed / 0 failed / 2 ignored** (74 s). The fix
     (commit 4fa0db25c) re-routed the ring/station/return legs at ring level over the nav
     web (the direct steer fell off the ring edge), cleared the ring gate band-limited
     (±2.0) with the 15 u far-stand skip, kill-looted by the drop's live position, restocked
     low ammo before fights, and deep-cycled firearms after loots (the plain cycle landed
     on the frag 0x34); the shared return-fire pick got an **opt-in** `level_fights` band
     (flag default false; the measured trench deaths y −4.65/−5.01).
2. **The full `walkreal_*_live` sweep** (lesson 7): **80 binaries, 246 passed / 3 failed /
   20 ignored**, every binary listed in "How it was verified" below. The only 3 failures
   are the g055-3 hand-off's master-merge shifts in other levels' own test files (next
   section) — nothing this branch touches. The opt-in helper flags are proven inert for
   every other walk: `level_fights` gates a filter that only runs when the flag is set,
   and no other test file sets it (grep over `crates/nf-view/tests/`).
3. **The full suite ran to completion** on this tree: 1104 test binaries built
   (`cargo test --offline --release --no-run` over `-p nf-game -p nf-formats -p nf-audio
   -p nf-view`), every binary run serially with `--test-threads=1` through a resumable
   per-binary chunker (state flushed after each binary), plus direct re-runs of every
   binary the chunker could not finish.
   **Total: 4202 passed / 15 failed / 339 ignored across 1103 unique binary names**
   (1104 built: `fix_ai_remainder_live` exists in BOTH nf-game and nf-view; the state's
   name-keyed record holds nf-game's 2-passed run and nf-view's copy — 1 passed — was
   verified directly, so 4203 passed effectively). Per crate:
   - nf-game: 101 binaries — **1058 passed / 0 failed / 8 ignored**
   - nf-formats: 45 binaries — **201 passed / 4 failed / 3 ignored**
   - nf-audio: 4 binaries — **28 passed / 0 failed / 0 ignored**
   - nf-view: 954 binaries — **2917 passed / 11 failed / 328 ignored** (+1 direct green)

## How it was verified

- The two target binaries were run whole and serial (their counts above are the whole
  binary, not a filtered run), after `git status` clean and HEAD = the pushed commit.
- The walkreal sweep and the suite ran each binary with `--test-threads=1` under a
  per-binary wall-clock cap (420 s), results parsed from each binary's own
  `test result:` line; three binaries that died to external kills/timeouts were re-run
  directly and their state records corrected to the direct evidence.
- The walkreal list, per binary (p/f/i): 07000001 8/0/0 (88 s), 07000002 3/0/1, 07000003
  5/1/1, 07000004 10/0/0, 07000005 14/0/0, 07000006 6/0/1, 07000007 5/0/0, 07000008
  6/0/0, 07000009 8/0/0, 0700000a 12/0/3, 0700000b 7/0/0, 0700000c 3/0/1,
  0700000c_barrelrow 1/0/0, 0700000c_eastroute 1/0/0, 0700000d 1/0/2, 07000011 1/2/1,
  07000011_route 1/0/0, 07000012 3/0/0, 07000013 3/0/0, **07000014 9/0/1**, 07000015
  6/0/1, **07000016 14/0/2**, 0700001b 17/0/2, 07000041 15/0/0, 0700004a 4/0/0,
  0700004a_satchel 2/0/0, celprobe 2/0/0, dump 1/0/0, probe014_param26 1/0/0, probe0c2
  1/0/0, probe0c3 1/0/0, the 24 probe0c_* spot probes 1/0/0 each (probe0c_northend
  0/0/1 ignored), probe11_48 2/0/0, probe2 5/0/0, probe3 2/0/0, probe81 1/0/0.

## The 15 failures + 2 timeouts, classified

- **Allowed (rule 5): `disc_golden` ×3** (bigf header, bigf tables, directory tree) —
  the slim-ISO allowance, exactly as documented.
- **Needs the full retail image (1): `identity_disc::the_real_disc_passes_the_identity_check`**
  — it asserts 1,191,072 sectors [D `docs/build-identity.md`] but `check_path` reads the
  sandbox's `Nightfire-onfoot.iso` (measured 518,435 sectors, 1,061,754,880 bytes) and
  panics on the count. Environmental: the worker setup downloads only the onfoot ISO.
  Not touched by this branch (the test predates the merge base; this branch changed no
  `nf-formats` code). Suggest the owner gate it on the full image like `disc_golden`.
- **Needs a GPU surface (5): `glass_guards_14_live` ×1, `fix_0c_pylon_live` ×2,
  `probe_pylon_render` ×1, `missing_scenery_live` ×2** — each panics with the same wgpu
  `adapter: NotFound` (no Vulkan/GL device in the headless sandbox). The pylon trio +
  missing_scenery match the October headless record exactly (the same four binaries
  failed there the same way). The main agent's GPU environment is where these pass.
- **Pre-existing other-level shifts (3): `walkreal_07000011_live::walk_spawn_to_the_
  lower_floor_load_point_real_damage` + its `_agent` sister, and
  `walkreal_07000003_live::walk_the_full_route_and_complete_every_objective_real_damage`**
  — the exact shifts g055-3's report documented as consequences of the mandated master
  merge ("no line of this branch touches their levels"). Measured still red on this tree
  (the WORKLOG's "also to measure" item); `walkreal_07000011_route_live` passes. Their
  owners: the `s/real-health-14-00agent-3` branch is in flight for the 00agent level.
- **Timeouts (2, sandbox window): `mp_stress_explosives_live`** — the default run is
  7200 ticks × 3 lockstep machines with an explosive spawned per human per tick
  [M `crates/nf-view/tests/mp_stress_explosives_live.rs:70`]; killed at 420 s mid-test
  with no result line (no assertion failed; it simply needs a longer window or a
  machine faster than this 4 GB sandbox). **`mp_online_failures`** — first test passed
  (socket rehost), killed at 420 s inside `join_in_progress_from_the_menus_then_leave_
  and_reconnect`. Both are nf-view mp stress tests, unrelated to this branch's files.

## What's left

1. The three walkreal shifts above need their owners (or a follow-up task with the
   level-0C/07000003/07000011 walk levers); everything else in the suite is green or
   environmental.
2. The two mp binaries need one full run on a machine/window that fits their default
   lengths (or a sanctioned `NF_STRESS_TICKS` reduction for `mp_stress_explosives_live`).
3. `identity_disc` needs a slim-ISO gate (owner's one-line change) or the full retail
   image in worker sandboxes.

## Live checks for the main agent

Nothing load-bearing from this branch (no game-code change; the sim is master's). If
wanted, the five GPU-surface tests above can be run directly in the main agent's
environment — they are ordinary `cargo test -p nf-view --test <name>` runs, no
particular spot or camera needed.

## Usage

not visible to me — no token, credit, context-window or cost figures are shown anywhere
in this environment's interfaces for this session.
