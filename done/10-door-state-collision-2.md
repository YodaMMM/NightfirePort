## Your task: `door-state-collision-2`, open doors stop blocking without breaking the walks

Start from `origin/s/door-state-collision` (the `Door_IsClosed` 0x1367b0 filter on the doors' collision parts; merge origin/master). On the full suite it (carried in the capsule-cull branch) turned these red: walkreal_07000001_live `walk_from_the_spawn_to_the_exit_with_the_contact`, probe_07_route `probe_the_route` (Bond STUCK), walkreal_07000014_live `the_east_duct_route_reaches_the_stairwell_slot`, ai_live `guard_takes_cover_like_the_game`. For each, find what the walker/guard now runs into or through (a door that is open in the port but the game's route still treats as an obstacle, a CLOSING leaf the game keeps solid for a few ticks, the doors' group states, cover points that used the door leaf as cover) and decide from the game's rule [M Door_IsClosed 0x1367b0, the zone consumer 0x135738/0x1336f8, the drone cover pick] — fix the port, or re-derive a test's pin from the game's behaviour with the cause cited. All four green plus walkreal_07000008_live 4/4 and every other walkreal / door / mp test on the full suite.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/door-state-collision-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox7 at 2026-10-27T04:39Z

---
## Report (done, 2026-10-27T06:19Z)


---
## Report from the worker (done, 2026-10-27T06:19Z)

Closed by the driver: the report is on `s/door-state-collision-2` (docs/reports/door-state-collision-2.md); the worker didn't run finish.sh.

---
## Report from the worker (done, 2026-10-27T06:19Z)

# door-state-collision-2 — the is_closed solidity filter was the render rule misapplied; movement collision is state-blind, and every gate is green without the filter

## What's done

1. **The `Door_IsClosed` solidity filter on the doors' collision parts is REVERTED**
   (commit 55e06f54): `drone.rs` `set_dynamic`'s doors extend is back to the
   unconditional `glist != 0` extend, with the decomp evidence in the comment. The
   filter (leaves solid iff `Door::is_closed`) was carried from
   s/capsule-cull-and-los-masks-5 and landed by s/door-state-collision; on the full
   suite it turned four tests red. This task explains the mechanism and heals all
   four without weakening anything.
2. **The decomp verdict [M]: the game's movement collision never reads the door
   state.**
   - `Door_Interp` 0x135738 writes the interpolated pose to `obj+0x90` every tick
     (the drawn AND collided pose) and sets `obj+0xFA` bit 0x20 (moved).
   - `control_movement_object_handler` 0x1336f8:147-236 runs every moved object
     through `control_handle_cel_change` 0x133288, which relinks the object into
     the world's cel chain **by position alone** — no state read anywhere.
   - `Collide_PickObj` 0x1eae80 gates on hittest kind (`hittest+4`), `obj+0xFD`
     bits and `obj+0xF0` masks only — never `obj+0xF4` (the door state) or
     `DoorGroupsStates`.
   - `Door_IsClosed` 0x1367b0 (kind 1 always closed; grouped →
     `DoorGroupsStates[g] == 2`; else `obj+0xF4 == CLOSED`) has **exactly one
     caller in the whole decompile**: `Vision_Recurse` 0x1e82e0:51 (←
     `Vision_Portal_Recurse` 0x1e86e8:32 ← `View_CaptureScene` 0x1e6df0:25) — the
     RENDER visibility recursion through door portals (don't draw rooms behind
     closed doors). The prior report's "zone consumer 0x135738/0x1336f8"
     attribution was wrong: those two addresses are the pose/tick path, not
     is_closed consumers. The filter took a render-cull rule and applied it to
     solidity — the wrong layer.
3. **The four suite reds, each explained and healed [L A/B in this sandbox, same
   tree, filter toggled]:**
   - `probe_07_route::probe_the_route` — Bond hovers (grounded=false) over
     `floor_below 43.00006` at (94.17, 44.14, −47.02) to the 6000-tick cap: the
     OPEN kind-0 hall door #244's leaf at (94.888, 43.0, −47.25) **is the sill
     bridge** over the 1.1 step down through that doorway; the filter deletes it
     from the collide set and there is nothing to walk on. Filter off: GREEN
     3.98 s.
   - `walkreal_07000001_live::walk_from_the_spawn_to_the_exit_with_the_contact` —
     GREEN 26.4 s with the unconditional extend (filter on: red on the full
     suite; the route crosses open-leaf geometry the game keeps solid).
   - `walkreal_07000014_live::the_east_duct_route_reaches_the_stairwell_slot` —
     GREEN 9.8 s (2 passed / 1 pre-existing ignored).
   - `nf-game::ai_live::guard_takes_cover_like_the_game` — GREEN (37/37 in the
     binary): cover points that use a door leaf as cover need the leaf solid
     whatever its state; the filter deleted exactly those covers.
4. **The counter-gate holds too: `walkreal_07000008_live` is 4/4 GREEN with the
   unconditional extend** (112.9 s binary run, incl.
   `walk_the_gondola_to_the_briefcase_and_back` with the secure-room walk). The
   prior round's root cause was never solidity: door #10/122's OPEN POSE swings
   clear — its keys [D, door_pose_probe dump] are closed −90° (key1, quat
   (0,0.7071,0,−0.7071)) with the open ends at 0°/180° about the hinge at
   (154.516, 41.999, −121.328); measured in-sandbox after ch 137 + E: pos01
   1.000, matrix row3 (154.518, 41.999, −121.328), ax (1,0,0) — the leaf lies
   along +x at z ≈ −121.33, clear of the doorway. The passability comes from the
   pose (the same mechanism the SS review proved for the script entities), and
   the A/B stall the prior round measured at the secure room does not reproduce
   on the merged tree (master's batch-1026n lineage walks it green). The scary
   "STUCK at the station doors" lines some runs print are the walker's lenient
   give-up legs — the tests pass.
5. **New probe** `crates/nf-view/tests/door_pose_probe.rs` (2 assertion-free
   dumps, --nocapture): the parsed leaf keys + create poses for 07000008's and
   07000007's doors — the [D] evidence the next pose question will need.

## How it was verified (all on the final tree, always-solid, `--test-threads=1`)

- The four named reds + `walkreal_07000008_live` 4/4: item 3/4 above.
- **Every other walkreal — all 24 `walkreal_*_live` binaries GREEN**: 0001 3/0,
  0002 1/0, 0003 4/0 (323.7 s, the slow one), 0004 4/0, 0005 8/0, 0006 2/0,
  0007 2/0, 0008 4/0, 0009 3/0, 000a 5/0/3ign, 000b 2/0, 000c 2/0, 000d 1/0,
  0011 1/0, 0011_route 1/0, 0012 1/0, 0013 1/0, 0014 2/0/1ign, 0015 1/0,
  0016 6/0/1ign, 001b 8/0/1ign, 0041 5/0, 004a 1/0, 004a_satchel 2/0.
- **The door family GREEN**: nf-view — fix_npc_doors 1/0, npc_doors 6/0/1ign
  (the pre-existing ignored escort-reach), door_portal_links 3/0, door_portal
  2/0, fix_door_react_path 1/0, door_aim_ray 2/0, door_text_labels 4/0,
  door_hint_boot_count 2/0, door_glist0_e_dead 2/0, doors_remainder 4/0,
  cutscene_door_07000014 2/0, party_door_lock_07000007 2/0, rook06_realpath
  1/0, door_pose_probe 2/0, probe_07_door_lock 1/0; nf-game —
  door_glist_census_live 1/0/1ign, door_guard_live 3/0, door_research_07000001
  2/0, doors_01_live 2/0, doors_live 2/0, misc3_door_wipe 1/0,
  cover_selection_live 2/0.
- **mp GREEN**: mp_core 5/0, mp_world_sync 8/0, mp_bots 3/0, mp_bots_moving
  3/0, mp_16_slots 3/0, mp_shot_gate 5/0, net_e2e_loopback 2/0, and
  `mp_net_slots_post_move_live` (the EXACT digest pin
  `four_human_session_digest_is_pinned`) 2/0 — the digest did not move (the
  unconditional extend is master's behaviour on Ravine 0700004b, which has no
  doors; the filter never touched the digest either way).
- `nf-game --lib`: **578 passed / 0 failed / 5 ignored**.

## What's left

- **The full-suite tally did not fit the window.** The sandbox kills background
  jobs at the tool-call boundary, so the 847-binary run has to go in foreground
  chunks; this window's chunks went to the explicit family runs above (which
  cover every walkreal/door/mp binary the task names plus their families). The
  suite's build step alone (nf-formats + nf-audio rebuild) consumed the last
  chunk (393 s, zero binaries run). Next window: run
  `python3 tools/test/suite.py` in foreground chunks on this tree and record the
  total; the families above are the acceptance gates and all green.
- `mp_online_failures` OOMs at this sandbox's 3 GB cap (prior round's dmesg
  evidence stands; unchanged by this task — the revert REMOVES no memory paths,
  it restores master's exact behaviour).
- Optional next-round pin: a live PCSX2 check of the 07000007 hall door's open
  leaf (the sill bridge) — the port's open pose there is still the prior round's
  [S]; this task only needs it solid, which the decomp + the A/B prove.

## Live checks for the main agent

1. **07000007 hall door #244 (the probe_07_route sill), while OPEN**: dump
   `obj+0x90` and the leaf's world corners. Expected (port): the leaf lies flat
   over the doorway step at (94.2..94.9, y≈44.1 top face, z −47.0..−47.5) and
   Bond walks ACROSS it ([L] in-port A/B: solid → crosses; filtered → hovers).
   If the game's leaf swings clear instead, Bond must cross on the step geometry
   itself and the port's step/floor at that doorway needs the look.
2. **07000008 door #10/122 while OPEN** (after ch 137): confirm the game's open
   leaf lies along +x at z ≈ −121.33 (inside the secure room), matching the
   port's measured open pose — then the door-pose layer is fully reconciled.

## Usage

not visible to me — this session's driver surface shows no token, cost or
context-window figures (the IM gateway metadata carries only session/chat/trace
ids).
