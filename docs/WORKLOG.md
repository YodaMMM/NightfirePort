# WORKLOG

## drone-bump-collision-2

Branch s/drone-bump-collision-2 (from origin/s/drone-bump-collision @ 07a98925, merged origin/master @ fcf6f334). Task: the Jointy push (Player_Collision 0x194df0 → Collide_Jointy 0x1ec180:229-266, Drone::bump_capsule per NDrone2_Collision 0x140e90:59-124) runs un-gated in every stance on master's port and turns seven live tests red (mayhew_hold_live escort-hold, party_door_lock_07000007_live both, walkreal_07000007_live party walk, probe_07_route, cutscene_timer_pause_live, walkreal_07000011_live lower-floor load). Find which push the game does NOT do there — the pick-level gate (which drones are armed for the player's class-3 query: +0x464 = 0x800 under NDrone2_DoCollision 0x149ec8:11-34, the 0x20/0x200 flag arms, NearPlayerCollision 0x14b190), allies/escorts/civilians in scripted states, the radius per state — and port exactly that. All seven green plus drone_bump_live; full suite green.

### drone-bump-collision-2 — outcome (checkpoint, the arming fix left)

The pick-level gate traced end to end (Collide_Pick's cel-straddle scoping 0x1eb320:125,312-336 → PickObj's exclusion word +0x98 0x1eae80 (disasm: 0x400/0x100/0x200 EXCLUDE classes 3+0x12/2+0x11/5; the player's +0x7d8 is zeroed per tick, 0x194df0:88) → Intersect's glist/+0xfa&8 gates 0x1ebaa8 (disasm 0x1ebb7c-0x1ebbac) → Jointy's dual kind-0x800 0x1ec180:203-209 → the arming 0x140e90:66-101 under DoCollision 0x149ec8 whose 0x20 master arm DefaultInit/Enable set and death/fade/disable clear, the near arm fed by PreDroneControl's +0x154 slots 0x148db0:35-48). Landed the cel-straddle arm on the producer (bot cel ∈ the player's straddled set, find_cel stand-in) — drone_bump_live 2/2 stays green. probe_07_route unchanged (same-room adjacency straddles), so the seven reds need the arming fix: frozen/scripted bots' control pass suspended → +0x154 stale → not near → +0x464 = 0 → no push; the port arms them from live positions. Left with the live checks in the report (the gdbwatch of a frozen bot's +0x464/+0x154 settles it).

## level-14-vent-drop-2

Branch s/level-14-vent-drop-2 (from origin/s/level-14-vent-drop @ ed167832, merged origin/master). Task: implement the Countdown Part 1 vent drop that level-14-vent-drop decoded but never wrote (touch 0xd9 -> OR 0x7e -> the SP switch; the case-6 capture -> Script_Free's Player_Enable), scoped as that report says, then walk walkreal_07000014_live::walk_the_infiltration_to_the_delta_exit legs 2..6 to the exit on real input; un-ignore on green, else back to #[ignore] with the exact blocker.

### level-14-vent-drop-2 — outcome
