## level-0d-crane-transit (started 03:03 UTC 2026-10-30)

Task: 0700000d the crane route into the ST-3 yard.
The walk test reaches the yard through a documented position stand-in; the game's way in
is the crane sequence (hint [D USATxt 0x4000064]; cranes are 0xE2 SimpleScripts -
shipping_handle #112, script players #115/#492, buttons #113/#114 animating on ch 0x2b)
which the port does not simulate [docs/levels/0700000d.md §6]. Find what the game does:
which input starts each crane, how the 0xE2 SimpleScript moves its load, how the player
gets carried [M the 0xE2 create/update functions, D the scripts]. Port it so the player
can do the crane route on real input, replace the stand-in in the walk with that route
(remove the set_channel of 0x1b/0x1c at line ~38 if the crane path raises them). Say in
the report whether the same 0xE2 family covers 0c's vent/pulley stand-ins. FULL suite,
quote its total line.

Plan: branch pushed from origin/master (21fbeed3) now; downloads running next; then read
the 0xE2 SimpleScript decomp + the 0700000d scripts, port, re-walk the test on real
input.

Progress 03:03-04:20 UTC:
- setup: repo cloned, work-slim + ISO in place, rustup/alsa built, all four crates'
  test binaries built (target-cache warm).
- the shipping-crane script 0x060009ef decoded off the disc (census_0700000d.rs
  dump_0700000d_scripts + a new dump_0700000d_crane_keys): 3000 ticks, 9 streams; stream 1
  = entity glist 02000a80 (type 31) with 301 markers every 10 ticks (keys 0..300) —
  the beam's world trajectory: origin (137.54, 10.33, −287.11) east to (177.22, 10.33,
  −287.11) at key 150 (t 1500), back west by key 300 (t 3000); the SP #115 placement
  (0, 1, 0) adds +1.0 y → the beam origin rides at y 11.33. Stream 2 = child glist
  02000a81 type 0xEA (a touch-once trigger riding the beam, params [0x67, 8, …] — its
  ch 0x67 reader is SoundTrigger #516, sound only). No camera, no Player_Disable, no
  carry event — the scripts themselves never move the player [D].
- the rotate crane 0x06000a08: 750 ticks, entity 02000a9c rotating in place about the
  SP #492 anchor (200.48, 13.29, −262.37), ~38° over 610 ticks (quat y 0.508→0.186) [D].
- port probes (probe_crane.rs, opt-in): the beam entity IS simulated and moves
  (entity_matrix tracked 142.8→177.1→west); the E on touchplate #460 from the dock
  (189, 9.9, −288.7) fires ch 0x2b through the real input path (matches the merged
  interactables test); the crane dock floor map (ray-down): the dock y 9.5 at
  x ~186-196 z −270..−294, a y 12.1 deck at z −266, a y 9.0 ledge at z −268 (x 164-186),
  a y 8.1 ledge at z −270 (x 160-184), the yard floor 3.98 under the beam's mid-gap,
  the beam top at rest y 14.32 (x ~135) / 13.47 (x ~140) — a gantry with two rails and
  an open mid-slot (a ray at x 137.5, z −280..−300 falls to the yard floor 3.98).
- OPEN (next window): the boarding leg — how the player gets onto the beam on real
  input (the ledge 8.1/9.0 at z −268/−270 is the likely approach; the beam's bottom
  flange sits at 8.35, 0.25 over the ledge); whether the port carries a rider
  (ground_motion exists, Player_FeetOnPoint → Control_InheritVelocity [M 0x194188]);
  then re-route walkreal_0700000d_live's stand-in through dock → E #460 → board → ride
  west → drop into the yard at x ~137, and remove the 0x1b/0x1c set_channel if the
  crane path raises them.
- checkpoint pushed before the 04:41 stop-new-work line.
