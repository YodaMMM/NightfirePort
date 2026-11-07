# 1b-zero-g-floor — the float wedge fixed by the two-phase route; consoles 7/8 latch; the six other reds probed: all route, none snap

Branch `s/1b-zero-g-floor` (from `origin/s/aim-console-phase`, already up to date with
`origin/master`). The driver's correction confirmed in full: the wedge is the WALK's, not the
game's. No zero-g or collision code changed; the [L] 08 pins and the 0x30000000 fix are kept
(untouched from the aim-console-phase lineage).

## 1. Why the walk's thrust cannot leave the pocket — measured, then routed around

**The comment fixed first** (the task's correction): `float_to`'s sag-guard comment claimed
"walk-mode pins it and the thrusters are dead" — wrong on both counts. The driver's no-input
measurement (PCSX2 and the port settle identically at the pocket and float-hold; port substate
stays 8) and the decompile agree: `Player_ZeroG` 0x192ab0 has no ChangeSubState, so zero-g never
ends by itself [M], and the lift the float commands IS the game's real vertical thruster — the
(action 7 − action 8) analog [M 0x192ab0:136-137 reads Input_Action actions 7 and 8; the port's
`Input.lift` eases into BL+0x24 and the move rides the +0x10 quad's y lane, 0x192ab0:92-159;
M player.rs the zero-g move]. The task's "check that the input used is the real lift input":
checked, it is.

**The pocket's shape** (the `[pocket]` one-shot ray dump, added to the 1b test, fired at the
wedge (44.76, −8.66, 1.69)): straight up hits at **0.83** — the bank's underside; every up
diagonal hits at 0.79-1.75 — the underside SLOPES DOWN toward +x/+z; +x hits at **4.73** — the
riser; −x, ±z and down are clear (the pocket hangs under the bank over open depth). The wedge is
entered from the cross-room descent: the [gate] trace shows the float at (38.55, −8.10, 1.24)
flying +x toward the station band with the head at y −6.9, meeting the sloped underside, and the
push settling him into the pocket.

**Why no thrust leaves**: under the port's radial-push collision model the sloped underside's
push cancels the whole move — the position held to ±0.05 through 3000+ ticks of full climb
(lift 1.0) plus sustained back thrust, and a first escape-phase implementation (five short
verified-clear hops) found no flyable hop at all. This is the known port divergence the tree
already documents: the game's own radial push tail is provably dead [M Intersect_CylGeom
0x109da0:447 — the HITTEST+0x50 lifetime-DeltaP guard boots (0,0,0) [L ee_start_*] so no contact
passes it; the game CLIPS the move at the face instead, which slides], while the port keeps the
radial push. Under clip a player's up-and-back thrust slides out; under push it freezes. The task
forbids collision changes, so the fix is the route.

