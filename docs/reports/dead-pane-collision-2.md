# dead-pane-collision-2

Task: make breakable panes solid while alive, without phantom collision.

Status: work in progress. Branch `s/dead-pane-collision-2` started from master 5215090c.

Plan:
1. Find why adding the pane parts to the per-tick `set_dynamic` set adds collision where the
   game has none: double in the static world, double transform (world-space triangle data),
   or `shape_key` resolving to the wrong object. Print each pane's part bounds against its
   placement bounds on 01, 08, 09, 0a. Cite [M Collide_PickObj / Break_Create / Break_Kill
   0x12b970:59-63].
2. Fix the pairing, add the parts to the set, re-enable `BREAK_WORLD_STOP`, pass
   `blocked_reach`; un-ignore dead_pane_live (both), keep gondola_windows_live,
   breakables_live, interact_07000007_live, interact_07000009_live,
   interactables_0700000a_live and all the regressions listed in the task green.
3. Census over every SP level: no pane part lies more than its own bounds away from its
   placement bounds.

## Usage

not visible to me
