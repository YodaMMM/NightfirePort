# drone-block-swaps-g055-3 — the last two real-health walks under the G-055 hunt

Branch `s/drone-block-swaps-g055-3` from origin/s/drone-block-swaps-g055-2 (82128666) with
origin/master (66822831) merged in (the task's first step). Status at the end of this
session: **the two target walks are NOT green yet** — this report is the honest hand-off;
the next session continues from the branch (everything below is pushed).

## What this session did

1. **The branch + the merge.** g055-2 merged with master cleanly except one conflict:
   the east-stairwell-foot y cap in `walkreal_07000014_live.rs` (two copies — the handback
   helper and the full-walk helper). Resolved to the looser cap (`p[1] < 1.1`) with both
   sides' justifications merged (g055-2's FirstAttack-flush fight phase, master's
   agent-difficulty sampling at y 0.405); the x/z bounds and the y > −1.6 pit floor are
   untouched. Commit 7d31ec78.
2. **07000016 (`walk_the_chasers_defeat_completes_the_mission_on_real_damage`) — diagnosed
   and improved, not green.** The death moved with the merge: it is no longer the 9241
   pit-floor death of the task text but a **dry-ammo death at 14401** (then 14551 pre-fix)
   — the route's 8 return-fire fights bleed ~80 of Bond's 100 (he reaches the pit floor at
   20.2) and every gun is dry by the finale squad phase (the measured open: clip 1 spare 1;
   the weapon cycling found no round in any owned gun). The measured chain of attempts,
   all in the branch:
   - The kill loop now side-fires the pit soldiers while the chaser holds outside her 2.0
     punch band (the task's "clear the pit squad, never the chaser", folded into
     `kill_the_chaser_with_range_keeping`'s targeting; the chaser still dies at +2930, well
     under the 5600 window).
   - `lane_fights` + `back_off` (the 07000013 idiom) measured WORSE — the stand traded 61.6
     health in one fight and the run died on the route at 1926 — reverted.
   - The opt-in `finish_fights` escalation in the shared helper
     (`crates/nf-view/tests/common/damage_log.rs`, flag default false, only the d1
     real-damage walk sets it): when the return-fire stand's cap expires without the drop,
     one determined `shoot_bounded` pass runs before the 240-tick cooldown. Measured: the
     fights now KILL (four `killed true` in the last trace) — under the flush the un-dropped
     hunter was re-opening at every leg.
   - A leg-wise hostile clear on the APPROACH route (the 07000014 `clear_hostiles` idiom,
     test-local `clear_hostiles_16`, 900 cap) with corpse walks (the walk-over take feeds
     the carry — the walker's own resupply refuses off-corridor detours, so the corpses ARE
     the walk) and the proactive pulse-1 sit + the hand-back stand (both measured: the
     soldiers are NOT exposed from the shelter interior or the hand-back spot — the shelter
     walls block the eye rays; they idle out harmlessly and stay as opt-in).
   - Still dies: last measured 10684 at the ramp foot — each kill trades 3–39 health under
     the ambush crossfire (Bond's dps with the carried pistol-class guns is the bottleneck).
     **Next session's levers (in the WORKLOG too):** switch to a killed guard's AR for the
     clears (higher dps), cover-positioned fights, and killing the ramp-mouth ambush's five
     fixed spawns [D] before the descent legs.
3. **07000014 (`walk_the_infiltration_to_the_delta_exit_real_damage`) — not reached this
   session.** The merge + the 07000016 diagnosis consumed the window. The binary is rebuilt
   and runs: **8 passed / 1 failed** — every earlier fix on this level (the tripbomb
   re-find, the trench-guard #270 kill, the handback pin) is still green; only the
   delta-exit real-damage walk is red, as the task's list says. The task's two levers
   (clear #281/#283 before the descent legs, or hold the door-wall lane while firing; keep
   the (29.99, 0.61, 38.00) descent-landing pin) are untried.
4. **The merged branch's own new reds (not this task's files, documented for their
   owners):** `walkreal_07000003_live::walk_the_full_route_and_complete_every_objective_
   real_damage` dies at tick 56836 at (−1.85, 5.96, −35.46) (deep-route fight; the `_agent`
   sister passes, the 00agent sister keeps its known ignore) and `walkreal_07000011_live`
   fails 2 of 4 (was green pre-merge; `walkreal_07000011_route_live` passes). Both are
   consequences of the mandated master merge (difficulty-gated-placements,
   trigger-plane-records-2, level-15-agent-doors-2, level-0a-ring-sweep-timing,
   level-08-pane-shootout landed since g055-2's base); no line of this branch touches
   their levels.

## How it was verified (this session's real runs)

- nf-formats + nf-audio: **136 passed / 3 failed / 0 ignored** — the 3 are exactly the
  allowed slim-ISO `disc_golden` failures.
- nf-game (whole crate, `--test-threads=1`): **102 binaries — 1051 passed / 0 failed /
  8 ignored.** This contains the G-055 pins: `drone_block_swaps_live` 3/3, rook06,
  abseil — the four earlier fixes hold on the merged sim.
- nf-view critical binaries (the walkreal/door/guard/swap set, rebuilt after the merge and
  re-run serially with the ISO env): 32 of 116 run in the window — **153 passed / 4 failed
  / 16 ignored**; the 4 are 07000003 (master-merge shift), 07000011 (master-merge shift),
  and the two target walks of this task. 07000014: 8/1; 07000016: 13/1. The full 948-binary
  nf-view run did not fit this sandbox window (the merge forced a full rebuild of all 948
  test binaries plus the walk grind); the per-binary results above are the real ones, each
  run whole with `--test-threads=1`.
- Infra notes for the next session: the sandbox's Bash tool dropped out twice mid-session
  (EOF/429 from the tool gateway — one early "sed shows a syntax error that grep cannot
  find" scare on walkreal_07000016_live.rs was exactly that: the file was never broken,
  md5-verified against HEAD). The frozen env that builds: `PKG_CONFIG_PATH=/home/z/alsa-
  prefix/usr/lib/x86_64-linux-gnu/pkgconfig`, `RUSTFLAGS=-L native=/home/z/alsa-prefix/
  usr/lib/x86_64-linux-gnu`, `NF_ISO=<repo>/Nightfire-onfoot.iso`; changing RUSTFLAGS
  between runs multiplies dep artifacts by hash and filled the 10 GB disk once already —
  keep it frozen.

## What's left

1. 07000016: the survival levers above (the AR switch for the clears is the highest-value
   one — the carried guns' dps cannot out-trade the ambush).
2. 07000014: the two levers in the task text, untried.
3. The two master-merge shifts (07000003/07000011) need their owners or a follow-up task.
4. The full 948-binary nf-view run on the merged branch.

## Live checks for the main agent

Nothing load-bearing from this session (no game-code change; the sim is master's). If a
check is wanted for the 07000016 dry-ammo diagnosis: on 07000016 at the chaser phase,
gdbwatch the player's inventory (the carried guns' clip/spare) from the pack-cinema
hand-back to the squad phase — expect the route's 8 fights to drain every carried gun
(measured: clip 1 spare 1 at the re-arm), confirming the route economy is the blocker.

## Usage

not visible to me — no token, credit, context-window or cost figures are shown anywhere in
this environment's interfaces for this session (a subagent task result quoted its own
per-task token line, which is not this session's budget).
