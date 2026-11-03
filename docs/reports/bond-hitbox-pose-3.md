# bond-hitbox-pose-3 — the crouch/duct reds: two merge restorations, the remaining reds rooted

Branch `s/bond-hitbox-pose-3` = `origin/master` (6f16146e) + `origin/s/bond-hitbox-pose-2` merged
(1f16f170). The window lost ~1.5 h to two sandbox resets (setup redone twice: clone, work-slim, ISO,
rustup, ALSA prefix, cold build). What follows is what the final tree carries, what was run, and where
the remaining reds actually come from.

## Fixed on this branch

1. **The merge had silently reverted two master fixes** (the pose-2 lineage predates them; git took
   pose-2's older lines in auto-merged hunks):
   - **The walk-move-clip `move_local` retain** (player.rs): the merge carried the old
     `self.move_local = [0.0; 2]` every-tick zero and the climb `=` writes. Restored master's:
     the quad zeroes on CLIMB ticks only [M Player_Update 0x193600:129-134 — case 1 → Player_Climb's
     own Vec_Zero pair], the walk body's stores overwrite per tick [M Player_Move 0x191dd0:166-170],
     and the climb/dismount/lunge/wire writes are ADDS on the retained quad [M 0x196310:56,90;
     0x1969d0:151; 0x197f50:121] — the guards' accuracy moving test reads them
     [M DroneWeap_DoOpponentTargetting 0x179a48:34-41].
   - **The weapon-switch owned read** (nf-view scene.rs): restored the BASE-slot read
     `owned.get(base_of(id))` [M Player_EquipWeapon 0x1a8a58:39-41 — the twins share `BL+0x1BA +
     base·0xC`] that pose-2's older line (`owned.get(id)`) had displaced.
   - Re-added pose-2's `crouch_view_height_matches_master_per_tick` camera-contract pin (the only
     net player.rs change pose-2 really carried — its toggle already writes `crouched`+`cc` only,
     exactly master's [M Player_HandleJump]). It passes on master's player.rs: **656 lib tests green,
     0 failed**.
2. **The bullet world trace keeps master's cel-scoped pick** in the merged `Bullet::update`
   (bullet.rs): `segment_bullet_scoped(world.find_cel(pos), …)` [M Bullet_init disasm 0x12c8d0
   obj+0x20 = build_LinkToRoom; Bullet_update disasm 0x12ef04 re-feeds the cel; Collide_Pick
   0x1eb320:125 → Collide_StraddleCels] **plus** pose-2's posed-box player walk
   (`damage::first_box_hit` over the 16 `HumanCollisionBones` boxes [D 0x2c6be8; M Collide_Intersect
   0x1ebaa8 → Collide_Jointy kind 0x201]). Plain pose-2+master (unscoped `segment_bullet`) turned the
   three named duct tests red; on this tree they are **green**:
   `the_east_duct_route_reaches_the_stairwell_slot`, `the_vent_drop_hands_bond_into_the_facility`,
   `walk_the_infiltration_to_the_delta_exit` — all ok in walkreal_07000014_live (6 passed / 2 failed).
3. Build fixes for the boxes API: `reload_and_bond_hitboxes_live`'s `bond_at` gained master's
   `body`/`move_local` fields (lesson 9); the `the_burn_segment_never_passes_the_record_range` unit
   test walks no boxes (`&[]` — its old far-away `PlayerVolume` cylinder never enters the boxes API).

## What runs green on the final tree (5359e087)

| suite | result |
|---|---|
| nf-game `--lib` | **656 passed / 0 failed** (incl. the camera pin) |
| fix_bond_damage_live | 7 / 0 |
| anim_events_live | 4 / 1 ignored |
| reload_and_bond_hitboxes_live | 6 / 0 |
| bond_boxes_every_level_live | 1 / 0 |
| mp_perf_16_live | 2 / 0 at the master-lineage pin |
| walkreal_07000014_live | 6 / 2 (below) |
| walkreal_07000004_live | 6 / 2 (below) |

The **full suite did not fit**: two resets consumed the first window; this window's build + the runs
above end at the stop line. The chunked runner (`tools/test/chunk_suite.py`) is the way to finish it;
no total line this time — the next message should run it on this tree first thing.

## The remaining reds, rooted

All four trace to ONE model change: **the guards' rounds now hit-test Bond's 16 posed boxes instead
of the old cylinder** — the game's own model [M 0x1ebaa8/0x1ec180; D 0x2c6be8]. The cylinder was the
documented deviation, and it MISSED rounds the game's boxes catch. Every red below is a
cylinder-era choreography expectation breaking on the game's model:

1. **walkreal_07000014_live::walk_the_infiltration_to_the_delta_exit_real_damage** — the shared
   east-duct route helper's stairwell-foot assert: y cap 0.2, observed **0.3419** (x/z pins hold,
   pit-floor pin holds). The corridor fight with drone #0 shifts tick phase because the guard's
   rounds now connect on the boxes (different first-hit tick and part) — the test's own comment
   documents the same coupling for the 2026-11-02 walk-move-clip re-derivation. The new phase is the
   game's; the cap was calibrated under the cylinder. Re-derive the cap with a fresh observation +
   the evidence note (the x/z and pit-floor pins stay).
