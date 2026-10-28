# WORKLOG

## drone-bump-collision-2

Branch s/drone-bump-collision-2 (from origin/s/drone-bump-collision @ 07a98925, merged origin/master @ fcf6f334). Task: the Jointy push (Player_Collision 0x194df0 → Collide_Jointy 0x1ec180:229-266, Drone::bump_capsule per NDrone2_Collision 0x140e90:59-124) runs un-gated in every stance on master's port and turns seven live tests red (mayhew_hold_live escort-hold, party_door_lock_07000007_live both, walkreal_07000007_live party walk, probe_07_route, cutscene_timer_pause_live, walkreal_07000011_live lower-floor load). Find which push the game does NOT do there — the pick-level gate (which drones are armed for the player's class-3 query: +0x464 = 0x800 under NDrone2_DoCollision 0x149ec8:11-34, the 0x20/0x200 flag arms, NearPlayerCollision 0x14b190), allies/escorts/civilians in scripted states, the radius per state — and port exactly that. All seven green plus drone_bump_live; full suite green.

### drone-bump-collision-2 — outcome (checkpoint, the arming fix left)

The pick-level gate traced end to end (Collide_Pick's cel-straddle scoping 0x1eb320:125,312-336 → PickObj's exclusion word +0x98 0x1eae80 (disasm: 0x400/0x100/0x200 EXCLUDE classes 3+0x12/2+0x11/5; the player's +0x7d8 is zeroed per tick, 0x194df0:88) → Intersect's glist/+0xfa&8 gates 0x1ebaa8 (disasm 0x1ebb7c-0x1ebbac) → Jointy's dual kind-0x800 0x1ec180:203-209 → the arming 0x140e90:66-101 under DoCollision 0x149ec8 whose 0x20 master arm DefaultInit/Enable set and death/fade/disable clear, the near arm fed by PreDroneControl's +0x154 slots 0x148db0:35-48). Landed the cel-straddle arm on the producer (bot cel ∈ the player's straddled set, find_cel stand-in) — drone_bump_live 2/2 stays green. probe_07_route unchanged (same-room adjacency straddles), so the seven reds need the arming fix: frozen/scripted bots' control pass suspended → +0x154 stale → not near → +0x464 = 0 → no push; the port arms them from live positions. Left with the live checks in the report (the gdbwatch of a frozen bot's +0x464/+0x154 settles it).

## drone-bump-collision-3

Branch s/drone-bump-collision-3 (from origin/s/drone-bump-collision-2 @ 69cd437b, origin/master already an ancestor @ fcf6f334). Task: port the arming exactly — a bot's capsule takes part in the player push only on ticks its own DoCollision returns 1, including the near-player slot Drone+0x154[i] written ONLY on the bot's control tick, so a bot whose control does not run keeps its old slot value [M]. Find which bots' control pass the game skips (cut-scene drone freeze, scripted holds, disabled drones) [M the control walk and the freeze paths]. Then the seven reds the push caused must be green: mayhew_hold_live, party_door_lock (x2), walkreal_07000007_live, probe_07_route, cutscene_timer_pause, walkreal_07000011_live — each either green or re-derived with a decompile cite; drone_bump_live stays green. Also port the capsule-bottom height from the anim height word ext+0xa0 [M DroneAnim_PreTransCallback 0x13b490:63-66] instead of the fixed 1.0. Full suite, total line quoted.

### drone-bump-collision-3 — outcome (the arming landed; three of the eight green; the 07 reroute attempted, mayhew left)

The arming is ported exactly: `Drone::kind_464` (ext+0x464) refreshed 0x800/0 on the bot's OWN control tick by
`Drone::arm_kind_464` — the disasm-checked port of `NDrone2_DoCollision` 0x149ec8:11-44 (master arm `enabled`; DSTATE
0xC/0x13 = HOSTAGE_SAVED/CIVILIAN_HIDING; ext+0x538 unmodelled→0 [S]; the near arm over `dist_to_player`; obj+0xFC =
`drawn_grace`; +0x570&2 unmodelled→clear [S]; the :42-44 fall-through → 1) — called after the UPDATE send + the flying
move, where `NDrone2_ControlSTANDARD` 0x1490f0:230 calls `NDrone2_Collision` 0x140e90:66-72/95-101. The +0x154 slot fill
MOVED from the batched opponents pass into the per-bot control pass head (PreDroneControl 0x148db0:35-48's own position),
so a bot whose control does not run keeps its last slots. `DefaultInit` 0x14b300:576 starts the kind 0x800. The freeze
zero [M Drone_CollisionHandler 0x140968:71-75] sits at tick_inner's head: `!npc_enabled || npc_system_disabled` → every
present bot's kind = 0. The producer (scene.rs `bot_capsules`) now gates on `kind_464 == 0x800` (+ the [S] corpse
exclusion, + the cel-straddle gate) instead of the live-distance test, and drops the whole list while `Sim::disabled`
[NEW M: Player_Update 0x193600:80-88 gates Player_Collision — the Jointy resolve's ONLY caller — behind the +0x94a
enable byte; Player_Disable(0) 0x195f70:12-18 zeroes the byte AND the player's own kind word +0x7d4]. `bump_capsule`'s
hip is the ext+0xa0 anim height (`hip_anim`) instead of the fixed 1.0 [M 0x140e90:68 feeds `r − hip` on both branches;
0x13b490:63-66 rewrites it clamp(animObj+0x5C − 0.02, 0.2)].

Verified green: **drone_bump_live 2/2** (the proven push keeps), **walkreal_07000007_live 2/2**, **walkreal_07000011_live
1/1** (the flight C band check raced a mid-fall tick the push's timing shift exposed — settle added, cited). Re-derived
with cites, NOT yet green: the 07 hall south-band anchors (party_door_lock ×2, cutscene_timer_pause, probe_07_route) moved
to the north lane — the crowd is armed [M the chain above; the party states write no 0x4F8 collision bits] and the walker
wedges at the 0.95 contact equilibrium — but the nav web still routes through the crowd band's nodes ((79.75,−50.45) et
al.) and a furniture hop wedges the lane at (78.6,−49.3); needs manual legs or a lane the nav serves. mayhew_hold_live
untouched this round: the flank legs wedge against the armed combat guards (state 0x29) — the fight choreography needs
push-aware legs. Scratch probe probe_drone3_11 (assertion-free) documents 07000011's flight C census (drone 10 armed at
2.40). Full suite NOT run in the window (the background runner dies between tool calls in this sandbox — re-confirmed;
the foreground attempt hit the 10-minute tool cap mid-build of the ~700 binaries). Crates touched: nf-game, nf-view.

Live checks for the main agent (carried from collision-2, now sharpened):
1. 07000007 fresh boot, the hall: gdbwatch a party bot's ext+0x464 while Bond stands 1.0 away in the crowd band —
   expect 0x800 (armed, control running). And Bond's own +0x7d4 while a cut-scene holds him (expect 0 — Player_Disable).
2. The party-crowd solidity: fieldprobe — walk Bond straight into a standing party guest mid-floor: stop at ~0.95
   (the push runs) or pass through? This settles the 07 reroute direction (the decompile says the push runs).
3. mayhew: gdbwatch an ambush guard's ext+0x464 during the fight — expect 0x800 (then the escort test needs push-aware
   flank legs, not an arming change).

## drone-bump-collision-4 (2026-10-28, in progress)

Branch s/drone-bump-collision-4 from origin/s/drone-bump-collision-3 (ba85aadf) merged with origin/master
(480aa3db). Task: the four remaining reds — party_door_lock x2, cutscene_timer_pause, probe_07_route (07 hall
south band, walker wedges at the 0.95 contact against the armed party crowd), mayhew_hold_live (flank legs wedge
on armed combat guards) — by giving the walker the real player's response to a crowd: wait for the line to clear,
step around the contact, or take the west mouth on measured timing. Every bot assumption cited [M]; never disarm
the bots. Suite total line to be quoted in the report.

### drone-bump-collision-4, session end (10:00 UTC)

The merge kept the [M] always-on Jointy push (master's airborne-only gate was [S]); the walker gained the
crowd answer (slide + doorway anti-trap with the mutual-block wait, opt-in `crowd_aware`). mayhew_hold_live
GREEN. The 07 trio still red at the west mouth: armed route walker parked ON the door's nav node
((79.75,−50.45) [D]), the pair mutually freezes, the 900-tick wait loses to the route dwell. Full anatomy +
live checks in docs/reports/drone-bump-collision-4.md. Suite started on the final tree (background runners
die between tool calls in this sandbox — the report quotes whatever landed).

## drone-bump-collision-5 (2026-10-28)

Branch `s/drone-bump-collision-5` = `origin/master` (ae4d2894) merged with `origin/s/drone-bump-collision-4`.
Continuing the 07 west-mouth wedge: the goal is the game's kind-8 route dwell + door swing/collision model at
the mouth, then the player's real answer for the four red tests (party_door_lock_07000007_live x2,
cutscene_timer_pause_live, probe_07_route) — wait the bot through, step aside, take the measured timing, with
cites. The push stays. Full suite to land on the final tree.

drone-bump-collision-5 research checkpoint: the west mouth is path3's RING [D links 0→6→7 door pair,
0→11→12→13→14→10→9→8→1→2→3→4→5] with door-flag nodes 0/6 (flags 0x2) straddling the free door at
(80.0,−49.9); dr22 boots on node7 east of the door, its kind-8 route = the 15-node ring [M InitAIPath
0x153720 chain], opens door3 itself (0x02→0x04 [M OpenDoor 0x1681f8]) and crosses west — then the port
freezes it at (79.48,−50.32) with the creep raced to leg 13→14: the probe_move stand-in [S drone.rs
3225] blocks the body while Route::follow's creep keeps advancing (the game's body never blocks — root
motion has no world gate, the only horizontal constraint is the bounds push [M 0x13b490/0x157bd0, see
walls-990 note]). Scratch probe probe_drone5_mouth.rs holds the anatomy.

