# level-4a-satchels-lift — the 0700004a lift satchels, as the game does them

Branch `s/level-4a-satchels-lift` off `origin/master` (4c1f5ffb). The driver's three play spots
(Bond root x,y,z,yaw,pitch; frame since load) were reproduced head-for-head: driving the real
level from its start with real input (the walker, no sim-state writes) reaches the lift ride and
hits the same ride phase the driver's frames show — my spot ticks 2188/3016/3167 against the
driver's 1979/3024/3177 (the two later ones within ~10 ticks; the first differs because the
driver's landing cut-scene was partly skipped). Everything below was probed on that ride.

## What the ride probe showed (sim side)

The port's SIM is correct on this level; the existing `walkreal_0700004a_satchel_live` tests
cover it and stayed green (2/2, 12.6 s):

- Shooter #286 [D cameras-turrets.md §3, weapon 0x3d] fires on ch 0x22 (the landing cut-scene's
  end [D s27]); the charge falls straight down [M Shooter_Update 0x17d0c0], lands (state 3
  [M Bullet_CollisionHandler 0x12ce80 @0x12d1fc]) at #286's x/z (3.04, −1.67) at ride tick +180,
  and RIDES THE CAB: it descends at −0.0303 u/tick, tick for tick with Bond's cab floor
  (probe log [ride +240..+780]) — the same 0.03 u/tick the driver's own two spots show.
- The fuse ticks down from 900 [M Bullet_init 0x12c5a0:199-211] and the digit pair updates every
  tick ([M Bullet_DoTrails 0x12d890:418-553] — the sim's `tick_charge_fuse`).
- The laser watch burns it (burn 100, 1/tick for the 0x4E [M 0x12d890 disasm 0x12e2f0-0x12e390])
  and an undefused charge explodes at fuse 0 (~t3050 at cab y −80.8; the driver's spot 3 at
  frame 3177 still SHOWS a satchel, so their session had already burned it — the sister test
  `an_undefused_satchel_explodes_and_fails_the_mission_like_the_game` pins the blast).

So the four play symptoms were all in the DRAW path the sim tests never exercise. Three fixes:

## Fix 1 — the satchel drew full-bright ("brightly textured, unlit like the old ghost guards")

- The satchel's projectile mesh 0x20007b3 (weapons 0x3b-0x40, 0x6c) carries glist flags 0x880
  [D 0700004a object table; probed]. Bit 0x800 is the ambient-dim flag: every dynamic object
  ramps obj+0x100..0x102 toward its cel's ambient each tick [M Lights_CalcAmbientLight 0x1cb178,
  called for every dynamic object in control_movement_object_handler 0x1336f8] and the draw
  multiplies the tweak rgb by those bytes [M View_SetupRenderModes_Tweak 0x1e5888, the 0x800 arm].
  The lift shaft's cel ambient is [25,27,35] [L probe] — the game draws the charge at ~10-14 %
  brightness. The port baked [1.0;3] for the whole projectile pool, and the per-attachment
  ambient walk carried a blanket exemption for every bullet slot (the rocket-flare fix).
- Fix (scene.rs `colour_mults_dense`): the bullet slots now dim by the MESH's 0x800 flag —
  `models_0x800[hash]` — at the charge's live position, instead of the slot-based blanket 1.0.
  The rocket/MGL meshes carry no 0x800 (0x2000432/0x20006e0 = flags 0x201 [L probe]) so the
  flare stays white; the thrown-grenade path is untouched. Verified headlessly: the satchel's
  colour-table entry on the ride = [0.094, 0.102, 0.133] = the shaft's ambient (was [1,1,1]).
  [*deviation*, [S]: the game's ClrStep ramp toward the ambient is modelled as a snap — a fresh
  object snaps on its first cel link [M control_link_object_to_cel 0x1346b8 copies the cel
  ambient over the all-0xFF init]; the ride's shaft shares one ambient triple so the deviation
  only shows mid-ramp across cel boundaries.]

## Fix 2 — the countdown display never showed digits

- The sim wrote `digits` every tick (both carriers), the digit effect slots existed, and the
  frame matrices were live — but the digits drew BURIED: the port's local offset (±0.03, 0.032,
  +0.06) ran the +0.06 z along the stick frame's z row = −normal, i.e. INTO the case/floor
  (probe: the digits drew 0.06 BELOW the charge centre). The game's constants [M Bullet_DoTrails
  0x12d890:486-511] put the tens digit at (0.09375, −0.0021, −0.123) and the units at (0.03262,
  −0.0021, −0.123) in the stick frame (z row = −normal, the same Mat_Align2Dir constants
  [M 0x12d210: 0x2d6210 up / 0x2d6220 right]) — the local −0.123 z maps 0.123 OUTWARD along
  +normal, the display floats off the face the charge sticks to — and the digit object's rotation
  is MulMatrix0(bullet_rot, RotMatrix((π/2, 0, π))) [M 0x1e2820 = A×B; 0x1e2368; the constant
  pair 0x3fc90fdb/0x40490fdb at 0x12e424-0x12e438].
