# WORKLOG

### level-16-every-objective (started 18:08 UTC, 2026-10-29)

Task: walk the WHOLE 07000016 mission from a fresh load on real input — every objective the
level's mission data lists, in player order, each asserted complete, ending on the level exit
firing. First list what the current tests cover (and every stand-in/poke), add legs for the
rest; every blocker is a port gap: find the game's behaviour [M/D] and fix it, or list it
(draw-only) in the report. Full suite, quote the total line.

Plan: branch pushed first; then inventory the existing walkreal_07000016_live coverage +
objective records [D], then the legs.

Done (18:08-19:25 UTC): the inventory — 07000016's mission data lists 2 objectives [L record 22
@0x2a46c0, objtable 0x2a42f0 ×2]. obj0 "Reach Drake's launch silo" (ch 0x8f) was already fully
pinned on the real-input walk (`walk_arrival_to_the_chasers_spawn`: fresh load + the carried
07000014/15 RamSave, the corridor + the security-station E + the panels, states[22][0]==4);
the carried 07000015 record completes at the arrival plate (states[21] all 4). obj1 "Prevent
Kiko from launching into space" (ch 0x90, the KikoDeathButton #693 E press, gate 0x8d) is NOT
reachable on real input yet — two blockers, both evidenced: (1) U-051 — the launch chain
re-enables Bond INSIDE HurtPoly-ShuttleLaunch #309's box at the pit floor [M Player_Enable
0x195498; D the pack cinema's single t747 key], the pulse runs at launch-script ticks 1611..2170
[D stream-6], the pit door locks before it and the lift summons after it, and the game's escape
is the pack (jetpack) flight — a substate the port does not enumerate [M player.rs]; (2) the
#616 gate — the mid-corridor TouchOnce latches ch 0x8d on every route to the silo and nothing
on the disc clears it [D census], the switch refuses while the gate is on [M 0x1b02a8:22]; the
game-side live check (does the game's contact pass register #616?) is open. NEW un-ignored pin
`the_walked_mission_ends_at_the_launch_blast_until_the_pack_flight_is_ported`: the whole walked
mission (fresh load, the full route, no teleports) — the carried record completes, obj0
completes on the route, the chain re-enables Bond at the pit floor (f6071), ch 243 at +1543,
the fail at +451 after the pulse (the +~450 death-sequence window [M 0x19a6a8 448; L 454]),
obj0 stays complete and obj1 stays waiting through the fail. Session probes (scratch, removed):
the launch gallery is walkable end to end (the east lane x 11..19, the pit floor → z −192), the
chaser spawns at (2.39,−22.04,−149.17) AFTER the pulse (ch 0xf0 at script tick 2180 [D]) — so
even the fight leg cannot precede the pulse without the pack flight. All 8 un-ignored tests in
the file green (37.4 s).
