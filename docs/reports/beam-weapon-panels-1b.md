# beam-weapon-panels-1b — the start weapon breaks panels it should not

Branch `s/beam-weapon-panels-1b`, cut from `origin/s/panel-mesh-wip` (786875a4f; origin/master fd8ee385b
already an ancestor). Environment reset before this task: full re-setup (clone, work-slim, ISO, rustup, ALSA
user prefix, cold build). The window closed mid-verification — the fix is pushed UNVERIFIED.

## What the research established ([M] decompile/EE, [D] live measurement)

- **Weapon 0x33 (id 51, the Beam)** record [L ee_start_05.bin weapon_data 0x2c26b4]: class +0x06 = 4
  (explosive), damage +0x0C = 20.0, blast radius +0x08 = 0.0, max range +0x1C = 500, speed +0x20 = 4.25,
  refire +0x40 = 30 frames, spend +0x91 = 12, clip +0x92 = 100, +0x44 fire_anim_len = 0, +0x6C = 0x10000,
  **+0x70 = 0x2 — no 0x100000/0x200000 explode bits** (the lead's Explode_Create 0x60007c4 hypothesis is
  dead on the record: those arms [M Bullet_CollisionHandler 0x12ce80:207-225] never trigger for 0x33).
- **It IS a bullet, and it fires on the press + every 30 frames**: `Player_WeaponFiring` 0x1a5460 spends
  the pull (`Player_RoundToFire`, +0x91), re-arms `BL+0x92C = +0x40` and, for +0x44 == 0, spawns
  `Player_WeaponInitBullet` → `Bullet_init` on the same tick [M 0x1a5460:654-690]. 245 frames of holding ≈
  8 shots ≈ 96 rounds of the 100 clip — "the meter rising to about 95" reads as the beam's clip/charge
  gauge draining 12 per shot (the walker harness already knows the drain: "each tap drains 12" and the
  +1-per-4-frames recharge when no fire script runs [M Player_Update 0x193600 disasm 0x193eb8-0x19405c]).
  So the game DID fire ~8 shots along the walkway and broke nothing — the difference is the IMPACT path,
  not a charge gate.
- **The game's bullet-vs-object pick is SPHERE-ONLY**: `Collide_PickObj` 0x1eae80 takes the closest point
  on the hittest segment to the object's BUILT world sphere (centre obj+0x80, radius obj+0x8c), skips when
  `radius² <= sqDist`, gated by `obj+0xF0 & 0x20` (the pickability bit the copter work documented) and the
  hittest-type gate (type 0x100 passes; other types need `obj+0xfa & 8` or a non-empty glist). **No
  glist triangles in the object pick** — `Collide_Intersect` 0x1ebaa8's triangle stages serve the OTHER
  hittest kinds (0x101 point, 0x201 segment-vs-static, 0x400), and its object branch reads the object's
  own glist (obj+0xD8 → HITTEST+0x88), not the level's collision shapes.
- **The built sphere's radius carries the object's scale**: `Control_BuildWorldSph` 0x133070 — centre =
  glist+0x10 through the object matrix chain, **radius = glist+0x1C · |obj+0xE8|**. The port posed the
  glist bound but never scaled it.
- **The measured geometry** (probe on the real disc load, this session): the 16 panels (placements
  41-56, hp 20) are stream Entities of hash 0x2000a42, sphere r 4.77, centres stepping x 10.14 → 56.12
  (Δ6.57) with y −1.72 → −9.83 (Δ−1.16), rows z ±4.62. Bond's walkway spot (16.74,−1.84,4.62) is INSIDE
  panel 50's sphere (1.04 from its centre); the first shot's tick-0/1 segments (muzzle (13.48,−1.24,4.53),
  −x at 4.25/tick) pass 0.49 from panel 49's centre. world.shapes HAS 0x2000a42 (24 triangles) — yet the
  real game breaks nothing: the pick's gates (0x20 / built-sphere scale) must reject what the port's raw
  glist bound accepts.

## What I changed (commit bf742360a, UNVERIFIED)

1. **Reverted the WIP's triangle stage** in `update_player_bullets` (both the held_targets damage gather
   and the sp_bullet_targets contact gather are back to the sphere test): the pick decompile
   [M 0x1eae80] is sphere-only, and the triangle stage measurably broke the console latch
   (walkreal's "console 1 latched") while still failing the walkway.
2. **Folded the object scale into the pick radius** in `Scripts::held_hit_volumes` and
   `sp_bullet_targets` (crates/nf-game/src/nis.rs, both Entity branches): radius ×= the stream matrix's
   row-norm mean — the port's analog of the game's `glist+0x1C · |obj+0xE8|` [M 0x133070].

**The open risk, stated plainly**: if the panels' stream matrices carry scale 1.0, the radius is
unchanged and the walkway still breaks panels — the true remaining gate would be `obj+0xF0 & 0x20`
(pickability), which the port does not model for script stream objects. The measurement I could not
finish: the panels' stream-matrix row norms (the probe binary kept being rebuilt as the window closed).
If the scale path is wrong, the next session should (a) print the matrices' row norms, (b) if unscaled,
source obj+0xF0's 0x20 for stream objects (Script_GetObj 0x1db688's creation path / the glist flags
celglist+0x44 bit 0x20 per level.rs's flags note) and gate both gathers on it.

## Baselines (pre-fix tree, all run this session, NF_ISO set)

| binary | result |
|---|---|
| solar_panel_hits_live | FAILED 0/2 — walkway still breaks panels, AND the down-shot sister fails (the WIP triangle stage rejects even the direct hit) |
| solar_panels_live | 2 passed, 0 failed |
| walkreal_0700001b_live | 15 passed, 2 failed — both Drake walks, the "console 1 latched" failure the task describes |

## What's left

- Rebuild (the fix commit is in the tree; `cargo test --offline --release -j4 --no-run` was still
  compiling when the window closed) and run `solar_panel_hits_live`, `solar_panels_live`,
  `walkreal_0700001b_live` on the fixed tree; then the changed-code set (grep
  `held_hit_volumes|sp_bullet_targets`) and the full suite as the window allows.
- If the scale fold does not fix the walkway: implement the `obj+0xF0 & 0x20` pickability gate for SP
  stream objects (source it first — Script_GetObj's object creation, the glist flags), keeping the
  consoles pickable.
- The scratch probe file was deleted before commit; nothing scratch is on the branch.

## Usage

not visible to me — this environment shows no token, credit, cost or context-window figures anywhere I
can read (no banner, no status line, no API surface exposed to the worker).
