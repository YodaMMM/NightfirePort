# level-13-lobby-rifles-2 — the 13 lobby rifles land without breaking the other walks

Task: `s/level-13-lobby-rifles` makes `walkreal_07000013`'s real-damage walk pass, but merged onto
master it kills two other real-damage walks:
`walkreal_07000001_live::walk_from_the_spawn_to_the_exit_with_the_contact_real_damage` (drone #12,
tick ~14958) and `walkreal_07000016_live::walk_the_chasers_defeat_completes_the_mission_on_real_damage`.
Both pass on master and with the branch reverted. The cause is the shared walker delta.

## What the shared delta actually was

The suite has TWO walkers, `tests/common/walker.rs` (the 01/02/03/16 walks and ~90 more files) and
`tests/common/walker_b2.rs` (the 09/11/13/14/12 walks). Both `include!("damage_log.rs")`, so every
damage_log change hit BOTH families, while the resupply change hit only the b2 family.

**damage_log.rs (all walks):**
- `exposed_part` grew the gun stand-in rays: the eye ray AND two low-lateral gun-offset rays must
  all clear before a hit box counts as exposed (the 07000013 security-room stand held aim while
  every round died on the door jamb [L the fight#9 rf-trace]). For the non-lane pick this meant
  guards that used to be fightable became invisible — the return-fire gate found nobody visible and
  the walker walked on under fire. That is the 01 death mechanism ([M] the pick at
  damage_log.rs `return_fire`, `cands.iter().find(|&i| DamageLog::visible(sim, i))`).
- the unseen-cover creep (forward 0.5 bursts while the aim is None) fires for any walk with
  `back_off` set — 01 sets `back_off = Some(4.0)` [M walkreal_07000001_live.rs:32];
- the stand's unseen give-up moved 60 → 120 ticks [M damage_log.rs stand_and_fire].

**walker_b2.rs (b2 walks only):** `resupply` — the fetch cap 45 → 12 u, the pickup floor filter
1.5 → 0.5, a tried-target list, the ammo count now includes the inventory, arrive 1.8 → 0.6.

## The fix: opt-in gate (task option 1)

Every shared behavioural delta now branches on `DamageLog::lane_fights`, which ONLY the 07000013
walk sets (all three chain segments: 12, 4A, 13 — [M] walkreal_07000013_live.rs:66,86,108). Default
paths are byte-for-byte master behaviour; the 13 walk keeps the full merged rule set:

- `exposed_part(sim, i)` restored to the single eye ray (pre-merge body); the 3-ray set moved to
  `exposed_part_gun_rays` / `visible_gun_rays` [M damage_log.rs:222-239, 285]. The lane pick and
  the lane stands call the gun-ray variants; every other caller (walker.rs users, the 02/16 test
  files' own calls) keeps the shared signature and master behaviour.
- the stand's unseen give-up: `if lane_fights { 120 } else { 60 }` [M damage_log.rs:534].
- the creep: gated on `lane_fights` [M damage_log.rs:569].
- `resupply` (b2): the five deltas branch on `lane` — ammo counts the inventory, filter 0.5, the
  tried-target filter, fetch 12, arrive 0.6 ONLY for the lane; master values 1.5/45/1.8 and
  clip+spare otherwise [M walker_b2.rs:300-357]. The tried list still fills (harmless) but the
  non-lane filter ignores it.

No game-code changes (the merged `world.rs` diagnostic fix `scoped_run_owners` + `scoped_run_bounds`
stay — diagnostics only, no sim path). No health or ammo writes anywhere; real input only. No shared
file rewritten; WORKLOG entries appended only.

## Verification (this sandbox, release, --test-threads=1, fresh runs)

| test file | result |
|---|---|
| walkreal_07000001_live | 6/6 pass (84.8 s) — incl. the formerly-killed contact real-damage walk (5.4 s) |
| walkreal_07000016_live | 14/14 pass (75.0 s) — incl. the formerly-killed chasers real-damage walk (12.8 s) |
| walkreal_07000002_live | 2/2 pass |
| walkreal_07000003_live | 4/4 pass |
| walkreal_07000013_live | 2/2 pass — incl. `walk_the_warm_chain_to_the_front_entrance_and_out_real_damage` (the 13 fix intact) |
| walkreal_07000014_live | 6/6 pass |
| walkreal_07000009_live | 6/6 pass (b2 user, back on master's resupply) |
| walkreal_07000011_live / _route_live | 1/1 + 1/1 pass |
| walkreal_07000012_live | 1/1 pass |
| interactables_07000012_live | 12/12 pass (b2 user) |
| guard_timeline_probe, sleeper_push_live, walkreal_probe12c/12d/11_48, probe0c ×4 | pass (the ignored ones stay ignored) |
| nf-game, nf-formats, nf-audio (whole crates) | all pass EXCEPT the 3 `disc_golden` tests (bigf header/tables, directory tree) — the allowed slim-ISO failures |

So: every required real-damage walk (01 02 03 13 14 16) + the full 13 file + the b2 resupply users
pass, and the only failures anywhere are the 3 permitted disc_golden ones.

## What's left

- The full `cargo test -p nf-view` fleet (~150 further test files) did not fit the sandbox window
  (cold build ~40 min + the disk-full strip/prune cycle ate the first hour). The files NOT run are
  the walkreal_probe0c_* remainder, the crane/qclaw probes and the interactables/golden families
  beyond those listed — none of them sets `lane_fights`, so they take the master paths this task
  did not touch; the changed shared functions' only behavioural branch is on `lane_fights`.
- Live checks for the main agent: none required (no visual work; no sim/game-code change).

## Usage

not visible to me — this environment shows no token, credit or context-window figures.
