# level-0a-ring-beams-3 — the report

Task: re-time the four red `walkreal_0700000a_live` qworm walks against the beams' key sweeps, on
top of `s/trigger-beams-3`'s beam-scoped strict trigger law (flags 0x645, `Trigger::seg_beam`),
without widening that scope.

## What's done

1. **The #625 crossing re-timed against its key sweep** (commit a29ee0e8, test file
   `crates/nf-view/tests/walkreal_0700000a_live.rs`):
   - The disc data first: beam #625 (out 0x26, mask 8 [D triggers dump]) is a seg_beam glist
     (flags 0x645 [D]) and its glist rides key motion — keys x 18.877 → 16.078 → 18.877, step
     0.008333 per tick, last 2 = one pass 240 ticks [D keys, `probe_0a_beams` dump; M
     `Trigger_Create` 0x1b1720:56-87 + `Trigger_Update` 0x1b1a78:60-70] — the same sweep shape
     #614 rides on z, along the corridor's x axis. The whole westbound lane (z 4.9-5.3) sits
     inside the beam's z span 2.38..7.64 [D], so no standing pass exists at any phase.
   - The old crossing was a blind press 1.0 before the STATIC line x 18.88 with the arrival at
     x 16.4 — inside the west park's reach (x_min 15.94 at the park − the 0.55 touch radius).
     It passed only while the walk's upstream phase kept the beam east of the arc's low tail;
     the timing shifted and the arc's low tail / the stands overlapped the band's footprint,
     stamping ch 0x26 (the final laser pin's failure on all four tests, measured before the fix).
   - The re-timing (`sweep_jump_beam625`, ring-sweep-timing's method for #614 rotated to x):
     the arm watch waits for the posed box parked east (`box_max ≥ 18.75`; the key-0 park hi
     19.022 [D]) — the takeoff side clear and the westward sweep entirely ahead of the arc; the
     press stands 1.0 before the static line (x 19.88); the arc flies west to x 15.3, past the
     west park's reach; the pins assert the press/landing gaps > 0.56 from the POSED box and
     that ch 0x26 never stamps [D #625 out 0x26]. The arc's middle clears vertically at any pose
     (the #614 probe's measured arc: ~0.1 u/tick, feet ~1.84; the standing toucher's bottom
     B = feet−0.485 [M `Player_Collision` 0x194df0 default arm] clears the band top 0.65 [D]
     while feet ≥ 1.69). The strict law itself (`Trigger::hits`'s seg_beam arm →
     `seg_touches_tri_raw` [M `Trigger_Hit` 0x1b1fc8 → `Collide_Intersect` 0x1ebaa8:93-99 →
     `Intersect_CylGeom` 0x109da0's RAW branch → `Dist2Tri` 0x1dfca8]) is untouched — scope kept.
2. **Two `Trigger` literals the base branch missed** (commit 38fea222): `seg_beam` was added to
   the struct on trigger-beams-3 but two `#[cfg(test)]` literals were not updated, so the tree
   did not compile (`cargo test --no-run`): `crates/nf-game/src/triggers.rs`'s test `plate`
   helper and `crates/nf-view/tests/exit_aim_hint_live.rs`'s `mk`. Both set `seg_beam: false`
   (hand-built volumes, not laser-beam glists). Rule 9.
3. **The probe example** `crates/nf-view/examples/probe_0a_beams.rs` (prints-only): the beams'
   seg_beam status, motion keys and plane records — the data source the re-timing cites.

## How it was verified

- **The four qworm walks green**: `walkreal_0700000a_live::walk_office_floor_qworm_keyboard_hatch`
  + `_real_damage` + `_real_damage_agent` + `_real_damage_00agent` — `4 passed; 0 failed` in
  23.9 s (real input from the level load; no teleport; the sweep-jump's live box at the landing
  read 17.396..17.673, 2.1+ clear of the arc).
- Every required binary, whole, `--test-threads=1`, on the slim ISO (the format
  `passed/failed/ignored`):
  - walkreal_07000001_live 8/0/0; walkreal_07000002_live 3/0/1; walkreal_07000003_live 6/0/1;
    walkreal_07000004_live 10/0/0; walkreal_07000005_live 14/0/0; walkreal_07000006_live 6/0/1;
    walkreal_07000007_live 5/0/0; walkreal_07000008_live 6/0/0; walkreal_07000009_live 8/0/0;
    walkreal_0700000a_live 13/0/2 (the four qworm walks inside); walkreal_0700000c_live 3/0/0;
    walkreal_0700000c_barrelrow_live 1/0/0; walkreal_0700000c_eastroute_live 1/0/0;
    walkreal_0700000d_live 1/0/1; walkreal_07000011_route_live 1/0/0; walkreal_07000012_live
    3/0/0; walkreal_07000013_live 3/0/0
  - interactables_07000001_live 10/0/0; interactables_07000002_probe 0/0/1 (the opt-in probe,
    ignored by design); interactables_07000003_live 9/0/0; interactables_07000004_live 4/0/0;
    interactables_07000005_live 6/0/0; interactables_07000007_live 4/0/0;
    interactables_07000008_live 15/0/0; interactables_07000009_live 4/0/0;
    interactables_0700000a_live 13/0/0; interactables_0700000c_live 6/0/0;
    interactables_0700000d_live 11/0/0; interactables_07000012_live 12/0/0;
    interactables_07000013_live 10/0/0; interactables_07000014_live 10/0/0;
    interactables_07000015_live 17/0/0; interactables_0700001b_live 9/0/0;
    interactables_07000041_live 18/0/0; interactables_0700004a_live 6/0/0
  - fix_truck_live 2/0/0; fix_npc_doors_live 1/0/0; fix_level_chain_live 5/0/0;
    interact_07000005_live 11/0/0; smg_close_range_live 1/0/0; use_key_census_live 20/0/0;
    mp_world_sync_live 8/0/0

## What's left

- **The full suite total line did not fit.** The sandbox recycled at 13:37 UTC (the warm
  `target/` and `work/` lost); the re-setup + the full cold rebuild + the named-binary runs took
  the window (stop-new-work 15:06 UTC). Per lesson #10 this is reported, not skipped silently.
  The next message can finish it with the established runner against this branch:
  `CARGO_TARGET_DIR=$PWD/target python3 tools/test/chunk_suite.py` (re-invoke until ALL DONE;
  the named binaries above are already measured green — everything else is what remains).
- The three `disc_golden` tests are the only allowed reds on the slim ISO (unchanged policy).
- No live checks needed this round: the disc's key list settles the sweep (the [D] keys above),
  the port already re-poses the tris per tick (the arm watch reads the posed geometry the sim
  itself advances), and the crossing pins the channel stamp end-to-end on real input.

## Usage

not visible to me — this session's environment exposes no token, credit or context-window
figures to the worker; nothing to copy exactly.
