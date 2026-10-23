# WORKLOG

(Started fresh from the rewritten master a67f1a27e; older entries live on the pre-rewrite branches.)

## lobby-roof-glass-09 — start

- Branch `s/lobby-roof-glass-09` created from origin/master a67f1a27e.
- Task: 07000009 lobby roof glass (round 2). `crates/nf-view/tests/interact_07000009_live.rs::shooting_the_lobby_roof_glass_breaks_it` — on master the test is NOT ignored but the driver says its fire harness cannot land a gun round on pane #123 since the bullet filter `filter | 0x106` [M 0x12bfd8:15 + 0x1ece30:24-33] landed. The full ignore analysis from the previous round lives on `s/995-gondola-windows-08` (1ade0bc73): the muzzle side offset puts every bullet outside the pane's 0.2-wide x-slab.
- Plan: reproduce on master, trace the game's shot route from a standable spot (pane filter param, weapon word, muzzle origin), port/fix, un-ignore-equivalent (make it pass), sister stays.
