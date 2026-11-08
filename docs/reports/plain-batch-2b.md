# s/plain-batch-2b

Fixes for the ten red tests on s/plain-batch-2 (d44399d4e). No game code changed: each red was a
test pinning old port behaviour that a merged branch corrected, or a digest moved by a merged rule.
Each cause was found by running the reds at every first-parent merge point of the batch.

| Test | Cause (branch) | Fix |
|---|---|---|
| light_create_census_live | unread-params-props: the 0x4E light reads the params block | Test reads params p0..p8. `$s1` is the dispatcher's second argument (0x1d050c `daddu s1, a1, zero`), the params block, not the placement record in `$a0` [M 0x1d0b8c-0x1d0bec]; [L] ee_start_07000001 LightList radius 6.0 / rgb 255,121,103 are the params values. Added the mask check. |
| searchlight_live (beam through Bond) | none in the batch: red on master too | Test now follows the port's kept player-sphere spot pick (Bond in the beam catches the spot and arms the head). Renamed `the_beam_catches_on_bond_and_a_zero_alarm_writes_nothing`. Still open: does the game's spot rest on Bond? (mask 0x10 skips the dynamic walk [M 0x1eb320:314-328]; needs a live render). Flip the pick and this test together. |
| walk_0700000c (photograph, gate) and walk_0700000d (photograph) | agent-gadgets: the Micro-Camera fires only with the aim bit up [M Player_WeaponFiring 0x1a5460:512-514] | The walks raise the camera until `sim.aiming`, then fire while aiming. |
| spots_0700000a_qworm_pc_installs | agent-qworm-ray: the Q-Worm fires only with the PC first under the crosshair, within range [M 0x1a5460:531-545] | Stands on the walked south stand (the old stand has a partition 0.14 in front of the eye) and aims at the PC mesh (+0.2) for a Check_Target cycle before the taps. |
| camera_surf_eye_live diving | agent-player-moves: the pitched swim stroke eases from cos(pitch)·lane [M Player_Move 0x191dd0:149-165] | The dive off the bank now reaches the shallow floor; the standing head bobs back over the water line, so Bond is surfaced and the eye rightly rises. The test swims out 60 ticks before diving. |
| load_scene_equality | agent-cable08: wires draw at `wire_draw_matrix` (`Wire_Create` 0x1981d8:12-13 negates rot.z before View_RotTransMatrix [M]) | Re-pinned 02/05/0c hashes, counts unchanged. A probe of every 0x3E placement: 02's and 05's yaw-only wires differ by one ulp in the sin/cos cells; 0c has three yaw-only wires (one ulp, signed zeros) and one tilted wire (rot.z -0.4887) whose slope now flips, like 08's zip line. 01 and 14 have no wire. |
| mp_net_slots four-human digest, mp_perf_16 digest | agent-crouch-2: crouch smoothers ease from the 0.75-scaled lane [M Player_Update 0x193600:296-299] | Re-pinned. On the batch head, restoring only the old unscaled crouch read-back gives back both old pins (0xca2f_e7f4_f872_763f, 0x7e8f_c618_440c_28c1). |

Notes:
- The agent-cable08 review says the yaw-only wires are unchanged. They actually move by one ulp,
  and the review does not mention 0c's tilted wire. Nobody has rendered that wire next to the game.
- The MP digests also moved at the agent-player-moves merge alone. That came from its interim
  crouch read-back (crouch_k re-read after the crouch toggle inside the tick), which agent-crouch-2
  replaced. The batch head no longer has it.