drone-bump-collision-5 trap round: the walker's doorway anti-trap now plays the player's three beats —
wait the bot through (300 ticks, break early when she clears the 2.6 band or walks 1.2), break the
capsule contact radially (the mouth's wall absorbs the blocked component), then cross on a vision fan
(3 m world rays around the goal bearing, best mix of goal progress and capsule clearance, re-read every
tick; past = 0.8 behind her along the crossing). The gap-scan probe (probe_drone5_gap.rs) maps the
mouth: from the pin (79.20,−51.25) only the 45..75° world rays reach the hall and the open-leaf/jamb
blocks the north arc — while dr22 parks at (79.51,−50.32) the passable band is ~0.1 m and NOTHING
walks through; the fan correctly picks the direct east ray and the Jointy push cancels it (400 ticks,
zero displacement [the trap trace]). The trap also refreshed self.pos per tick (it was stale inside the
wait/fan loops — the aims computed from a frozen origin). walkreal_07000007_live's second test now
wedges at the same mouth (it does not opt into crowd_aware — the moved shared-RNG phase parks dr22 on
its route too). The verdict stands: the port's probe_move stand-in parks the armed guest mid-mouth; the
game's root motion is ungated [M 0x13b490/0x157bd0] so she walks through and the player waits — the
bot-side move model is the next round's fix, with the live checks in the report.

drone-bump-collision-6 start: branch = s/drone-bump-collision-5 (cbd1c5c1) merged with origin/master (17aa36ab), clean. This round ports the bot-side move model at the 07 west mouth per round-5's verdict — the game's route-walk root motion is ungated world-wise [M DroneAnim_PreTransCallback 0x13b490 / AINetwork_GetBoundsPushVectorForSphere 0x157bd0] while the port's probe_move stand-in parks dr22 mid-mouth — then the five reds (party_door_lock x2, cutscene_timer_pause, probe_07_route, walkreal_07000007 hall walk) go green behind it. Suite started in background; checkpoints every ~25 min; cutoff 14:57 UTC.

drone-bump-collision-6: the route walk now moves by the game's own constraint — root motion applied
ungated [M AnimFrameResolve 0x1c35f8:425-432] behind the boundary-flag per-axis clamp [M
DroneMove_SetBoundryFlags 0x14f140's four nav-only NDrone2_MoveTest 0x155330 probes; the zeroing in
DroneAnim_PreTransCallback 0x13b490:120-136 on the object-local step], the clamp skipped while the
door target +0xBA8 stands [M 0x13b490:120] — replacing probe_move for route-walk anims only (combat
keeps the walls-990 fold). dr22 crosses the west mouth (79.34,-50.27 at t100; the round-5 park is
gone) and completes her ring idx15/15 [probe_drone5_mouth]; the four red tests' mouth legs arrive.
Remaining red: the EAST door — two CIVILIAN_DOOR_GUARDs (0x31 [D type row mode 0x16 start 49], boot
(94.89,-48.00)/(93.66,-47.98)) shuffle in the doorway and the game-true Jointy pin leaves no passage;
the guards' zone reaction never fires in the port (near_armed_person needs the reaction-acked sight).
Live checks in the report own it. A test-side north-arc waypoint was tried and reverted (the nav
steer wedges at the same pin; not evidence-backed).

drone-bump-collision-7 start: branch = origin/s/drone-bump-collision-6 (988e9a49) merged with
origin/master (17aa36ab — already an ancestor), clean. This round: the east door's two CIVILIAN_DOOR_GUARDs
(0x31) — port the game's answer to a player walking up to them (the zone reactions' opponent model) and
settle the depenetration question, then the four east-door reds. Cutoff 16:30 UTC; background jobs die
between tool calls in this sandbox (setsid included), so everything ran foreground.

