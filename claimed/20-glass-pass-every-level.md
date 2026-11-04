# glass-pass-every-level — rounds through glass on every level

origin/s/level-08-00agent-hind ports the flight's glass pass (rounds whose record +0x6C bit 0x800 is clear pass a surface-0x10 contact while the contact still drains the glass [M Bullet_update 0x12e7b0 disasm 0x12f030-0x12f058, Collide_Update 0x1ea398:216-229]). Once it is on master, sweep the levels with breakable glass (the placed 0x20 panes [D], 14's glass guards, 06, 08): on real input, shoot a guard through glass and pin that the pane breaks AND the round reaches him; pin the fists/laser exception (bit 0x800 set). Fix any level where a pane does not break or a round stops. Every walkreal_*_live; the full suite total line.

---
claimed-by: sandbox3 at 2026-11-04T10:38Z
