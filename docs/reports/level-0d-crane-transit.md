# level-0d-crane-transit — the crane route into the ST-3 yard

## What this window did

Setup ran clean (repo, work-slim, ISO, rustup, ALSA user prefix, all four crates' test
binaries built warm). No game-code changes were made this window; the work is evidence
plus two additive, `#[ignore]`d, print-only tests.

**The scripts are decoded off the disc [D]** (`crates/nf-view/tests/census_0700000d.rs`,
new `dump_0700000d_crane_keys`, plus the existing `dump_0700000d_scripts`):

- `0x060009ef` (SP #115 `ScriptPlayer_ShippingCrane`, mode 8, mask 1, out ch 0x2b):
  3000 ticks, 9 streams. Stream 1 = entity glist `02000a80` (type 31) with 301 markers
  every 10 ticks over keys 0..300 — the beam's world trajectory is an east–west shuttle:
  origin (137.54, 10.33, −287.11) east to (177.22, 10.33, −287.11) at key 150 (t 1500),
  back west to (137.54, …) at key 300 (t 3000). SP #115's placement (0, 1, 0) adds
  +1.0 y, so the beam origin rides at y 11.33. Stream 2 = child glist `02000a81`,
  type 0xEA — a touch-once trigger riding the beam, params [0x67, 8, …]; its only
  reader is SoundTrigger #516 (gate ch 0x67 → sound 333, play-once) [D audit]. Streams
  7/8 are the crane-motor sounds. **No camera stream, no Player_Disable, no carry
  event: the script itself never moves the player.**
- `0x06000a08` (SP #492 `ScriptPlayer_CraneRotate`, mode 0x40, out ch 0xa): 750 ticks;
  entity glist `02000a9c` rotating in place about the SP anchor (200.48, 13.29, −262.37),
  ~38° over 610 ticks (quat y 0.508 → 0.186), position keys all (0, 0, 0) [D].

**The port already simulates the moving beam and carries a rider** (new opt-in probe
`crates/nf-view/tests/probe_crane.rs`):

- `probe_the_shipping_ride`: after a real-input E on touchplate #460 from the dock
  (189.0, 9.9, −288.7) — ch 0x2b fires [matches the merged
  `interactables_0700000d_live::crane_e_plate_arms_the_handle_ss_and_the_shipping_sp`]
  — the beam entity's world origin tracked 142.84 → 177.12 (apex ~t 1300) → back west,
  at constant y 11.33, z −287.11.
- `probe_the_ride_carry`: Bond placed on the westbound rail (origin x − 2.54, top
  y 14.32, at z −280) was carried the whole ride — his x holds a constant ~1.9 offset
  from the beam origin through the west apex (137.55) and follows the beam back east
  when it reverses. This is `CollisionWorld::ground_motion` (Player_FeetOnPoint
  0x194188 → Control_InheritVelocity [M]) doing its job on the moving set
  (docs/systems/script-player.md §Collision). **The ride works; only the boarding is
  unproven.**
- `probe_the_dock_floor_map` (ray-down over x 168-196, z −262..−294): the crane dock
  floor y 9.5 at x ~186-196 (z −270..−294+), a y 12.1 surface at z −266 (x 168-196),
  a y 9.0 ledge at z −268 (x 164-186), a y 8.1 ledge at z −270 (x 160-184), the yard
  floor 3.98 south of z ~−288, a 4.0 step at x 184. The beam's glist bounds are
  (−4.17, −2.98, ±17.01) local with an identity quat → a 34 m N-S gantry: two rails
  (tops y 14.32 at origin−2.54 and y 13.47 at origin+2.46) with an open mid-slot (a ray
  at x 137.5, z −280..−300 falls to the yard floor 3.98).

**Baselines on this tree are green**: `interactables_0700000d_live` 11 passed / 0
failed; `walkreal_0700000d_live::walk_arrival_to_the_c5_door` 1 passed / 0 failed
(5.02 s, still via the documented stand-in).

## What is left

1. **The boarding leg** — how the player gets onto the beam on real input. The ledges
   (y 9.0/8.1 at z −268/−270) sit just outside the beam's z extent (−270.1..−304.1) and
   5-6 m below the rail tops; the beam's bottom flange (8.35) passes ~0.25 over the
   z −270 ledge and the ledge-watch showed the standing player is NOT pushed as it
   passes. Candidates still to probe: the trolley's park slot at the beam's north end
   (solid at (137.5, −272) at rest, open south of it), the wire #271 `wire_crane2crane`
   (206.93, 17.04, −246.13, gate param p0 = 0xa — arms with the ROTATE crane), the
   rotate-crane jib ride, and ladder #435 (`ladder2crane1`, bottom 0.93 top 17.21 at
   (212.47, −234.51) — reachable only from the ground below the dock). [S] all — no
   assertion yet.
2. Re-route `walkreal_0700000d_live`'s stand-in through dock → E #460 → board → ride
   west → drop into the yard (~x 137), then decide the `set_channel(0x1b/0x1c)` line
   (the warm save's record-11 carry is separate from the crane path; nothing on the
   crane route writes 0x1b/0x1c [D — the script has no channel writes], so that line
   stays unless the next window proves otherwise).
3. Say whether the same 0xE2 family covers 0c's vent/pulley stand-ins: from the merged
   0c docs, no — the `obj_stor*_pulley` winches #351/#364/#377 are DEAD props (no
   translation keys; the 0c traversal went static and is green), so nothing there rides.
4. The FULL suite total line on the final tree (this window: no source changes; the two
   touched test binaries pass; the suite run is owed on the re-routed tree).

## Live checks for the main agent

1. Fresh-boot 0700000d, walk Bond onto the z −270 ledge (~(176, 8.9, −270)) and let the
   shipping crane cycle run: does the game push him when the beam passes (the port does
   not), and what does a real player DO to board the beam? Watch Bond's position.
2. Does SP #115's script restart after t 3000? The port probe saw the beam origin rise
   again past the script end (143.68 at ~t 3160) — loop or hold at 137.54?
3. During the ride, does the beam's riding 0xEA trigger (glist 02000a81 → ch 0x67) fire
   when it crosses the player/SoundTrigger #516's gate (sound 333)?

## Usage

not visible to me
