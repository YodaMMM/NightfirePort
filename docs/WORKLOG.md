# WORKLOG

## level-14-float-narrow
Started 2026-11-06 11:21 UTC. Branch s/level-14-float-narrow (from origin/s/level-14-float-on-master
80f472e88, the hover fix + probe14_pocket_settle on current master d6245281a). Task: "14 hover: a
narrower fix" — the broad suite failure of the spine-fallback push means it does real work; measure
where the small up-facing fallback push fires on a lift and on an 11 stair flight and why those need
it while the 14 pocket does not (cite the game; the port capsule is a stand-in, so the push may stand
in for the game's floor band or Player_FeetOnPoint grounding). Prefer a fix to what stops Bond
grounding at the pocket (walk floor band missing by an ulp: dy 0.65 against cc + 0.1, candidate).
First run: fix_autoaim_elevator_live, walkreal_07000011_route_live, mp_net_slots_post_move_live,
mp_perf_16_live, fix_npc_doors_live, probe14_pocket_settle — all must pass with MP digests unmoved;
then the walkreal set whole. No digest or walk re-pins to absorb the change.
