# level-14-tripbomb-real-route — the report

## Status: in progress (the route is mapped and the take is proven; the last 4 u into the rack lane is the remaining work)

This sandbox restarted mid-task (the repo was gone — setup redone: clone, `work-slim.7z`, the ISO,
rustup, py7zr). Two checkpoints are pushed on `s/level-14-tripbomb-real-route`.

## What is done and how it was verified

**The pickup placements are world coords in the main map.** The audit's #1341/#1342 are
placements #1015/#1016 of map 54 (the main map of 55): params `(0:0)` = type WEAPON, `(1:58)` =
0x3a, `(2:5)` = rounds [D the parsed archive, `probe_tripbomb_pickups.rs`]. No map offset exists:
the sim spawns the pickups at **(-32.187, 1.409, 31.122)** and **(-32.187, 1.409, 35.858)** — the
audit's coords to two decimals [L the sim dump].

**The rack wall.** The pickups hang on the armoury rack wall at x ≈ -32.0, the low row under the
mounted HKOICW rifles (#1088-#1096 at y 1.70/2.34, z 31.04..36.21 [D placements]). The lane in
front runs z 30.7..36.4 at floor 0.80; a 1.31 shelf sits at x -32.3; the take radius
(bounds.radius 0.242 + 1.1 [M Pickup_Update 0x18e148]) is reachable from the lane floor and from
the 1.45 crate at the rack front.

**The take works through the real touch path.** A teleport-scout (scouting only — the live test
has none) walked Bond to the rack front; `Pickup_Update`'s touch fired and `drones.taken` gained
`Taken::Weapon { weapon: 0x3a }` with the pickup tombstoned [M Pickup_Update 0x18e148 →
Pickup_Handler 0x18e650]. The same touch cluster took the rack's HKOICW and satchel ammo, exactly
as the game's walk-by does.

**The route is walked for real up to the armoury branch, every run.** The live test
`walk_the_tripbomb_route_pick_up_place_and_shoot_it` (appended to
`crates/nf-view/tests/walkreal_07000014_live.rs`) runs: the east route + the SP #276 hand-back
(`walk_infiltration_to_handback` — the SkyBridge Q-Claw crossing, the trench, the shaft reel, the
vent drop, all this file's pinned chain), the descent to the facility corridor (the shared west
route's first block, duplicated verbatim so the shared fn keeps its shape), then the nav web's
**77-node south loop** to the armoury branch — the same haul the passing west-route walk covers
(72 nodes to the west corridor); the branch splits at (-30.34, 0.40, 23.13). The loop completes in
~45 s of test time. Every tick is real `Input` through `step_bond`; no teleport after the load.

**The gadget legs are written against the pinned rules.** After the take: the real switch request
(`Input.switch_to = 0x3a`), the real fire sticks it on the rack wall [M Bullet_CollisionHandler
0x12ce80 +0x70 & 0x100 world-hit arm — the family fix_satchel_tripbomb_live pins], the retreat
legs walk Bond ≥ 12 u off the 200-u beam line [M Bullet_update case 3], the shooter spot is picked
from the level's own collision (floor + clear eye→body segment ≥ 12 u, then WALKED to), and the
P2K (0x02, owned from the level start [M Inventory::level_start]) shot must detonate through
`Bullet_handle_object_destruction` 0x12cd98 with the 0x0600_0052 blast and the beam deleted — the
exact rule tripwire_shot_live pins, reached here the way a player reaches it. Final position and
the shot line are logged.

## What is left — the last 4 u into the rack lane

The lane's west entry is a crate band (x -36.6..-35.4): elevated crates (tops 3.5), a 1.45-top
middle crate at x -36.2 spanning z 33.3-34.0, a ~1.2 sill at x -37.2. The only body-clear thread
from the west is **z 33.0-33.25** [L the ray grid in `probe_lane_gap.rs`]; the steer-walker's
wedge hop climbs the sill/crates instead of threading it (three leg tunings all end on the 1.45
crate top — Bond stands there grounded, he just won't walk the thread).

**The promising next step (found at the end of this session): the lane's SOUTH entrance.** The
lane extends south past z 30.7 to z 28 at x -33 (floors 0.0-1.1). A real-input walk from the
loop's node (-30.34, 0.40, 23.13) via (-31.5, 0.4, 26.5) reached **(-33.08, 0.68, 28.60)** — the
lane's south mouth. A **1.10-high crate at ≈(-33, 29.4)** then blocks the north run (the walk
stalled at (-32.16, 0.65, 29.10)). Next session: thread around that crate (probe floors at
x -34..-33.5, z 29-31; if the west side is clear, two waypoints (-33.7, 0.7, 29.2) → (-33.6, 0.75,
30.9) reach the lane floor at z 31+ and the proven legs run unchanged: (-33.3, 0.8, 33.3) → the
rack front (-33.2, 0.85, 35.7) → the take → place/retreat/shot).

Then: run `tripwire_shot_live`, `interactables_07000014_live`, `walkreal_07000014_live` (the new
test included) and the full suite; push; the driver merges.

## Test files on the branch

- `crates/nf-view/tests/walkreal_07000014_live.rs` — the appended live test (compiles; runs green
  to the armoury branch; the take assert is the next gate).
- `crates/nf-view/tests/probe_tripbomb_pickups.rs`, `probe_tripbomb_area.rs`,
  `probe_lane_gap.rs` — the passing diagnostic probes (the placement resolution, the floor grid +
  the take scout, the crate-band ray grid + the lane-south legs). They are diagnostic aids for the
  next session; drop them at merge time if unwanted.

## Live checks for the main agent

None this session (no GPU work; all evidence is [M]/[D]/[L]-probe). If useful for the lane
question: a PCSX2 stand at (-33.0, 0.7, 28.6) walking north — does the real game's Bond pass the
crate at (-33, 29.4) on its west side, and what pose does the player hold at the rack front
(x -33.2, z 35.7)? That settles the entry the port must thread.

## Usage

not visible to me (no token, credit or context-window figures are shown in this environment).
