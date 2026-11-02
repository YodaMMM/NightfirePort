# level-14-15-real-health-finish — report

Branch `s/level-14-15-real-health-finish`, cut from origin/master 524924db (the five
real-health sisters merged in bbf9ac76; four of them ignored with their measured blockers;
this task finishes two of them).

## What's done

Both real-health walks are un-ignored and green on real input, real damage, no health
writes, no teleports after load. Commit c89fbdb7 touches only the two test files
(`crates/nf-view/tests/walkreal_07000014_live.rs`, `crates/nf-view/tests/walkreal_07000015_live.rs`);
no lib code, no shared test helpers under `crates/nf-view/tests/common/`.

1. **walkreal_07000014_live::walk_the_infiltration_to_the_delta_exit_real_damage** — the
   route's live-wire check no longer asserts the absolute health 100. Per the driver's
   directive it now measures the wire's own hurt: health is read before and after each
   fusebox cut window and asserted non-decreasing ("no live-wire hurt across the #383/#385
   cut") [M FuseBox_Update 0x1adfe8:99-101 — the live wire never hurts with +0x56
   configured]. The absolute assert assumed the top-up; on real health the value carries
   the earlier fights' losses (65.0 in the measured run, 33.0 in the driver's — the fight
   lottery, not the wire).
2. **walkreal_07000015_live::walk_spawn_to_the_omega_sector_exit_real_damage** — routed the
   way a player plays it:
   - the door[38] laser-mine fence #383 is shot from RANGE (~10 u, the exact dance
     `a_p2k_round_from_10_units_explodes_the_door38_fence` pins), not from 4.5 u — the
     old spot sat inside the fence's own blast [M Explode_Propagate 0x17c518 — radius 8,
     damage 100 [L ee_start_05.bin @0x2bf150 + #383's record]]; the measured 4.5 u shot
     cost 44.8 (−16.0, −12.8, −16.0 across frames 2686-2687) and the walk now asserts the
     shot distance is past the blast;
   - door[42]'s fence #447 is taken the same way from the open door[42] threshold (the
     through-the-door shot from z 23 is 5.7 u off the beam, inside the blast; the press
     goes to index 42 directly — the lower-deck twin door[20] at y 23.85 wins the dxz
     `door_near` otherwise);
   - the silo rifle guard (placement 699), the door[32] doorway grenade guard (722) and
     the great-hall grenade guard (733) are dropped explicitly before their lanes, gated
     on `!tops_up()` — the master (top-up) walk never fights, and an unconditional shoot
     there burned the clip dry at #462 (the measured master failure);
   - the 16-arrival kit assert is a superset law: the fights' guard drops are the base
     weapons 0x14/0x1a [M 0x17a2a0].

## How it was verified

All runs on the slim ISO, release build, `--test-threads=1`, real input only:

- **The two task binaries, green**: walkreal_07000014_live — 8 passed, 0 failed, 0 ignored
  (138.75 s), including `walk_the_infiltration_to_the_delta_exit_real_damage`;
  walkreal_07000015_live — 5 passed, 0 failed, 0 ignored (23.09 s), including
  `walk_spawn_to_the_omega_sector_exit_real_damage`.
- **Every walkreal level-walk binary re-run (all 22)**: 124 tests passed, 0 failed, 7
  ignored. All 17 real-damage walks not owned by other tasks ran green (01, 03, 04, 05,
  06, 07, 09, 0a, 12, 13, 14, 15, 16, 1b ×2, 41, 4a). The 5 ignores are the other tasks'
  measured blockers, byte-identical to master: 08 (level-08-long-range-hits' target:
  drone #20 at 60.5 u), 0b, 0c, 0d, 11 (the real-health sisters' documented blockers).
- **Full suite, sliced** (the one-command suite build cannot fit this sandbox's 9.9 GB
  disk — proven twice in earlier sessions and in the WORKLOG; run as foreground slices,
  executables deleted between batches):
  - **nf-game**: 101 test binaries, 0 failures.
  - **nf-formats + nf-audio**: 9 binaries ok; the only failure is `disc_golden` with
    exactly the 3 allowed slim-ISO failures (`bigf_header_fields_behave_as_documented`,
    `bigf_tables_match_the_golden_manifest`, `directory_tree_matches_the_golden_manifest`).
  - **nf-view lib**: 100 passed, 0 failed.
  - **nf-view integration sweep**: 400 test binaries run alphabetically
    (0700000d_gunimp_aim_live → interact_07000007_live), 1109 tests passed, 0 failed.
    30 binaries whose batch hit the 400 s per-batch cap were set aside un-run (slow, not
    failing — batch starting `elevator_buttons_09_live`, `fix_0a_rain_indoors_live`,
    `inv_items_live`-range); 505 binaries after `interact_07000007_live` were not reached
    when the 19:48 UTC stop-new-work line arrived. My branch's delta vs master is the two
    green test files above, so this sweep is a formality belt-and-braces, but it is
    honestly not the whole suite.

## What's left

- The nf-view integration slice sweep stops at ~400/935 binaries (0 failures among the
  400 + all 22 walkreal level binaries + lib). Next message can continue the sweep from
  `interact_07000008_live` alphabetically (the batch list method is in the WORKLOG entry);
  no code is expected to change.

## Live checks for the main agent

None needed — both walks reach the level ends end-to-end on real input, so the port's
behaviour along them is proven by the walks themselves.

## Usage

not visible to me
