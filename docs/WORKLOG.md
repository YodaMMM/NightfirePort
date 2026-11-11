# WORKLOG

## level-14-claw-route-retune
Started 2026-11-11 15:47 UTC. Branch s/level-14-claw-route-retune cut from origin/master 0306b19c5.
Task: un-ignore walkreal_07000014_live::walk_the_infiltration_to_the_delta_exit_real_damage and
its _agent sister — re-tune both Countdown infiltration walks with real input after the claw
wall fix (the SkyBridge crossing is a stand-in, the claw bolt dies on the duct wall; the
descent lands at (29.63, 0.65, 29.25) instead of the leg-2 start (29.99, 0.61, 38.00), and the
Agent run falls short on the pit band's east jump at (46.34, -1.78, 53.19)). First verify the
claw: does the game's claw reach the SkyBridge grapple point from that stand, or is the duct
wall really in the way [M the claw's ray and the grapple target]. Never lower damage or health,
no top-ups. Run walkreal_07000014_live and interactables_07000014_live.

Work log:
- (15:47) branch created from origin/master 0306b19c5, WORKLOG entry, checkpoint pushed. Suite
  fast runner starting in background next.
Work log (continued):
- (16:10) The claw verification, decompile-side: the latch is Bullet_Delete 0x12e660 — the hit
  object's kind byte +0xff must be 0x51 ('Q') else the bolt just dies, then
  Player_SetGrapplePoint 0x195da0 stores the bolt's +0xc0 position and ChangeSubState(2); the
  bolt's death is Bullet_CollisionHandler 0x12ce80 — a solid contact sets bullet +0xf4 = 2 and
  state 2 sets +0xf0 |= 0x10; the reel is Player_SSGrapple 0x192860 (pull 0.05 → 1.5/tick cap
  SP, hang at dist < 2.0, release on action 9). From the duct stand (40.8, 2.40, 65.5) the ray
  to #658 (45.2888, 5.0961, 62.9362) crosses the wall plane x 41.53 at (41.53, 2.84, 65.08) —
  inside the duct east wall's band — the duct wall really is in the way; the crossing stand-in
  stays.
- (16:12) The plain real-damage walk's ACTUAL blocker on this tree is not the descent (it lands
  (29.51, 0.65, 37.77), in the pin box) — it dies at tick 4104 to room guard #284 (50 hits for
  100.0, no fight machinery at d1) during the landing drop. Two real-input laws added, both
  gated !tops_up(): the room guard kill from the machine-room lane start (the landing perch
  shot cannot land — the slab west face, top 2.37, eats the floor-level ray; measured), and the
  pre-cut hall clears (the first run's generator blast death).
- (16:40) The plain walk then reached tick 33324 (the box A cut) and died to a WORLD blast at
  (-88.7, 0.9, 32.6), radius 10.0, 500 base, d 2.4 [L NF_DMG_TRACE] — the generator overhead.
  Two more runs (pre-cut clears ×1 and ×2) died the same way (33336/33348, identical fight
  counts — the clears find nothing within 24). The _agent sister walks out green (22633 ticks).
  State: _agent UN-IGNORED, plain still #[ignore] with the measured blast + levers in its note.
