
## real-health-00agent-01-hall
Started 2026-11-11 00:51 UTC. Branch s/real-health-00agent-01-hall cut from origin/master 784ba2d99.
Task: un-ignore walkreal_07000001_live::
walk_from_the_spawn_to_the_exit_with_the_contact_real_damage_00agent — Bond reaches the
AND(0x11,0x12,0x13) hall gate at 85.6 hp but the door phase is ammo-exhausted (the pre-hall
stock already consumed the off-path boxes). Plan the ammo: restock on the route the level
offers [D the pickup placements] before the hall, take the three named guards with head shots
from the aperture. Real input only, no top-ups, never lower damage or health; helpers gated to
07000001 and difficulty >= 3. Run walkreal_07000001_live and interactables_07000001_live.

Work log:
- (00:51) branch created from origin/master 784ba2d99, WORKLOG entry, checkpoint pushed.
  Next: read the ignore message and the walker/damage helpers, then the route + pickups [D].
- (00:55-01:05) reproduced the d3 walk on the new base (784ba2d99): the stream moved again —
  Bond died at tick 6709 to the hall captain's volley after the unconditional stock's corridor
  legs took 51.0 (guards 13/0/8). Read the full ignore + the drain-4 detour machinery.
- (01:05-02:00) five d3-gated levers landed (the hoard stock, the pre-clear diet,
  d3_switch_to_deepest_gun, the reload arg in the inline loops, the crouch + the hold-line
  fight; the threshold stand retired; the flank refused for across guards). 15 instrumented
  runs ([L] traces in the report): the walk now reaches the door phase armed and unhurt there,
  but the deep captain's fire lane out-trades the hold line (his hp 24→16, Bond 51→0).
- (02:05-02:15) two un-gated edits (the lure aim, the reload handling) shifted the d1/d2
  streams — the d1 plate walk died; gated both to d3 and re-verified the base parity [L the
  stash A/B]: the whole walkreal_07000001_live binary 7 passed 1 ignored (the d3 ledger),
  interactables_07000001_live 10/10.
- (02:15-02:26) the ledger pushed into the ignore (the REMAINING WALL + the next lever: the
  west-corridor pre-clear before the meet walk); the fast suite died early on the sandbox
  memory (0-byte suite.out) — the explicit batteries are the coverage; the report written and
  the branch pushed.

## real-health-00agent-01-hall-2
Started 2026-11-11 07:12 UTC. Branch s/real-health-00agent-01-hall-2 cut from origin/s/real-health-00agent-01-hall (4f76e3fb90) and merged with origin/master (already up to date).
Task: continue the 00 Agent hall walk from the prior ledger's next lever — the west-corridor
pre-clear before the meet walk. Helpers stay gated to 07000001 + difficulty; real input only,
no top-ups, never lower damage or health. Remove the ignore when green; else push ledger+lever.
- (07:12) setup: clone, branch, WORKLOG entry, checkpoint pushed. Next: read the prior report
  and the 00 Agent ignore message; downloads (work-slim, ISO) next.
