## level-0a-bridge-reach (2026-10-31, s/level-0a-bridge-reach)

Branch from origin/master (bf6b5a2a, batch-1031g). Task: 0a reach the bridge on real input —
walkreal_0700000a_live still teleports Bond to [-11.35, 1.12, -18.6] (REACH onto the bridge's
west end, ~line 884) and [-8.0, 1.12, 16.29] (~line 1005). Find the game's route to each spot
[D collision/placements/nav; M the classes used], walk it on real input, delete the writes;
where the port wedges where the game does not, find the collision difference with evidence
(capsule rules: head-on cull on the raw denom [M 0x10a1f0-0x10a2e8], Dist2Tri flags 0/99
[M 0x10a330/0x10a33c]). If the data shows no route, pin that with a sister assert and name
the live check.
