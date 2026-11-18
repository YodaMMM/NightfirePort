## retune-nb-0c (2026-11-18, s/retune-nb-0c)

Task: commit 3412a89a3 made every guard run the game's spacing scan and the close-range alt attack from the fire states; the 0c whole-mission walks `the_whole_mission_from_the_start_to_the_0d_handoff_walked_real_damage_agent` and `_00agent` now die. Read both ignores and the walkreal0c.rs changes in that commit, fix the fight that kills Bond (real input only, no top-ups, never lower damage or health, never change guard code), helpers stay gated to level+difficulty. Remove each ignore when green; if still red, push the measured ledger and the next lever into the ignore message. Run the whole walkreal_0700000c_live binary and interactables_0700000c_live.

### retune-nb-0c progress

- Reproduced both ignored deaths [L runs]: Agent d2 t6159 (53.76,6.01,-158.87) on the mezzanine
  deck — the balcony pair #19/#20 (y 9.7, d 7.9-9.1) took 56.0 + 18.1 past 9 failed fights;
  00 Agent d3 t35413 (94.20,8.92,-232.05) mid crate-hop — pit-floor shooter #5 (wpn 0x12,
  (62.1,7.2,-207.5)) 74.8 in 51 hits, hp 10.0 intact through FIVE failed fights (the aimed
  fights' cursor-window law at d 17-40).
- Lever 1 (agent_0c): the deck pre-clear (the d3 `hard_0c` block) now gates difficulty >= 2 —
  the Agent rows die under the same pair. GREEN: #19 dropped at d 11.5, mission walked
  28944+838 ticks at 81.1 hp.
- Lever 2 (d3, the walk's own body): from the east ground at the chasm's east the pit shooter
  IS exposed to the plain gun rays (dxz 17.0, dy -0.1 [L the probe]); the walk switches to the
  P2K (spread 0.0, range 50 [L]) and runs bounded pre-clear stands on every live hostile
  exposed within 20 u. GREEN: #5 shot at d 17.0 (clip 7 spare 41), mission walked 36586+838
  ticks at 34.8 hp.
- Both ignores REMOVED. Whole walkreal_0700000c_live: 6 passed / 0 failed (102 s).
  interactables_0700000c_live: 7 passed / 0 failed.
- Measured and dropped: stuck_answers on the d3 armour/drop navs (the wedges are geometry, not
  body blocks — the wrapper was a no-op, reverted).
