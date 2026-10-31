## Your task: `capsule-cull-and-los-masks-18`, the faithful cull + masks without the 13 reds

Continue `origin/s/capsule-cull-and-los-masks-17` (merge `origin/master` first; read its report). Its two capsule rules check out against the disassembly (the head-on cull compares the RAW denom f2 = [sp+128] at 0x10a1f0 against +/-0.999 with the 0.5 halves, 0x10a288-0x10a2e8 -> 0x10a7b8; Dist2Tri's flags 0/99 skip at 0x10a330/0x10a33c -> 0x10a508) — keep them. But merged onto master the full suite went 13 red: walkreal_07000014_live (the_east_duct_route_reaches_the_stairwell_slot, the_vent_drop_hands_bond_into_the_facility, walk_the_infiltration_to_the_delta_exit), walkreal_0700000a_live (walk_office_floor_qworm_keyboard_hatch, walk_office_floor_to_the_security_keyboard), interactables_07000011_live (multiplexor_seq_and_raises_its_out_on_both_inputs, qworm_computer_e_installs_and_writes_ch0x38), walkreal_0700000c_live::walk_the_whole_mission_from_the_start_to_the_0d_handoff, interactables_07000014_live::the_east_wing_fusebox_cuts_through_the_laser_and_the_e_is_inert, fix_death_restart_live::lethal_fall_reaches_the_death_path_and_the_restart, fix_level_chain_live::the_07000006_party_touch_chains_to_07000007, urn_break_live::shooting_the_sky_bridge_starts_its_break_script, frame_cost_live::frame_budgets_hold_on_every_single_player_archive. Bisect which part breaks each (the LOS masks on, the head-on cull, the flags-99 skip, the turret muzzle/sphere change) and find what still differs from the game — a mask a caller passes wrongly, the grounded rule [M Collide_Update 0x1ea398 + FeetOnPoint 0x194188], or a test that pinned the old stand-in (fix only with [M]/[D]/fresh-boot [L] evidence). Land only with every binary above green AND the whole walkreal_* set green; quote every total.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/capsule-cull-and-los-masks-18`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox6 at 2026-10-31T06:18Z

---
## Report (blocked, 2026-10-31T07:23Z)