2. **walkreal_07000014_live::walk_the_tripbomb_route_pick_up_place_and_shoot_it** — instrumented run
   (scratch eprintlns, reverted): stick frame 22415, armed frame 22536 (121 ticks [M 0x12ce80:330]
   — the arm is correct), then at frame 22633 the bomb's laser **triggered on BOND himself**
   (dist 1.00 to the wall-clipped beam [-32.10,1.75,36.29]→[-34.98,1.75,36.29]; nearest guard 1.88):
   during the under-slab crawl a guard round CONNECTED with the crouched boxes and its pain push
   shoved Bond ~1.4 u north into the beam [M the beam's char trigger 0x12ef40-0x12ef78; the port's
   `hit_push`]. The blast deletes the gadget (state 2, drained by the mid-pass retain) →
   `grenades[gi]` panics (len 0). The test's premise (a clean retreat, the bomb alive) is cylinder-era
   choreography. Needs a re-derivation against the game's crawl (does the game's Bond take that hit
   under the slab? which part?) — a live check, below.
3. **walkreal_07000004_live::the_results_page_…_and_scores_the_real_tallies** and
   **::every_stats_row_matches_the_game_formulas_from_the_real_finish** — the tracked tallies print
   `fired 35 hit 37 acc 106%`: **the hits tally moved** (+2 over fired; the cylinder missed two
   rounds that the boxes connect, each registering an NDrone2_HitDamage event [M 0x199840 ←
   0x145558:60]). The failing assert is the Stats page's Accuracy row: the row reads **"100%"** while
   the tracked stat says 106%. Decompile fact: the row's pct is `round(f32 @ score_entry+0x70 × 100)`
   [M P_NFSTATS_Handler 0x209938:86-92 — no clamp at the reader], a STORED ratio in the committed
   score entry. The port's `ScoringEntry` has **no ratio field** (progress.rs stops at +0x38), and
   the row is built from the post-commit snapshot where the run's ratio never landed — hence 100
   (the fired==0 full-marks rule [M 0x1984b0:74-78]). The port is missing the +0x70 write at the
   commit; **the writer's store is not yet pinned** (0x1984b0's block or the level-end hand-off —
   the decompile read is the next step), and with hit 37 > fired 35 the unclamped row would show 106
   — the game's own reader allows that, but pinning it needs the writer evidence first. The +2 hit
   events themselves are consistent with the boxes model; the walk's real-tallies pin then needs
   re-derivation against a live finish.

## Live checks for the main agent

1. Crouch under the 07000014 armoury slab (the tripbomb lane, under-shelf band y 0.9..2.2): does a
   guard's round connect with crouched Bond at y≈1.45-1.8, and which part? Watch
   `Collide_Jointy` 0x1ec180 hits on Bond's boxes (obj+0xFA bit 3 set [L ee_start_05 0x16cefe0]) and
   Bond's pos delta the tick after (the pain push magnitude the game applies).
2. 07000014 east duct, corridor drone #0 fight at difficulty 1: Bond's health per hit (the parts the
   rounds take) — to re-derive the real-damage stairwell-foot y cap from the game's fight length.
3. The score entry's +0x70 ratio: after a mission finish, dump the committed `ScoringTable` entry
   (base map 0x07000001's entry) and read +0x70 — is it the run's hits/fired unclamped (106-style
   values possible) or clamped at 1.0? That decides the 04 row fix.
4. The results tallies on a real 07000004 finish (fresh boot): shots fired / hits on the results
   page — do hits ever exceed fired in the game (blast hits counting)?

## Left

- The full suite on this tree (chunked runner; ~2 h) — quote the total line.
- The four re-derivations above, each behind its evidence/live check.
- The commit's +0x70 ratio write (decompile the 0x1984b0 store first).

## Usage

Session token/credit/cost figures: not visible to me — no environment output in this sandbox exposes
them. Context window: the conversation runs in a continued session; per-message budget not visible
either.
