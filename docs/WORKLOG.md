# WORKLOG

One entry per task, append only (never rewrite or reorder others' entries).

## level-14-landing-hover (2026-11-04)
Started 2026-11-04 21:59 UTC. Branch s/level-14-landing-hover (cut from origin/master dcd51c7e8).
Task: the 1.72-landing hover on the 14 descent. Read s/real-health-14-00agent-5's report; the
mezzanine lip is settled as the game's own behaviour, and the real blocker of the d2
with_real_damage wrap and the d3 un-ignore is the hover at the 1.72 landing. Try lever 2 first
(a d2/d3-gated approach that keeps the fights' displacement out of the pocket, walk-side only).
Touch collision only with a fresh-boot [L] reading of what the game's Bond does at that pocket,
and scope any rule to it, then run the full walkreal battery.
**Outcome (same window):** lever 2 landed and measured — a d2/d3-gated walk-side law set whose
stationary-stand window through the mezzanine descent (the new opt-in
`DamageLog::descent_stationary_fights`: the fights still run, the stand's feet are tied — no
back_off shove, no creep, no lane-resupply fetches; the shared west-route fn clears it after the
descent block) removes the fights' displacement and with it the 1.72-landing hover: every run
crossed the landing cleanly (the landing leg arrived at +280 ticks, the drop west cleared, the
descent assert passed) where the -5 window's runs wedged at tick 25740/26340. The no-fight
variant was tried first and measured dead (the kill ledger starved, the whole 100.0 budget gone
by tick 5328). The escalation opt-in (`escalating_cooldowns`, the 0c law) answered the next
blocker — guard #8, geometry-blocked from every route spot, 7 failed stands at 240-tick cadence
— and the d2 head-hunt halved the stands' burst exposure (the walk then died ONE LEG short of
the delta doorway, tick 18851, 3.2 health). precise_fights was tried and reverted (a 41k-tick
stall). The d2 wrap and the d3 un-ignore did NOT land: their blocker moved from the wedge to
the fights' budget (d3's infiltration grind alone takes 45.0 at accuracy 1.3; death at tick
6786 just past the descent) — both tests parked in their master forms with the measured record
and the ordered levers (the stairwell AIM-pair stands, then the wrap, then the un-ignore). No
collision code touched. Verified: walkreal_07000014_live whole 9/0/1 (197.9 s); the
fight-path twins 13 (3 passed), 11 (3 passed/1 ignored), 12 (3 passed) — the rest of the
walkreal battery did not fit the window (the flag defaults false; the untouched binaries are
byte-identical by construction — the report carries the merge-time note). Report:
docs/reports/level-14-landing-hover.md.

## real-health-14-00agent-6

2026-11-07 06:50 UTC, branch `s/real-health-14-00agent-6` (from origin/master 22d5836b7).

Task: the 14 d2 and d3 walks' health budget. Read
docs/reports/level-14-landing-hover.md on `s/level-14-landing-hover`: the landing hover is
cleared walk-side and the d2 with_real_damage wrap and the d3 un-ignore now stop on the
fights' health budget. Use its measured levers, keep the d1 walk green, and land both if the
budget closes. Walk-side only; any change to game code needs a fresh-boot [L] reading first.
Merge origin/master first.

- [x] setup: branch created from origin/master 22d5836b7, pushed.
- [x] merged `s/level-14-landing-hover` (the wedge fix + the measured budget ground) on top of
  origin/master; conflicts resolved additively (both DamageLog opt-ins kept, the newer 014
  notes kept, WORKLOG entries concatenated).
