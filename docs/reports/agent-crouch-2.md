# s/agent-crouch-2: the 0a stealth routes re-timed for the game's crouch pace

Built on s/agent-player-moves-crouch (the crouch walk eases on the 0.75-scaled lane, 0.039/tick
at full stick [M Player_SSCrouch 0x1924c0:58; Player_Update 0x193600:296-299]). No engine code
changes here: only test routes in `walkreal_0700000a_live.rs` and `fix_0c_hatch_live.rs`. No [L]
pin was touched or loosened.

## Why the five walks went red

All five failures had one cause: the office civilian g8 (team 3, sight 12, 180° cone, from the
`NF_ALARM_TRACE` roster). The slower crouched #607 crossings (the prologue's two, plus the
ring's) put the ring about 150-300 ticks later than before. That is exactly when g8's patrol
crosses the west corridor's south mouth. The port's walked trace of his lap (about 2400 ticks):
(−15.3, 6.5) → his pause at (−18.6, 5.4) → the mouth (−22, 6.3) → the west room (−26.4, 10.0)
→ (−35.9, 8.7) → back east along z ≈ 4-5 → the mouth again.
- Hatch walk (and its three real-damage sisters): g8 saw the #614 landing at d 3.3, went
  0x3e → 0x63 → 0x8a PressAlarm, and the mission failed on lane leg 6. In the real-damage runs,
  wall gun #639 went to COMBAT under the alarm and killed Bond at (−24.4, 1.1, −8.8).
  A first try that held north of the beam was worse: g8 saw the waiting stand straight down the
  corridor at d 9.9.
- Bridge-edge walk: its ring walked into g8 at his (−18.6, 5.4) pause (d 0.1), and he got to the
  alarm before the jink-3 leg ended.

## The re-route (how a player would play it)

`wait_mouth_window`: Bond waits in cover in the west offices' north corridor, out of the west
corridor's line. He moves once every live non-team-0 drone is either out of sight range (12) of
the landing and the lane's first legs, or has gone into the west room (x < −26.5, z > 9) facing
west. Then he makes the crossing behind g8's back.
- Hatch ring: the wait comes before the "west corridor's head" leg. It opens after 668 ticks.
  The sweep-jump, the lane's stealth gates and every pin after it are unchanged.
- Bridge-edge ring: the same wait comes before "down the west office block". It opens after
  894 ticks.

## Results (this branch)

The five tests were red before the change (the failure messages above) and are green after:
- bond_stands_at_the_mouth_bridge_edge
- walk_office_floor_qworm_keyboard_hatch
- walk_office_floor_qworm_keyboard_hatch_real_damage (end health 97.6; it was 16.0 on the old
  timeline)
- walk_office_floor_qworm_keyboard_hatch_real_damage_agent (end health 96.8)
- walk_office_floor_qworm_keyboard_hatch_real_damage_00agent (end health 96.0)

The laser pin (no mask-8 beam or alert OR stamped) and the hatch exit pin hold in every run.

## The 0c roof walk (fix_0c_hatch_live)

`from_the_duct_top_the_roof_walk_descends_to_the_balcony` went red with the crouch change
(y 15.3725 against the [L] 15.6536 ± 0.25). Bond stayed crouched for 900 ticks and was sampled
pressed against the balcony's south wall. There his root bobs 15.35 ↔ 15.62 on a 90-tick beat,
and the old pace happened to end on 15.62. The [L] value is a standing root (balcony 14.61 +
1.0436). The walk now crouches under the hatch, stands up past the vent plane with the real
crouch-key edge (pulsed every 70 ticks for the 60-tick lockout), and reads the walked rest at
z ≤ −118, inside the [L] span. The rest reads (5.80, 15.7226, −118.07), 0.069 above the [L]
root. The test also pins that Bond stood up. It was red on the old route and is green now.

## The sweep (every walkreal binary + the 0c crouch/vent binaries)

115 binaries: all 107 `walkreal_*` plus fix_0c_vent2_live, fix_0c_hatch_live,
interact_0700000c_live, interactables_0700000c_live, fix_ss_operate_0c_live,
probe0c_plates_route, fix_level_0c_live and walk_0700000c_live. Default (non-ignored) tests:
114 binaries green on the first pass, 293 passed and 32 ignored in total. The one red was
fix_0c_hatch_live's roof walk (above). It is green after the fix, and that binary is now
5/5. The 0c vent crawls are green: fix_0c_vent2_live 3/3 and walkreal_probe0c_ventroute_live
1/1.

## Unproven

g8's lap comes from the port's simulation, not a live capture. The window rule depends on the
port's patrol matching the game's. A live read of his lap would confirm it.
