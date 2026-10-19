# Reverb zones — the level's per-area reverb (the cel's environment value)

How an on-foot level switches the SPU reverb depth and the indoors mix as Bond moves between
areas. The carrier is the **room cel**: every room-class cel placement carries a reverb value and
an underwater flag; the viewer's current cel feeds `SFXSetEnvironment` every frame; the IOP fades
the applied depth toward the target. Names: the `Env_*` functions in ACTION.ELF are the **weather**
system (rain drops, wind, breath — `Env_Update` 0x17bc88, `Env_Create` 0x17b968) and have nothing
to do with this system; the sound path is `Sound_UpdateListeners` → the SFX.IRX environment block.

Port status: **ported** (fix-room-sound landed the cel parse + the per-frame send; the IOP side
landed with WORKLOG 0075; this task adds the systems doc, the per-level [D] census and the
position-pinned zone tests on the indoor/outdoor pair 07000006/07000005).

## 1. The level data: which placements carry a reverb value

Cel placements are main-map placements with `flags & 0x8000` (they create a runtime cel,
`parsemap_block_map_data_static` 0x1d1998). `parseentity_fixup_entity` (0x1d00c0) then keys on the
low 16 flag bits and fills the runtime cel; exactly two classes carry room data:

| flags16 | meaning | cel+0x3c written | cel+0x98 (s16 env) | other |
|---|---|---|---|---|
| 0xc023 | water/room cel | 0x804c023, or **0x1804c023 when param 3 ≠ 0** [M 0x1d00c0] | **0x9898 written literally** (= s16 −26472, the IOP's underwater sentinel) [M 0x1d00c0: `*(u16*)(cel+0x98) = 0x9898`] | `cel+0x90` (water plane) = glist bbox max y; ambient = params 0/1/2 |
| 0xc047 | room cel | 0x804c047, or **0x1804c047 when param 3 ≠ 0** [M 0x1d00c0] | **param 9 as a raw u16** (`*(u16*)(level_tag+0x50)`, no mask/clamp) [M 0x1d00c0] | `cel+0xa2` lighting zone = param 5; second ambient triple = params 6/7/8; water = −500 |

- The **underwater flag** is `cel+0x3c & 0x10000000` — bit 0x10000000 is outside the `& 0xffff`
  the map block itself writes, it comes only from this fixup (param 3 = `level_tag+0x38`, a full
  u32) [M 0x1d00c0; 0x1d1998 writes `cel+0x3c = u32@record+8 & 0xffff`].
- A cel placement that is NOT one of these two classes keeps the **allocator fill 0x98** in both
  bytes of +0x98 (`build_alloc_cel` 0x1c93f8 never writes them), which reads back as the same
  **0x9898 sentinel** — the malloc fill and the written sentinel coincide [M 0x1c93f8 (no write)
  + 0x1d00c0; L ee_start_05.bin: untouched cel bytes at +0xa4..0xaf are 0x98].
- If the whole triple-dword at +0x98..0xa3 has no ambient byte set (`& 0xff00ff00ff0000 == 0`),
  the ambient RGB defaults to 0x3c each [M 0x1d00c0 tail].

Per-level census of the retail disc [D] (main map, `Level::parse` — the probe
`crates/nf-view/tests/reverb_zone_probe.rs --ignored census_cel_classes_per_level`):

| level | 0xc023 cels (param 3) | 0xc047 cels (env values) |
|---|---|---|
| 07000001 intro | 0 | 11 (0) |
| 07000002 The Exchange | **5 (param 3 = 0/1)** | 23 (0, 10, 15, 50, 80) |
| 07000003 / 04 / 24 / 29 / 4b | 0 | all env 0 (04: 2, one 80) |
| 07000005 Mayhew A | 0 | 19 (0, 40) |
| 07000006 Mayhew B | 0 | 37 (0, 20, 30, 35, 40, 50, 60, 70, 90) |
| 07000007 Mayhew C | 0 | 32 (0, 30, 40, 50, 60, 80) |
| 07000008 Mayhew D (gondola) | 0 | 18 (0, 50, 60, 69, 80, 100) |
| 07000009 Night of the Agent (Tower A) | 0 | 14 (20, 40, 80, 100) |
| 0700000a / 0b / 0d / 11 / 12 / 14 / 15 | 0 | mixed, see probe |
| 0700000c power station 1 | 0 | 22 (0, 40, 50, 70, 80, 90) |
| 07000041 / 16 | 1 (param 3 = 0) | 41: 7 (0, 60, 100) / 16: 7 (0, 20, 60, 100) |
| 0700004a Tower-2 elevator | 0 | 57 (all 100) |

07000002 is the only level whose cels exercise the 0xc023 sentinel path with the param-3 flag
split (0 and 1) [D].

## 2. Which cel is "the viewer's cel" — Camera_CheckLocation

Every frame `Camera_UpdateAll` (0x1bbd68, called from `Game_Run` 0x1c9790 / `GameFlow_Main`
0x1c98c8) runs per viewer (up to 11, `glb_viewer[]`): `Camera_Update`, **`Camera_CheckLocation`**
(0x1bae90), `Camera_UpdateGlbVars`. `Camera_CheckLocation` maintains **viewer+4 = the cel the
viewer's eye is in** [M 0x1bae90]:

1. If `viewer+0x29` (the viewer-active byte) is set: ray from the previous eye (`viewer+0xd0`) to
   the current eye (`viewer+0xc0`) through the current cel's **portal list** (`cel+0x28`):
   `Cel_ObjectLeftCel` (0x1e0d58) bbox-filters each portal (+0x80/+0x8c), ray-tests the portal
   quad's two triangles (+0x30/+0x40/+0x50, +0x50/+0x60/+0x30) with `Collide_RayTriangle`, and the
   first hit inside [0, |new−old|] returns the portal's **destination cel** (+0x8) [M].
2. No portal crossed: if the eye is still inside the current cel's bbox
   (`build_point_in_bounding_box`), keep it [M 0x1bae90].
3. Otherwise `build_FindCel` (0x1c8cb0) — the global bbox scan (the port's `find_cel` keeps the
   smallest containing bbox among up to 0x40 candidates, the game's tie-break, verified against
   ee_start_05 in fix-perf-level); if it finds none, **keep the previous cel**, and if there was
   none, fall back to `*glb_world` (the first cel) [M 0x1bae90]. A fresh find sets `cel+0xa0 = 1`
   (the visited flag the vision walk clears) [M].
4. The eye is remembered: `viewer+0xd0 = viewer+0xc0` on every path [M].

Player code resets viewer+4 to 0 on state switches (teleports: `Player_StandAtNewPosition`
0x190d88, `Player_Update` case 10, GT take/lose control, the redeemer HUD, SS scan mode — all
write 0), so the next frame re-finds from scratch [M; the writes are `= 0` only].

## 3. The per-frame send — Sound_UpdateListeners

`Sound_UpdateListeners` (0x1dd278, per frame) ends with [M 0x1dd278, disasm 0x1dd4b0–0x1dd4fc]:

```c
cel = viewer[4];
if (cel != 0) {
    underwater = (cel->flags_3c & 0x10000000) != 0;
    if (!underwater) underwater = (*(short*)(cel + 0x98) >= 0x14);   // env >= 20
    SFXSetEnvironment(*(short*)(cel + 0x98), underwater);
}
```

- The send happens **every frame, no change detection**; a cel-less viewer sends **nothing** and
  the IOP keeps its previous targets [M].
- The first argument is the **raw s16** — the 0xc023 cels and any untouched 0x98-fill cel send the
  −26472 sentinel [M].
- The second argument (the IOP's "indoors" mix selector) is `underwater_flag || env >= 20` — a
  signed compare [M disasm 0x1dd4b0].
- The same function then does the listener setup, `HandleMapSoundAllocation` and `SFXUpdate`
  (which runs the fade below) [M 0x1dd278].

The EE→IOP transport is the batcher: `SFXSetEnvironment` (0x116fc8) writes RPC command 0x1d with
the two words into `SFXBatchCommands` (flushed by `SFXUpdate` in the same frame) [M 0x116fc8].

## 4. The IOP side — target clamp, the fade, the SPU write

SFX.IRX (all [M], `work/decomp/SFX.IRX`):

- **`SFXSetEnvironment` @0x0db4**: `reverb == −0x6768` (−26472) → `SFXUnderWater = 1`, reverb
  target 0; otherwise `SFXUnderWater = 0`, target = clamp(reverb, 0, 100). The second argument
  sets the **indoors mix target**: 0 → 100, anything else → 50 (`SFXNewIndoorsVolume`).
- **`SFXUpdateEnvironment` @0x0e60** (every `SFXUpdate`): `SFXReverb` (the current depth) moves
  toward the target by at most **+16 rising / −4 falling** per frame; `SFXIndoorsVolume` moves
  **±4** per frame. So a zone change from 0 to 40 takes 3 frames (16/32/40) to rise and 10 frames
  to fall back — the game's crossfade between areas.
- **`psiSetReverb` @0xc840**: `EVOL = (((r·0x50)/100)·0x3fff)/100` (integer; 0x50 = 80 — the game's
  max send depth is 80 %) written to EVOLL/EVOLR of both cores (`sceSdSetParam` 0xb80/0xc80/0xb81/
  0xc81) [M].
- **`PS2_SetupReverb` @0xc944**: the global SPU effect preset set once at boot — effect mode 4,
  delay 30, feedback 200 on both cores, `sceSdSetCoreAttr(core|2, 1)` (enable) [M; the
  `sceSdEffectAttr` local layout is partially mangled in the decompile]. The port's SPU2 effect is
  a stand-in (G-032d, `nf-audio` lib.rs `STANDIN`).

Port mapping (`crates/nf-audio/src/sfx.rs`): `Driver::set_environment` (the clamp + sentinel +
indoors target), `Driver::update_environment` (the ±16/−4 and ±4 steps, returns the EVOL depth
fraction), `Driver::environment()` (test accessor for the targets), run per frame from
`Sound::batch` → `Driver::update` [M mapping, WORKLOG 0075].

## 5. The port's side (nf-view/nf-game)

- **The parse** — `nf_view::scene::parse_cel_graph` / `Cel` (`crates/nf-view/src/scene.rs`): room
  cels carry `env: i16` and `underwater: bool`, extracted exactly per §1 (0xc023 → −0x6768,
  0xc047 → param 9 as raw u16, param 3 ≠ 0 → the 0x10000000 bit).
- **The pick** — `Sim::cel_at(p)` = `find_cel` (the smallest containing bbox of `Scene::cels`).
  The port has no incremental portal-ray cel tracker: every `sound_tick` re-finds from the eye
  position. Deviations from the game (documented in fix-room-sound's report): no viewer array
  (the single viewer is Bond's eye; the script-camera listener call is dropped by the EE batcher
  anyway), sub-map cels are not in `Scene::cels` (a viewer inside one sends nothing where the
  game would send the 0x98-fill sentinel), and the keep-previous / first-cel fallbacks never
  engage (a Bond outside every room cel sends nothing, where the game would keep the last cel —
  the game's own resets to 0 make the empty send reachable there too).
- **The send** — `nf_view::sound_tick` (`crates/nf-view/src/lib.rs`): every frame
  `drv.set_environment(cel.env as i32, (cel.underwater || cel.env >= 20) as i32)` from
  `sim.cel_at(bond.head())`, then the batch (`SFXUpdate`) applies the fade.

## 6. Tests

- `crates/nf-view/tests/fix_room_sound_live.rs` (fix-room-sound): the 07000005 parse pin (19 room
  cels, 15 × env 0 dry / 4 × env 40 underwater [L ee_start_05.bin]) and one end-to-end
  teleport into an env-40 cel.
- `crates/nf-view/tests/reverb_zones_live.rs` (this task): position-pinned zones on the
  **indoor/outdoor pair** — 07000005 (the castle approach, outdoor: the spawn yard is env 0 dry,
  small interior pockets env 40) and 07000006 (the castle interior, indoor: spawn hall env 50,
  cellar pockets env 90, side rooms 30/70) — each through the real `sound_tick` path with sister
  assertions, plus the crossfade ramp (§4's ±16/−4) driven across a zone border.