**The fix (the report's own next lever — the hysteresis-free two-phase goal)**, file-local to
`walkreal_0700001b_live.rs`: while the goal is a LOW station (y < −7: the 7/8 bank end) and Bond
is still horizontally far (> 1.2) and above it, the fly target is the CROSS COLUMN TOP — the
goal's own x/z at goal.y + 5 — so the crossing rides ABOVE the bank's top (the screens' face
tops at y −5.21 at x 38.74) and the descent runs the STATION'S OWN column, which float-holds
[L the driver's probes: (46, −8.3, 0) and (46, −8.7, 0) hold in both PCSX2 and the port]. The
progress measure and the stall detector read the fly target, so the crossing cannot trip the
ban; the clamp releases within 1.2 u of the column and the plain descent takes over. Measured:
the wedge never forms; the walk flew the crossing and latched.

## 2. The 1b walks re-run: consoles 7 and 8 latch (all eight latch)

`walk_arrival_to_drakes_defeat` (+ the real-health sister): the consoles latch in deadline order
3, 4, 5, 6, 8, 7, 1, 2 — the latched list with frames: **[(3, 4017), (4, 4425), (5, 4629),
(6, 5043), (8, 5511), (7, 5943), (1, 6406), (2, 6537)]**, health 100.0 armour 44.0. Consoles 7
and 8 latch — item 2's goal.

**The next gate, newly reachable and not passed**: `MissileDeploy [3, 3, 2, 2, 3, 2, 2, 3]` —
consoles 5, 8, 1, 2's missiles armed. The deadline math from the run: the p0 scripts start at the
launch order (~frame 2800, bounded by the disarm/arm split: console 4's disarm needs S > 2626,
console 5's arm needs S ≤ 2830), so the state-2 reads land at S+1799 (3/4/5), S+2399 (6/8) and
S+3299 (1/2/7) — console 5 missed by ~30 ticks, 8 by ~312, 1 by ~307, 2 by ~438. The budget
levers for the next window: console 3's phase setup ate ~1200 ticks (the clip recharge to the
48-round burst gate + the aim convergence + the first-hold playback); the 7→1 cross-room leg
costs 463 and 7 latched at 5943, leaving 156 before 6099 — impossible. So the phase needs ~800
ticks off the front (pre-charge the clip during the first wave's last stretch or lower the burst
gate to 36 = the three holds' exact cost), and the 7→1 flight needs the two-phase treatment too
(it is a y < −7 → y −2 crossing, the clamp's low-goal condition does not fire on the arrival
station but the DEPARTURE is from the pocket-adjacent band). The whole 1b binary: 15 passed /
2 failed / 2 ignored (63.3 s pre-fix counts; post-fix the two drakes walks run ~300 s each to the
new assert — the other 15 tests do not touch `shoot_consoles` and are unaffected).

## 3. The `[ring]`/`[gate]`-grade probe on 08, 0c, 14 ×3, 16 — all route, none snap

Every red re-ran whole, one at a time, and each death ledger + in-world state was read. **None
of the six is an aim-snap fault** — consistent with the aim-console-phase evidence that console
8's snap was innocent. The classifications:

- **07000008 real_damage_00agent — ROUTE (the station legs' hurt volume).** Death tick 28756 at
  (111.06, 13.53, −179.71): 78.0 in 3 from the world (live wire / blast / hurt volume) after
  chip exposure at d 84.8-92.3 (#17/#9/#10, weapon 0x0); 10 return-fire fights ran; the drone
  chips total 22.0. The fights answered; the killer is the volume — the station-legs re-pin the
  aim-console-phase report predicted (its drone.rs comment: "the real-health route cost is
  re-pinned with the 08 station walks"). Not an aim fault.
- **0700000c real_damage — ROUTE (the far-sniper drain).** Death tick 13461 at (5.11, 12.15,
  −207.80): 102 hits for 100.0, 9 return-fire fights in 13.4k ticks; the finishing dealers #39
  (0x5000093, weapon 0x1e, 16.0 at d 61.3) and #20 (0x5000087, 0x1e, 11.0 at d 64.3) plus the
  #33 chip line at d 90 — ALL left at full hp 10: the walk's fights never once reached them (the
  plain 20 u shooters window vs d 61-90 snipers — the same shape 03's red had). Route/exposure.
- **07000014 ×3 — ROUTE (the nav stick, exactly where the driver said).** All three reds
  (`the_e_use_switch_fires_the_sabotage_cascade_and_opens_the_west_door`,
  `walk_the_infiltration_to_the_delta_exit`, `walk_the_tripbomb_route_pick_up_place_and_shoot_it`)
  stick at the same leg: "the corridor south to the leg-2 spot", 6000-tick cap, bond
  (30.33, 0.55, 35.44), target (30.09, 0.65, 38.73) d 3.30, `segment_to_target = Some(0.170)` —
  the straight line clips at 0.56 u out. What stands there: the corridor's west wall beside the
  kind-2 door at (32.0, 2.1, 35.8) — the door itself is OPEN (state 0x4, pos01 1.00) but the
  leg's straight line runs 1.7 u WEST of the doorway, into the wall face; a guard (#5, state
  0x5c) stands off the path at (33.22, 1.20, 32.86) d 3.88 and is not the blocker. The lever is
  file-local and cheap: an intermediate waypoint through the open doorway (~x 31.8, z 36.3)
  before the leg-2 spot, or re-anchor the leg's approach on the door line.
- **07000016 chasers real_damage — ROUTE (the point-blank melee swarm).** Death tick 14656 at
  (−13.10, −22.00, −170.15): the chasers closed to d 0.8-1.3 (#12 0x12, #15/#14 weapon 0x14) and
  finished Bond in the always-hit band; 94 hits for 100.0, only 5 return-fire fights (#8's 24.0
  was answered — he shows hp 0). The fights' back-off could not hold three swimmers off at once.
  Route/melee-space, not aim.

## What this branch does to the walks (the s/stream-batch question)

Two round-landing deltas ride this lineage: (a) the traced snap refactor is behavior-preserving
(aim-console-phase's battery, unchanged here); (b) the snap-aware planner (`snap_shot_ok` gating
`level_station`, the default station and the fire tap) changes WHERE the 1b console rounds land —
they now only fire from stances whose crosshair ray's snap pick stops on the screen's own sphere
— plus this task's two-phase crossing, which is movement-only but shifts the console phase's
timeline ~1500 ticks later than the pre-wedge runs (the latched frames above). No other level's
walk touches this code: 08/0c/14/16 binaries re-ran whole this session with their pre-existing
red counts unchanged (5/1, 2/1, 6/3/1, 13/1/2). The 1b pacing work (the MissileDeploy gate) will
move rounds' TIMING further; joining `s/stream-batch` makes sense once the 1b drakes walks are
green end to end so the batch picks up a stable timeline.

## What's left (next window)

1. 1b: the MissileDeploy pacing (the ~800-tick front load: the burst gate 48→36 or the clip
   pre-charge, the cross height `min(goal.y + 5, pos.y + 0.5)` so a mid-descent pickup doesn't
   climb back, and a faster 7→1 crossing) — the deadline windows are derived above.
2. 14: the doorway intermediate waypoint for the three reds (the geometry is measured above).
3. 08: the station-legs re-pin (the hurt-volume approach); 0c/16: the far-shooter and melee-swarm
   route levers (the 03 walk's hurting_first arm and the 06 sweeps are the measured shapes).
4. The full `cargo test` suite and the `npc_doors_live`/`fix_guard_misc_live` binaries: not run
   this window (no `drone.rs`/game-code change landed on this branch).

## Live checks for the main agent

1. **The pocket's underside slope** — at the wedge (44.76, −8.66, 1.69) with the debugger:
   confirm the face above is the bank's underside and read its normal; the port's push model
   cancels slope-parallel thrusts there while the game clips — one screenshot of Bond at the
   wedge and one after a sustained up+back thrust in PCSX2 (expect: the game SLIDES out −x, the
   port does not) settles the model divergence claim.
2. **The missile p0 scripts' start frame** — watch ch 19's rise and the first missile script's
   state-1 tick counter: the disarm deadline math above assumes S ≈ 2800 (bounded 2644 < S ≤
   2830 by the measured disarm/arm split); the exact S turns the pacing budget into arithmetic.

## Usage

not visible to me — this session exposes no token, credit, context-window or cost figures to copy.
