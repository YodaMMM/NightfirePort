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