- Fix (scene.rs digit effect route, kinds 4/5): the game's exact offsets and the local rotation
  fold (rows × M_local via the same A×B order; `object_matrix` is the port of RotMatrix's
  PS2Sinf3 euler convention [M 0x1e2368 vs placement.rs]). Verified: the digits now draw 0.123
  ABOVE the floor-stuck charge (probe spot2: digits (3.04, −80.29, −1.70/−1.77) over the charge
  at (3.04, −80.41, −1.67)) — visible from the rider's downward view, exactly where the game
  puts its display.

## Fix 3 — the black frame at (3.61, −54.06, −2.58)

- What draws there and what doesn't, probed at the same ride phase: the eye's cel resolves
  (cells 57/56/55 as the ride descends [L probe4a_draw]), the portal walk marks 20 cells, and
  the static shaft geometry's rooms are IN the walk's set — the shaft walls are not culled.
- The cab (script obj #0, entity kind 49, mesh 0x20005b4, glist bounds a 4.3 × 50.5 × 4.3 cage
  [L probe]) draws through the script-entity attachment route and its frame matrix was LIVE at
  every probed spot. But its keyed translation is the mesh-space origin (0, y, 0) whose cel is
  an off-shaft VOID room (28/22/21, never in the walk's set). Two gates read that bare origin:
  the skin palette's script-character room test, and the script arm's ambient lookup — both
  mis-attribute the cab to the void rooms. Fix (scene.rs): both now read the posed matrix ×
  the mesh's glist-bounds centre — the game's obj+0x80 world sphere is the posed glist bounds
  [M Control_BuildWorldSph 0x133070], and the game relinks the moved entity into its cel chain
  per tick [M control_handle_cel_change 0x133288 in 0x1336f8:147-236]. On 0700004a this is
  defensive (the cab's mesh is not 0x800-flagged [L probe: flags 0x40000] and no entity
  characters exist here), but it removes the void-room attribution for every keyed entity.
- **Not fully closed**: the sandbox has no GPU ("No suitable graphics adapter" — no wgpu
  backend, no root to install one), so I could not render the frame and diff pixels. The
  remaining suspect if the frame is still black after the above: the driver's spot 1 sits in
  the landing cut-scene's window (their frame 1979 is before their ch 0x22); the script camera's
  own eye-cell resolution and the Character-layer frustum cull during the scene deserve the
  main agent's render (below).

## Tests

Run on the fixed tree (release, NF_ISO, --test-threads=1):

- `walkreal_0700004a_satchel_live` **2/2 ok** (11.2 s) — the real-input ride: both charges land,
  ride, burn under the watch, the lobby reached alive; the undefused sister blast fails the
  mission at the fuse.
- `walkreal_0700004a_live` **1/1 ok** (7.7 s) — the ride + the lobby walk (exercises the
  palette/cab-path changes).
- `camera_beam_satchel_wreck_live` **3/3 ok** (5.1 s) — the thrown-satchel draw + digit path.
- The three headless verifications above (colour table, digit matrices, cab/room attribution)
  were probed through the same frame-state/palette calls the renderer consumes
  (`examples/probe4a_draw.rs` — a scratch probe, not committed).

## What's left

1. **Headless renders of the three spots** — needs a GPU; my sandbox has none. Render commands
   for the main agent (the same drive, then `nf_view::shot` at the driver's roots):
   - ride the route of `walkreal_0700004a_satchel_live::ride_the_lift...` to bond y −54.06 and
     capture at cam (3.61, −54.06+eye, −2.58, yaw −1.486, pitch −0.662);
   - the same at y −79.29 / y −83.87 with the driver's yaw/pitch. What each should show: the cab
     interior around the satchel, the charge dimmed to the shaft's dark ambient, the two counting
     digits floating 0.123 off its top face.
2. **Live checks for the main agent** (PCSX2, fresh boot, on the lift ride):
   - the charge's obj+0x100..0x102 while riding (expect the shaft ambient ≈ [25,27,35], i.e.
     obj+0x100-2 ramped there) — confirms Fix 1's target value [L];
   - the digit objects' world positions relative to the charge (expect ≈ +0.123 along +normal)
     — confirms Fix 2 [L];
   - the frame at (3.61, −54.06, −2.58, −1.486, −0.662) on a fresh boot vs the port's render —
     closes Fix 3's remaining question (the cut-scene camera).
3. The burn sparks (`Emitter_Create(0xc00000a, ...)` [M 0x12e37c]) are still not ported (the
   port's emitter set has no charge-spark shape) — the beam's EFFECT on the charge is visible
   through the digits/defusal, not through sparks.
4. The fast suite ran on the PRE-FIX tree (started 16:03, ~90 min): the failure set matched the
   documented classes (3 disc_golden + identity_disc on the slim ISO, the 0c-pylon GPU trio, two
   OOM crashes); its total line could not be re-run on the final tree inside the window.

## Usage

not visible to me — this session's environment shows no token, context-window or cost figures
(the IM gateway metadata carries none; no usage panel is exposed to this agent).
