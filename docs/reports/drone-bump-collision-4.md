# drone-bump-collision-4 — mayhew_hold green through the crowd slide; the 07 trio's red root-caused to the west-mouth doorway (armed route walker + swing cell), the walker toolkit landed, the trio still red; the full suite did not fit the window

Branch `s/drone-bump-collision-4` = `origin/s/drone-bump-collision-3` (ba85aadf) merged with `origin/master`
(480aa3db) [commit c146c6f0], then 12b7429f / 9e59b083 (this round). WORKLOG entry `drone-bump-collision-4`.

## What was done

### 1. The merge kept the decompile-backed push — and that is the story of the remaining reds

`origin/master`'s player.rs still carries the airborne-only push gate (`if !ctx.bot_capsules.is_empty() &&
!self.grounded`, self-marked `[S] unverified` there: "master's green walks path Bond THROUGH ally tangent
zones"). The -3 round replaced it with the game's rule — the Jointy push has NO stance gate [M
Collide_Jointy 0x1ec180:229-266: the separation vector is ADDED to the player's position slots; tangential
motion survives it] — and the merge took -3's side (verified on the merged tree: player.rs:1854's comment,
scene.rs:2943's `.filter(|x| x.kind_464 == 0x800 && !x.corpse)` producer). Consequence: with the [M] push
always on, the party crowd is SOLID, and every 07 test whose walk was written under master's walk-through
behavior now meets an armed crowd at the hall's west mouth. The task's framing is exactly right: the push
stays, the walker needed the player's answer.

### 2. The walker's crowd answer (opt-in `crowd_aware`, default false — every other caller bit-identical)

- **The crowd slide** (common/walker.rs `walk_to`'s steering): armed bots (kind word +0x464 == 0x800 [M
  NDrone2_Collision 0x140e90:95-101; the freeze zero 0x140968:71-75]) inside the 2.2 band repulse the steer
  direction (goal attraction + bot repulsion, forward never zeroed inside the band). The earlier bypass-aim
  variant stalled — `err > 1.2 → forward 0` turned in place inside the push's radial arm.
- **The doorway anti-trap** (the stuck ladder's first branch, ≤3 fires per leg): an armed route walker at its
  nav node just beyond a door and Bond's capsule freeze EACH OTHER — the bot's state-0x16 walk (the civilian
  walk tuple [M drone.rs 0x14..=0x16]) tests its move against Bond's capsule and stands while Bond claims
  the doorway, while the push pins Bond at the 0.95 contact inside the leaf's sweep cell. The player steps
  BACK out of the cell (reverse 70 ticks), then waits up to 900 ticks for the bot to walk its route leg (the
  guests DO walk their kind-8 routes [M 0x1728c0 → 0x154190 → 0x15d860; walkreal_07's zone test]), then
  follows behind it.
- Scratch probe `probe_drone4_crowd` (assertion-free, --nocapture) — the wedge anatomy at the 07 west mouth:
  bot 22 armed (`enabled 1, kind 0x800, state 0x16, mode 0x9`), STATIC at (79.42,−50.32) for 1200+ sampled
  ticks — that is the nav-dump node (79.75,−50.45) [D] beside the FREE door at (80.0,−49.9) (state 0x4
  open). Bond pinned at (79.19,−51.21), dxz 0.92 from the bot — the 0.95 contact — standing ~1.1 above
  `floor_below 43.0` (on/inside the open leaf's box), stone still across the stuck ladder's strafe/jump
  ladder. From the north the hall is walled (`segment` stops Bond at (79.19,−45.7) — the -3 report's own
  finding); the mouth is the only entry.

### 3. The 07 trio re-anchored on walkreal's proven line

The -3 round's north-lane anchors (z ≈ −46.6) are WRONG (its own "What's left" item 1): the nav web's hall
nodes sit in the crowd band and the lane wedges on the hall's west wall at x ≈ 80. party_door_lock,
cutscene_timer_pause and probe_07_route now cross on walkreal_07000007's exact south-band line (west mouth
(76.6,−50.7) → floor (83.1,−50.5) → east (93.3,−50.4)), with the arming-chain comment carried.

## Verification (what ran — all nf-view, --test-threads=1)

- **mayhew_hold_live 1/1 GREEN** — the task's Mayhew ask: with `crowd_aware` the flank approach legs slide
  around the armed combat guards instead of wedging; the escort hold, the plate, the kills and the 0x99
  release all pin as before (finished in 119.85 s).
- **Regression proof (the push machinery untouched, the walker opt-in): drone_bump_live 2/2 GREEN,
  walkreal_07000011_live 1/1 GREEN** (the flight C settle keeps), walkreal_07000007_live's zone test GREEN
  (the bots-walk proof; its hall walk test is red below — same mouth).
- **Still RED (the shared west-mouth wedge): party_door_lock_07000007_live 0/2, cutscene_timer_pause_live
  0/1, probe_07_route (panic in the manual leg)**. With the slide + anti-trap the wedge moved OUT of the
  swing cell (Bond backed to (77.24,−51.58)) but the re-approach re-freezes the pair — bot 22's route dwell
  at the mouth node outlasts the 900-tick wait window, three trap fires burn, the leg gives up. The E the
  hall door leg then re-traps identically.
- **The full suite did NOT fit the window** (two sandbox resets + a from-scratch build consumed it;
  background processes die between tool calls in this sandbox — re-confirmed). The suite runner was started
  on the final tree; whatever its state at commit time is quoted below. Crates touched: nf-view only (tests
  + the common walker) — nf-game/nf-formats/nf-audio are untouched by this round's commits (the merge's
  content was verified by the runs above).

## What's left, in order

1. **The 07 trio needs the mouth's route-dwell answered, not more walker code**: the bot's kind-8 route
   dwells at the door node long enough to freeze any approach. Either the port's route-dwell timing for the
   party guests diverges (the game's CivilianMission 0x15d860 node-wait model — check the dwell word), or
   the real game's bot walks on a phase the port shifted (the shared-RNG phase moved with the merge's
   props.rs `channel_kills` rewrite — any draw-stream difference re-phases every bot).
2. The suite total line on the merged tree.
3. probe_07_route's post-E plate-free legs re-validated once the walk reaches the E door.

## Live checks for the main agent

1. **The west mouth, fresh boot 07000007**: when Bond reaches the hall's west mouth (the free door at
   (80.0,−49.9)), where does the armed hall bot stand? gdbwatch its ext+0x20/0x30 (position) — expect the
   port's parking spot (79.42,−50.32) or a walking bot. This settles whether the port's route dwell is the
   divergence.
2. **The party-crowd solidity** (carried from -3, still the direction-setter): walk Bond straight into a
   standing party guest mid-floor — stop at ~0.95 (the [M] push runs; the -3 chain) or pass through?
3. **A frozen bot's kind word** (carried from -3): inside the control-taking scene, gdbwatch a hall bot's
   ext+0x464 (expect 0x800 — the scene fires no event 5) and Bond's own +0x7d4 (expect 0 — Player_Disable).

## Usage

- tokens/context/cost figures: not visible to me (no token or context-window readout is exposed to this
  session; nothing to copy exactly).
