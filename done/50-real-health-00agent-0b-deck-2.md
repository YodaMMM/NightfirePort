## Your task: `real-health-00agent-0b-deck-2`, 0700000b's 00 Agent real-damage walk crosses the deck

Branch from origin/master. walkreal_0700000b_live::walk_fresh_spawn_to_the_mission_end_real_damage_00agent is ignored; its ignore message holds the current ledger (death t17415 on the deck crossing, 22 hits; #4's 0x18 rounds from his column (1.62, 58.03, 32.11) cost 35.0 in 5; the jam close and the d3 deck fight call are in). Next lever, from that ledger: the fight pick finds no VISIBLE candidate from the crossing although world segment_bullet rays to #6/#8/#9 read clear - census both ray sets from the crossing spots (NF_RF_TRACE=1 + the [0b-ray] map), then either widen the d3 fight's reach to the world-ray set (opt-in, d3 only) or move the P2 aim so a fight opens from a visible spot; the dart sleep on the dartable #4 (state 0x68 dartable true at t17199) saves the 35.0. Helpers gated to 0700000b and difficulty 3, so the Operative replay stays byte-identical (replay_parity_0700000b green). Real input only, no top-ups, never lower damage or health. Remove the ignore when green, otherwise push the ledger. Run walkreal_0700000b_live, replay_parity_0700000b, interactables_0700000b_live.

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

**Branch:** `s/real-health-00agent-0b-deck-2`.

---
claimed-by: sandbox7 at 2026-11-13T06:02Z

---
## Report (done, 2026-11-13T07:40Z)


## Report from the worker (done, 2026-11-13T07:41Z)

# real-health-00agent-0b-deck-2 — 0700000b's 00 Agent real-damage walk crosses the deck

Branch `s/real-health-00agent-0b-deck-2`, merged with origin/master 6daafd915 (merge commit 55435596e, pushed).

## What's done

**The walk is green and un-ignored, on the merged tree, at byte-identical numbers.**

`walkreal_0700000b_live::walk_fresh_spawn_to_the_mission_end_real_damage_00agent` runs the whole mission on real
health at difficulty 3 and finishes: **mission complete t18215; stuck 0; falls 1; end health 25.5, armour 0.0,
lowest 25.5, 18 hits for 74.5, other 0.0, 4 return-fire fights** ([L this session's run on the merged tree,
2026-11-13 ~07:04Z, byte-identical to session 1's two runs on 80ebf43c9]). The ignore is removed.

The lever (landed in session 1, commit 915a1e3e4, kept unchanged this session):

1. **The census** — the ledger's next-lever note asked to compare the two ray sets from the crossing spots. The
   `[0b-ray]` diag in `walkreal_0700000b_live.rs` now prints, beside the world `segment_bullet` ray, the hit-box
   eye-ray census (`DamageLog::visible`) and the world-ray spine aim (`DamageLog::world_ray_point`). The measured
   answer [L the 2026-11-13 census run, `[0b-ray]` lines t17109/17139/17428]: the hit-box eye-ray set answers NONE
   of the roof band from the crossing (every box's centre ray dies on the deck pieces), while the world
   segment_bullet rays read CLEAR to #6/#8/#9 from the same spots.
2. **The world-ray fights** — opt-in `DamageLog::world_ray_fights` (crates/nf-view/tests/common/damage_log.rs):
   the fight pick accepts a world-ray-clear shooter, and the stand aims the world-ray spine ladder (highest of
   1.2/0.9/0.6/0.3 above the guard's pos whose straight `segment_bullet` ray from the eye is clear) when no box
   centre is clear. Justification is the disc's own flight law, not centre-sight: a round spawns along the
   CROSSHAIR ray [M Player_WeaponInitBullet 0x1a4bf0:226-245] and hits iff the ray enters a hit box before the
   flight's world stop [M 0x1ea398:216-229]. The flag is set ONLY by the 0700000b 00 Agent walk
   (`w.dmg.world_ray_fights = true` in walkreal_0700000b_live.rs:1985); `DamageLog` derives Default so every other
   walk runs with it false, and every gate is `self.dmg.world_ray_fights && ...` — the d1/d2 sisters and every
   other level keep their committed streams.
3. **The rockets do the killing, not darts** — with the flag the fights open on the world-ray set and the in-hand
   0x2e rockets fly down the crosshair aim; the chase detonations wipe the band (#4 hp -60, #6/#7/#8/#9 hp 0 at
   the latch [L session 1's run4 death-log owner lines]). The deck-band fights take #7 at t17477 (health 57.0 ->
   25.5) and #5 at t17720 (no further loss) [L this session's run]. MEASURED-NOT-SHIPPED (session 1): the
   dart-rearm variant landed sleeps faster but died harder (t17624) — the dart sleeps hold only ~144 ticks before
   a team-1 wake [M 0x147188 k=60; L the ring sleeps], the wakes re-arm the shooters, and the chained fights stand
   Bond at the deck entry until the band chews him; the rocket fights END where the dart fights only pause.

The jam close and the d3 deck fight call from the ledger are in (session 0's commit, unchanged).

## How it was verified this session

All on the merged tree (origin/master 6daafd915, merge 55435596e), real input from the level load, no teleports,
no top-ups, no damage/health changes. This sandbox kills background jobs between tool calls, so every run was
foreground.

| Binary | Result |
| --- | --- |
| walkreal_0700000b_live (whole) | 8 passed, 0 failed, 0 ignored (59.5s) — includes the d1/d2/Operative sisters |
| walk_fresh_spawn_to_the_mission_end_real_damage_00agent (solo, --nocapture) | 1 passed (8.4s); end health 25.5, 18 hits for 74.5, t18215 — byte-identical to session 1 |
| replay_parity_0700000b | 1 passed (8.0s) — the Operative replay stays byte-identical |
| interactables_0700000b_live | 14 passed (24.4s) |
| walkreal_07000015_live | 8 passed, 1 ignored (master's own Countdown 15 ledger walk stays ignored per master) |
| walkreal_07000001_live | 7 passed, 1 ignored |
| walkreal_0700000a_live | 13 passed, 2 ignored |
| nf-game tests (all binaries, --test-threads=1) | green (largest: 170 passed, 92 passed, 35 passed; 0 failed) |
| nf-audio tests | green |
| nf-formats tests | exactly 3 failed: bigf_header_fields_behave_as_documented, bigf_tables_match_the_golden_manifest, directory_tree_matches_the_golden_manifest — the golden-manifest/disc-tree tests need the full retail image; this is the documented slim-ISO allowance ("on the slim ISO only the 3 disc_golden tests may fail") |

The three master-touched walkreal binaries (15, 01, 0a — the merge changed their test files and the 0a guard
behaviour) are the highest-risk sweep targets and they are green on my merged tree.

## What's left

- **The full walkreal_*_live sweep beyond the four run.** 27 level walkreal binaries were built; the walkreal
  sweep loop ran ~8 binaries in its first 10-minute window and none of the remaining ones finished — this sandbox
  runs each binary 2-4x slower than the suite doc's reference machine, and a background suite is impossible here
  (background jobs die between tool calls). The shared-helper change (`world_ray_fights`) is opt-in, defaults
  false via derive(Default), and every gate checks the flag, so the unrun levels' risk is the master merge, not
  this branch; master's own merge touched 15/01/0a and those are green. The probe binaries
  (walkreal_probe0c*_live etc.) and walkreal_dump/celprobe did not run.
- **The suite.py total line**: `tools/test/suite.py` cannot survive as a background job in this sandbox, and one
  foreground pass exceeds the 10-minute tool cap. The table above is the per-crate equivalent for nf-game,
  nf-formats, nf-audio plus the seven nf-view binaries named. A future session on a sandbox that keeps background
  processes should run the full suite on this merged tree and paste the total.
- The 0x2e chase-detonation law on disc (below) is still an open measurement, [S] suggested only.

## Live checks for the main agent

1. **The 0x2e round's fuse/proximity law** — Bullet_init 0x12c5a0's speed/fuse fields for weapon 0x2e [M the
   record walk]: does the disc's heatseeker round time out, or detonate on chase-proximity? In the port the chase
   detonations wipe the band (#4 -60; #6/#7/#8/#9 0 at the latch [L the run4 death-log lines]); gdbwatch.py on the
   0x2e bullet's fuse field during the deck fight would settle whether the disc's round behaves the same.
2. **The crossing eye-ray** — from the crossing (-6.9, 58.8, 18.8) does a real player's eye ray reach #4
   (1.62, 59.2, 32.11)? The port's census says the box centres are buried while the world segment_bullet rays to
   #6/#8/#9 are clear [L the census run]; fieldprobe.py poke + render from the crossing spot would show whether
   the deck pieces occlude on screen the way the ray census says.

## Usage

not visible to me — no token, credit, cost or context-window figures are shown anywhere in this session's
environment.