drone-bump-collision-7: the 0x31 zone read now runs on the game's opponent model — DoorGuardHandler's
+0x170 gate [M 0x1416d8:16-24] is the port's `bond_alive` fold [M NDrone2_FindOpponent 0x1424e0:100-120,
run every tick from ControlSTANDARD 0x1490f0:167; PlrStat_OkToUpdate 0x1993e0 is the mission-status gate],
replacing the [S] "the guard currently sees Bond" los stand-in that starved the zone ladder (the
reaction-acked sight +0x228 & 4 is NearArmedPerson's gate, not the zone read's). The zone reactions now
fire on real input — and the measured answer stands: the zones only FACE Bond (SetAngleToObj [M
0x160648:61-83]); the guards never translate (the idle shuffle ±0.15 [watch d10/d19]); the two capsules
OVERLAP each other (centers 0.97 < 2×0.95 [D boot (94.89,−48.00)/(93.66,−47.98)]), so the doorway is a
wall no steer rounds (east = the jamb pocket (95.43,−48.62), west = the d19 pinch); and the depenetration
hypothesis is refuted statically — the leaf swings away from the activator [M Door_SetupSwing side·swing]
and never touches the guards, bots resolve against neither the world nor the player in SP [M Collide_Jointy
0x1ec180:229-233], and the guards' capsules are [M]-proven armed (DefaultInit 0x14b300:60 keeps 0x4f8 bit
0x20, clears 0x200; DoCollision 0x149ec8; NDrone2_Collision 0x140e90) — so nothing pushes them for an
unarmed player. The E-spot reachability itself is the [L] question (the report's live checks).

## drone-bump-collision-8 (2026-10-28, UTC)

Branch s/drone-bump-collision-8, from origin/s/drone-bump-collision-7, merged origin/master
(clean). Scope: the 07 east door's Bond-vs-door-guard push. The -7 live checks are answered
(audits/live-07-eastdoor.txt): the guards do not relocate; Bond STANDS on the E-spot (94.6,
44.4, -47.4), 0.48 from d10's centre, and stays 120 frames — the port pins him at the 0.95
r-sum contour; placed between the guards the game walks Bond out NORTH. Suspects [M]:
Collide_Jointy 0x1ec180:203-281's radius source, NDrone2_Collision 0x140e90:66-101, the anim
height word, whether the 0x31 guards are armed. Port the game's numbers, then green
probe_07_route, party_door_lock_07000007_live (x2), cutscene_timer_pause_live,
walkreal_07000007_live's hall walk. Work in progress; will update as steps land.

drone-bump-collision-8 checkpoint (18:05 UTC): the fix landed — the at-post CIVILIAN_DOOR_GUARD
(0x31) state no longer joins the player push set (scene.rs bot_capsules filter), cited to the
-7 live answers [L: Bond stands on the E-spot 0.48 from d10's centre, 120 frames, no push; the
static chain [M 0x1ec180/0x140e90/0x149ec8/0x1ebaa8] reconciles everywhere else]. Measured on
the rebuilt tree: cutscene_timer_pause_live GREEN (1/0), walkreal_07000007_live GREEN (2/0),
party_door_lock 1/2 (the sister now fails LATER — the #244 use-key assert at :272, past the old
wedge), probe_07_route reaches the doorway (94.86, −48.03) but the route's post-E legs wedge on
walker pathing (the crowd_aware steer still repulses off the guards — the same [L] contract
needs the steer exemption; the E use-zone is the 0x48 trigger at (94.3, 43.7, −45.9), not the
door face). Next: the walker steer exemption + the E-leg use-zone re-derivation, then the
remaining legs, then the full suite.

drone-bump-collision-8 final (18:07 UTC): the steer exemption landed (walker.rs crowd repulsion
skips the 0x31 at-post guards) — no change to the two remaining legs. Final counts on this
tree: cutscene_timer_pause_live 1/0 GREEN, walkreal_07000007_live 2/0 GREEN, party_door_lock
1/2 (the sister's #244 use-key assert — the 0x48 use-zone trigger sits at (94.3, 43.7, -45.9),
~1.5 north of the door face), probe_07_route red at the post-E hover wedge ((93.40, 44.26,
-46.32), grounded=false, floor 0.23 below — not a guard push). Report:
docs/reports/drone-bump-collision-8.md — two live checks own the open arms (the at-post push
killer; the east door's real use zone).
