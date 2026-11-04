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
