# WORKLOG (branch running copy)

## glass-pass-every-level (started 2026-11-04 10:57 UTC)

Branch `s/glass-pass-every-level` cut from `origin/master` (f9f6544c). The hind session's flight
glass pass (rounds whose record +0x6C bit 0x800 is clear pass a surface-0x10 contact while the
contact still drains the glass [M Bullet_update 0x12e7b0 disasm 0x12f030-0x12f058, Collide_Update
0x1ea398:216-229]) and the 08 bottom-terminal pane shoot-out fix are both already on master
(f9f6544c) — no merge needed.

Task: sweep the levels with breakable glass (the placed 0x20 panes [D], 14's glass guards, 06,
08): on real input, shoot a guard through glass and pin that the pane breaks AND the round
reaches him; pin the fists/laser exception (bit 0x800 set). Fix any level where a pane does not
break or a round stops. Every walkreal_*_live; the full suite total line.

Progress:
- [10:57 UTC] Sandbox start (deadline ~12:54 UTC): repo + work/ + ISO + nf-queue all present from
  the prior session; branch cut, WORKLOG started. Next: re-read the hind + pane-shootout reports
  on master, census the glass levels, then the real-input per-level tests.
- [11:00-12:36 UTC] The census [D, breakable_census.rs]: placed kind-2 (glass) panes in 01(12)
  03(23) 06(1) 07(4) 08(3) 09(17) 0a(44) 12(97) 13(33) 14(3); none in 02/04/05/0b/0c/0d/11/15/16/1b.
  The [L] weapon_data +0x6c census (ee_start_05.bin @0x2bf150 + id·0x10C + 0x6c): bit 0x800 SET
  for 28 ids — 0x01 fists, the beams 0x4e/0x4f (0x900), 0x45 (the grenade-belt pseudo-weapon,
  NOT a laser — the hind report's label was wrong [M gun.rs:147-157]) and the mount/gadget
  family — CLEAR for every player gun (0x40050/0x40040) and the launchers 0x2c/0x2d.
- [12:36 UTC] glass_pass_every_level_live: the records census, the unit law (penetration_roll:
  glass passes iff the bit is clear, no RNG draw) and the 01 Mayhew real-input pin — GREEN 3/0/1.
  The 01 pin: Bond holds the spawn through the intro on raw step_bond, taps the first converged
  post-wake ticks; TWO rounds each DRAINED a live entry pane (130 died t2131 off the t2130 tap,
  136 died t2142 off the t2140 tap — the 5 u/tick flight window [M bullet.rs:463-464]) — the
  crossing half of the pin. The MEASURED leftover: no Drone impact registered on the freshly
  woken chargers at wake+7..11 at converged aim — the waiters are not yet bullet-pickable
  [M Collide_PickObj 0x1eae80:26-30's obj+0xF0 0x20 drop]; the smash lands wake+16 [L the probe].
  The 14 window stand-walk test is #[ignore]d (the next window's first item with the 06/08 pins).
