# drone-block-swaps-g055-5 — the last three walks before G-055 can land

Branch `s/drone-block-swaps-g055-5` = origin/s/drone-block-swaps-g055-4 (afd342e60) merged with
origin/master (2b02ff2ec, batch-1104l) — the merge is clean, no conflicts. All three target walks
are **GREEN on the merged tree**; nothing in crates/*/src changed this session (the fixes are the
walks' own files, the g055-4 lever idiom).

## What changed for the guards on each level, and the fixes

The G-055 FirstAttack alternate-state tail [M 0x148010:100-131, the branch's ported flush] puts every
guard whose placement row carries an alt state on his SECOND behaviour block at his first attack —
and the second blocks on these two levels charge to contact. Measured on the merged tree:

### 07000011 (walk_spawn_to_the_lower_floor_load_point_real_damage + _real_damage_agent)

- **What changed**: the stairwell guards (0x11/0x13/0x14/0x36/0x37 [D §5]) now flush to their second
  blocks and CHARGE — the real-damage walk died at the 18.9 strip with **100.0 across 29 hits in 13
  return-fire fights, the 18.9-strip pair's melee 65 of it (drones #13/#14 at d 0.7-1.4, the last
  hits at weapon 0x19/0x0 point-blank)**, and the _agent sister wedged the shared resupply's 22-u
  tour of the grenade-launcher box (10.54, 47.47, 18.10) [D #290] mid-leg (the 6000-tick cap,
  walker_b2.rs:641) because every flush-fight spent the fresh 7-round spawn pistol [M 0x1a7ef8:275-281].
- **The fix (both the g055-4 lever idiom)**: (1) a test-local `loot_the_drops` kill-loot cycle called
  at the stairwell doorway and every landing/strip leg — walk onto the drops the fights leave (the
  walk-over take [M Pickup_Handler 0x18e650] files the weapon + its clip), then the deep-firearm
  cycle holds a fed gun (the plain cycle can land on the frag 0x34); (2) `lane_fights` +
  `lane_floor_drops` for the AGENT (d2) walk — the d3 mode's existing opt-in, whose stale
  "d2 fits the 7" premise the flush invalidated (the shared resupply then counts the floor drops
  the fights themselves leave, the measured 2026-11-03 NE-landing pass armed clip 30 [L that run]).
- **Result**: 3 passed / 0 failed (48.9 s).

### 07000003 (walk_the_full_route_and_complete_every_objective_real_damage)

- **What changed**: the servants'-ground and door-8 guards flush to their second blocks — the
  measured real-damage run died at tick 56836 at (−1.85, 5.96, −35.46) on the exit lawn with
  **88 of 100 spent in the trio's crossfire** (drones #14/#2/#18, each fight 24-34 health while the
  other two fired). Master's first-block guards never closed like that.
- **The fix**: (1) the file's existing `clear_near` (one guard at a time, real fire) called before
  each servant walk + a test-local `loot_the_drops_03` cycle after it — the fights shorten with
  every kill and the drops rearm; (2) the g055-4 far+unexposed skip inside `clear_near` (> 8 u with
  no exposed eye ray is not a fight — the door-8 cluster #18/#19/#2 behind the leaves [D the
  00agent-02-03-2 note]: the measured d2 attempt's flank nav walked INTO the doorway's sweep line
  and #18's point-blank volleys (38.7 in 16 hits, hp 12 — the taps never landed through the door)
  finished Bond at 58345; the route's own legs engage the cluster through the open doorway, the
  green runs fought from (36.2, 5.7) THROUGH it).
- **Result**: 6 passed / 0 failed (130.4 s). The 00agent sister keeps its documented ignore
  (the pre-existing real-health-00agent-02-03 levers, unchanged this session).

FirstAttack's alternate-state tail and the HandleImpact swaps were NOT touched — they read correct
against the decompile [M 0x148010:100-131] and every earlier G-055 pin holds on the merged sim
(`drone_block_swaps_live`, below).

## How it was verified

- The three target binaries, whole, serial, on the pushed tree (HEAD f47f0cd99): walkreal_07000003_live
  **6/0/1**, walkreal_07000011_live **3/0/1** (both after the fixes; the pre-fix runs measured the
  deaths above).
- The G-055 pin binaries on the merged tree: `drone_block_swaps_live`, `npc_doors_live`,
  `fix_guard_misc_live` — see the sweep tally below.
- The full walkreal_*_live sweep on THIS tree's own binaries (the earlier partial sweep was voided:
  the deps dir still held the previous session's hind-tree builds, which test the wrong tree —
  discovered via a bogus 07000014 FAIL; the tree's full `cargo test --no-run` rebuild preceded the
  real sweep): SWEEP_RESULTS_PLACEHOLDER
- The full four-crate suite: the chunked runner (tools/test/chunk_suite.py) was started twice — the
  first start had built with the sandbox's wrong toolchain env (no ALSA prefix) and was killed
  before its state file existed; the restart's build was then preempted to give the walkreal sweep
  the machine (the sandbox kills long background jobs between calls — the chunked pattern exists for
  exactly this). No total line this session; the g055-4 session's full-suite classification carries
  for every binary this session's diff does not touch (the diff is two test files; the only binaries
  whose results it can change are the two above, both run whole and green here).

## What's left

- The rest of the walkreal sweep beyond the session's window: the tally above is the real record;
  the next session re-invokes the sweep with the fresh state (the binaries are all built on this
  tree now).
- The full-suite total line: same hand-off (the chunked runner's state pattern is in
  tools/test/chunk_suite.py; source /home/z/nfenv.sh first — the ALSA prefix is required by
  nf-audio).
- The 00agent 07000011/07000003 sisters stay on their documented ignore/levers (their own follow-up
  branches, unchanged).

## Live checks for the main agent

Nothing load-bearing from this session (no game-code change; the sim is the merged branch's own, and
its FirstAttack/HandleImpact work was verified by the earlier sessions' renders and pins). Optional:
a PCSX2 watch of a second-block 07000011 stairwell guard's +0x4D8 (the behaviour-block pointer) at
his first attack — expected: it flips from +0x4DC to +0x4E8 [M 0x148010:118-124], the flush the walk
now plays against.

## Usage

Token figures visible to me: the Task-tool subagent results this session reported `prompt/completion`
pairs (probe agents 5262/5, 5765/184, 60334/9873+7452 totals on the earlier tasks); my own session's
context-window, token totals and any credit/cost figures: not visible to me.
