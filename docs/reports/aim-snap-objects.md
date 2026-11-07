# aim-snap-objects — the crosshair aim snap: objects and glass

Branch `s/aim-snap-objects`, cut from `origin/s/panel-aim-snap` (08ace8edf). Four commits: the start
(WORKLOG), 6b049b4e0 (the object side of the ray), 05cac0260 (the script entities leave the ray), and
faaac6ae0 (the aim_pick filter's final home on CollisionWorld).

## What the research established

- **The snap pipeline [M]**: `Collide_RayIntersect` 0x1ec690 builds a **type-0x201** hittest (the
  segment), feeds it the SHOOTER'S CEL (param_3 = *(obj+0x20)), runs `Collide_Pick` 0x1eb320 (the cel
  walk: the built-sphere broad phase per object [obj+0x80..0x8c], the +0x94 & 0x20 skip), then
  `Collide_Intersect` 0x1ebaa8's 0x201 arm per hit — the segment vs the object's OWN glist triangles
  (obj+0xD8 → HITTEST+0x88), back-face-culled — first hit wins (param_9 & 0x10). The snap replaces the
  aiming point; the round is re-aimed from the muzzle midpoint [M Player_WeaponInitBullet 0x1a4bf0:154-164].
- **The glass mask is NOT inverted**: mask 0x20a carries bit 2 = glass transparent; 0x208 (records with
  +0x6C & 0x800 — the Laser 0x4E's 0x900 [L ee_start_05.bin]) stops on glass. The port's `glass=true`
  (the level query's pass convention, [M 0x1ea398:216-229]) matches; the caller's
  `flags6c & 0x800 == 0` is right as written.
- **The level ray is CEL-SCOPED, and the doors are not in it** [L the probe + the live lines]: through
  the 08 window's non-portal hole the ray never reaches the far room; and the shutter doors (kind 0xdb)
  carry NO 0x8000 cel flag [M parsemap_block_map_data_static 0x1d1998:92 — they never had a static cel;
  the port's stat fold is the deviation]. The measured probe: the camera ray's x at the shutters
  (139.562) is INSIDE the wood band (139.44..139.62) yet the live straight rocket pierces the pane —
  the game's ray saw neither the shutters nor (through the hole) the far wall, so the aim stayed the raw
  256-out point and the round's own line o→P256 crossed the pane span (x 139.72).

## What I changed

1. **`segment_aim_scoped`** (world.rs): the cel-scoped flight query with a per-instance
   **aim_pick filter** — stat runs owned by placements WITHOUT the 0x8000 cel flag are skipped for the
   aim ray only (the flight's door stops ride the unfiltered sets, untouched). The aim_pick vec is built
   in the stat loop from each instance's placement flags.
2. **The drones in the ray**: each alive drone's hit boxes [M AnimGetCollData] via `damage::ray_box`,
   entry t only.
3. **The script entities are NOT in the ray** (commit 05cac0260): the live lines say the game's snap ray
   passes them (the 08 shutters; the 1b walkway beam clears the panels whose r 4.77 spheres swallow the
   line [L the breakpoints — no hit entry without a triangle touch]). The port's stream-matrix posing of
   world.shape triangles is provably mis-framed (the 14 wires' posed AABBs landed at x +42 [D the probe];
   the walkway's phantom panel plane). Their real contacts stay with the flight's own object tests
   (the fusebox wires' posed gfx bounds, the consoles' held spheres), which the raw aim lines reach.
4. **rocket_origin_live updated** to assert against the snapped point (the task's instruction): the
   sister now builds the Data view, snaps `head + look·256` through `aim_snap`, and measures the flight
   line at the snapped distance.
5. The glass argument verified not-inverted; `+0x6C & 0x800` checked against the records (0x2d = 0x1a200
   no bit → the ray passes the pane; 0x4e = 0x900 → stops) — the caller was already right.

## Runs (each binary whole, --test-threads=1, NF_ISO set)

| binary | result |
|---|---|
| rocket_stop_plane08_live | **4 passed, 0 failed** (was 2 failed) |
| rocket_glass08_live | **3 passed, 0 failed** (was 1 failed) |
| rocket_origin_live | **1 passed, 0 failed** (updated to the snapped point) |
| fusebox_geometry_07000014_live | **1 passed, 0 failed** (the laser cut works) |
| interactables_07000014_live | **10 passed, 0 failed** (was 1 failed) |
| walkreal_0700000c_live | **3 passed, 0 failed** (the real_damage walk the ray gap broke — fixed by the ray work) |
| walkreal_07000016_live | 13 passed, 1 failed (was 2 failed — improved) |
| walkreal_07000014_live | 6 passed, 3 failed (the branch had 2 — needs the per-walk review) |
| walkreal_0700001b_live | 15 passed, 2 failed — consoles 3/4/5/6 latch; "console 1" stalls |
| solar_panel_hits_live | FAILED — firing_along_the_walkway_breaks_no_panel is red on the branch TIP (it was red before this branch's changes too — measured at session start, before any edit) |

The walkway failure is NOT the aim ray: the probe shows the snap clean (level t 1.0, snapped = raw, the
camera ray missing every object). The break comes from the FLIGHT's panel triangle stage
(update_player_bullets' `Some((SOLAR_PANEL_GLIST, m))` arm) whose world.shape-source triangles posed by
the stream matrix catch the muzzle-origin segments — the same mis-framing the wires' +42 AABBs prove
[D]. The stage kept per the task's instruction ("keep the panel-only triangle stage unless a live check
says otherwise") — the live check (the breakpoints) says the game's panel test is triangle-exact, so the
STAGE is right and its GEOMETRY SOURCE is wrong: the port needs the panel model's own glist (no model
glist parse exists). That is the honest blocker for the walkway pin and the 1b console spots (the
surviving panels block the walk's tuned firing lines — the task's "new firing spots above the panels"
needs a next session with the console machinery).

## What's left

1. The panel model glist parse (the geometry source for the flight's panel stage) — unblocks
   firing_along_the_walkway and probably the 1b console walks.
2. The 1b console firing spots (the task's ask): the walk stalls at "console 1" (order 3,5,6,…,8) —
   consoles 3/4/5/6 latch, the rest need spots above the intact panels.
3. walkreal_07000014_live's 3 reds (the branch had 2 — the third may be my ray's object-side change
   shifting an aimed shot; needs the failure detail) and walkreal_07000016_live's remaining red.
4. The full suite on the final tree — the window went to the build churn (the world.rs struct change
   relinked everything; the last chunks were still running at the stop mark). The sweep infra
   (/tmp/order.txt + the chunk-runner pattern) is on the branch's report trail.
5. The probes (NF_AIMPROBE eprintlns in scene.rs/drone.rs/world.rs) are still in the tree — they are
   env-gated and inert without NF_AIMPROBE; the next session should decide to keep or strip them.

## Usage

not visible to me — this environment shows no token, credit, cost or context-window figures anywhere I
can read (no banner, no status line, no API surface exposed to the worker).
