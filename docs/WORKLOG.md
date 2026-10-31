
## lamp-wind — 2026-10-31

Task: the game's hanging lamps never settle (fresh-boot live: 2-4° off rest, drifting, shot or
not); the port fed rigid bodies zero wind, so its lamps sat dead still. Branch from master,
merged s/lamp-calib (the RB init/evolve base).

Findings so far ([M] unless noted):
- `RB_Update` 0x1d6e38's force block is BUOYANCY, not heat: the gate is `obj_y < cel+0x90` where
  cel+0x90 is the WATER surface (`build_alloc_cel` 0x1c93f8 default −500.0; `parseentity_fixup_entity`
  0x1d00c0 sets cel+0x90 = cel+0x54 for room class 0xC023 — the port's verified fix-ladder doc), and
  the force (0, |obj_y − cel+0x54|·mass·0.4 + Rand_FRand(0.15), 0) ∝ depth, with `Gas_Init` bubbles.
  Inert for the lamps (type 1: the linear impulse is consumed only by the type-0 arm of
  RB_ApplyImpulses 0x1d6ff8).
- The lamp mover is the WIND: `RB_ApplyWind` 0x1d7c78 → `Env_GetWindVel` 0x17c048 — for type != 0 the
  force lands at a point (obj+0xc4 + rb+0x9c − rb+0x13c) → torque, the only per-frame torque a freely
  hanging Rot body gets.
- Wind source: placement kind 0xDF (`Env_Create` 0x17b968) writes a wind struct W on the cel
  build_FindCel 0x1c8cb0 picks (cel+0x2c): dir = W+0x00 (RotMatrix 0x1e2368's dir row of the
  placement rotation; Mat_GetDir 0x1e3398 = the +0x20 row), strength W+0x30 = target W+0x34 =
  param3·0.1, phase W+0x38 = −1, falloff W+0x44 = 1/(pCelList record+0x1c radius), grow latch W+0x4d.
  `Env_Update` 0x17bc88 animates it per frame from the ONE per-frame `Rand_Rand(20000)` (no extra
  draws): phase sweeps −1..1, strength doubles up / decays between 0.25×target and target.
- Data probe ([D] the port's level parse): 07000004 main map has 0xDF #5 (rain, p3=6) in room cel #1,
  the SAME cel as lamps #39/#40 (type 1, mass 1) — wind on the lamps. 07000006's barrel #65 shares
  cel #29 with SNOW_gen #1 (p3=6). 07000001 has NO 0xDF → its four same-kind bodies are the
  in-game zero-wind sister.

Next: port EnvWind + the gust cycle + Env_GetWindVel, plumb cel lookup per body, live test the
07000004 drift within the 2-4° live band, sister 07000001 stays still, update the 07000006
wiring test (its zero-wind premise predates the wind port).

## lamp-wind — 2026-10-31 (the port + the pins)

Ported: `EnvWind` (Env_Create 0x17b968's wind half: strength=target=p3·0.1, phase −1, grow 1,
dir = RotMatrix 0x1e2368's dir row, falloff 1/(the wind cel's pCelList record+0x1c radius)),
`env_update_gust` (Env_Update 0x17bc88's per-cel step from the one per-frame `Rand_Rand(20000)`
— no new draws), `env_get_wind_vel` (0x17c048). The room-cel build + the 0xDF→cel attach + the
per-body cel pick live in `Rbodies::create` (nf-game, no view dependency); `tick_winds` runs at
the env slot of the tick (before the object handler, the game's order); `tick_one` passes the
body's buoyancy fields (cel+0x90 water surface, cel+0x54) and its wind. RB_Update's force block
re-read as BUOYANCY (cel+0x90 = the water surface [M fix-ladder's verified parseentity_fixup_entity
0x1d00c0 reading], the depth-proportional lift + Gas_Init bubbles) — inert for the Rot lamps; the
lamp mover is the wind's at-point torque [M 0x1d7c78].

Data [D the port's level parse]: 07000004 0xDF #5 (rain p3=6) and lamps #39/#40 share room cel #1;
07000006 barrel #65 shares cel #29 with SNOW_gen #1 (p3=6); 07000001 0xDF #279 shares cel #0 with
lamps #9-#12. Every 0xD8-bearing named level has a wind — the zero-wind sister is the target-0
gate (0x17c048) pinned live + in unit tests.

Tests: `lamp_wind_live.rs` (the drift through the live 2-4° band + the zero-wind bit-still sister);
fixed per the decompile (their zero-wind premises predate the port): `rigid_body_wiring_live`
(the 07000006 barrel drifts, bounded < 0.5°), `interactables_07000004_live` (both rest pins),
`interactables_07000001_live` (the identity-rest sisters → the bounded-drift wind sisters, the
bullet detector is a per-tick omega jump now), `rigid_body_draw_live` (the settle band + the
replay's wind-phase caveats). Unit: the gust cycle, the wind formula, the create parse, the
update order (18 rigid_body lib tests green).

Finding: the port's wind torque at the p99 replay is ~2× the recording's (peak 52.3° vs live
44.24°) — the amplitude gap needs the live checks in the report (the exact point-vector slots of
0x1d7c78 from the disassembly + a fresh-boot tilt trace).
