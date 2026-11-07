# stream-batch-8 — the merge window

Branch `s/stream-batch-8` cut from `origin/s/stream-batch-7b` (d725c4855) as the task
directed. This window went to the three merges and the first full-tree suite map; no
`crates/*/src/` fix landed yet. Everything below ran on this branch's merged tree.

## 1. The merges (each pushed as it landed)

1. `origin/master` (22d5836b7, the Exchange weapon-refusal line) — clean auto-merge.
2. `origin/s/bullet-pass-water` (e5837c3ff) — already up to date (contained by 7b/master).
3. `origin/s/level-gates-world` (ea3787f06) — **merged WITHOUT its Player_Move ×0.9
   change** as instructed: the whole `move_locked` feature removed (the field + doc in
   `player.rs`, the `k * 0.9` scaling at the `Player_Move` port site, the Sim latch in
   `scene.rs`, and the `the_party_walk_lock_scales_the_move_by_0_9` test). `grep move_locked
   crates/` is empty. The branch's other fixes ARE carried: AttackTalk case 0x11
   (0x93+Rand(2), 1-in-3 on 07000007), the inverted first-wave spawner stinger removal, the
   drone.rs sweep fixes. Two conflicts: `player.rs` (HEAD's `substate_stops` vs the
   incoming `move_locked` initializer — resolved to HEAD after the removal) and
   `docs/WORKLOG.md` add/add (union kept; a merge note marks the ×0.9 exclusion).
4. `cargo test --no-run` green over all four packages: 1145 test binaries build clean.

## 2. The suite map on the merged tree (in progress, resumable)

The chunked runner (`scripts/suite_chunked.py`, per-binary state in
`scripts/suite_state.jsonl`) ran with `--test-threads=1`, the real ISO, whole binaries:

**193/1145 binaries done: 739 passed / 20 failed / 33 ignored.** The state file resumes
for the next window; ~950 binaries remain, mostly the slower walkreals.

## Still red, by name (measured on this branch, whole runs)

| binary | red legs | cause (carried from batch-3/5/6/7 reports, re-reproduced here) |
|---|---|---|
| ai_live 35/2 | guard_notices_and_fires_like_the_game | the SG5 `(4,4)` burst pin — a stream-position pin; rule already stated [M weapon_data +0x40 interval, burst delay ≥ 4+15]; the mechanical fix did not fit this window |
| ai_live | guard_takes_cover_like_the_game | UnderCoverAim 90 vs live-37 window 32..=40 — the file's own seed notes list 90 as a valid alternate-lean outcome; needs the 0x95 exit rule stated |
| tase_silent_live 1/2 | a_p2k_shot_behind_a_guard_still_alerts_him | measured: after setup exactly ONE guard within radius 40 has alert headroom, rise 0.00 [L SFXOutputData] — witness too far; needs a nearer unaware witness or a live gain-curve measurement |
| guard_barks_live | spot_bark_plays_once_within_cooldown | red here, green as guard_barks_07000013_live 4/4 in batch-7; needs its own seed/rule pass |
| fix_portonly_review_live | guard_casing_ejects_from_the_weapon_datum_matrix | carried red (batch line) |
| interactables_07000001_live 9/1 | the_bond_bonus_walk_over_takes_it | walker stuck 1.40 from the bonus target at the 1.3 floor step (batch-6: the walk_to cap needs a look at what stands there) |
| abseil_elevator_live | the_rope_follows_the_opponent_down_and_lands | "hung at the rope top (test 1)" 143 vs 144 — off-by-one wedge at the rope top (batch-6) |
| escort_follow_07000001_live | escort_at_walking_speed… | the 01 escort NPC settles at (3.0, 6.3, 7.3) and never advances to the plate/shoji — the escort-advance/radii wedge shared with fix_npc_doors + mayhew_hold |
| fix_npc_doors_live | the_escort_opens_the_interior_doors_on_his_route | same escort-advance wedge |
| mayhew_hold_live | the_escort_holds_at_the_ally_wait_node… | same wedge |
| goggles_draws_live | a_captains_laser_ray | the press eaten by the start-room stagger (batch-3) |
| upward_shot_0700000c_live 0/2 | both legs | the 15-draw cone realization moved on the joined streams; needs the file's own PCSX2 re-check (batch-3/6) |
| start_corridor_guard_07000011_live | both d2 legs | the 0x5c follow-wedge stall (batch-3/6) |
| walkreal_0700000c_live 2/1/1 | real_damage | carried red (batch-6 battery) |
| walkreal_07000014_live 6/3/1 | real_damage, agent, tripbomb route | carried reds (batch-6: the placed tripbomb no longer stuck at the retreat — a fight/flush shift) |
| walkreal_0700000b_live 6/1 | real_damage_00agent | carried red (batch-6 improved 5/2 → 6/1; unchanged here) |
| **mp_stress_explosives_live** | **CRASH exit −9 (NEW, not on the round's list)** | the binary is killed mid-run — SIGKILL-shaped, consistent with the 3 GB sandbox OOM during the explosives stress, not a test assert; needs a smaller stress size or a memory-capped re-run to classify |

**Improvement vs the round's list:** `walkreal_07000007_live` 6/0 GREEN on this tree —
the level-gates-world AttackTalk 0x11 + stinger fixes ride the merge and hold on the
joined streams (batch-6 pinned 6/6 with them).

**No ignores were added anywhere this window** (the measured-reason ignore lane for 00
Agent walks was not reached).

## 3. What's left (the next window resumes from the state file)

1. The unmeasured walkreals on this tree: 01, 02, 03 (both real_damage legs), 04, 05, 06,
   08 (00agent), 0a, 0d, 11, 12, 13, 15, 16, 1b — run them via the chunked runner.
2. Classify mp_stress_explosives_live's kill (OOM vs hang) at a smaller stress size.
3. ai_live's mechanical SG5 burst-pin → rule assert (the rule is already stated).
4. The remaining lanes in the round's order: plain walks → real damage → Agent → 00 Agent.

## Live checks for the main agent

None new this window. The carried asks stand (batch-6/7): the P2K hearing gain curve
(Sound_Alertness 0x1ddc48 — a near-field shot alert measurement) and the 0x5c follow-wedge
checks.

## Usage

not visible to me
