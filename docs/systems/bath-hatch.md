# The 02 bath hatch over the deep ladder (07000002)

The deep ladder on 07000002 (centre (−50.35, ·, 5.50), yaw π/2, top 5.531, bottom 0.547 —
`docs/reports/ladder-underwater-02.md`) climbs out of the undergarden water tunnel into the
bathhouse. Its exit is closed by a hatch SimpleScript; this note is the disc + decompile answer
to how the game opens it, and what the port runs.

## The hatch is SimpleScript #308 [D]

The 0xE2 placements of the main map (nf-formats parse of `07000002.bin`; the sim's
`SsList::create` fields) — note **#308**, not "#269" (the ladder-underwater-02 report's
misnumbering; no #269 exists in the placement list, and the params it quotes are #308's):

| placement | name | at | p0 state | p1 wrap | p2 mask | p3 ch | p4 snd | p6 spline | keys/rate |
|---|---|---|---|---|---|---|---|---|---|
| #308 | bath_hatch | (−49.75, 4.76, 6.13) | 64 = 0x40 | 1 | **4 (E)** | **2** | 586 | 1 | 2 / 75 |
| #309 | bath_hatch1 | (−49.76, 4.76, 16.94) | 64 = 0x40 | 1 | 1 (no E) | 1 | 586 | 1 | 2 / 75 |
| #272 | sg_grill_large | (−16.12, 4.34, 35.81) | 0x40 | 1 | 4 (E) | 0 | 586 | 1 | 2 / 75 |
| #440 | grill1 | (−21.62, 2.82, 33.03) | 0x40 | 1 | 4 (E) | 0 | 586 | 1 | 2 / 75 |

Field map per `SS_Create` 0x1a9c58 (+0x22 = param 0 state, +0x24 = param 1 wrap, +0x1E = param 2
mask, +0x10 = param 3 channel, +0x14 = param 4 sound, +0x08 = param 5 rest, +0x26 = param 6
spline) — `crates/nf-game/src/ss.rs`. The closed hatch's posed matrix translation is the
placement spot (y 4.76), its collision mesh solid at y ≈ 4.74 right above the ladder top [D
probe, the ladder-underwater-02 report].

## The game opens it with the use key [M]

- `Player_Activate` 0x1a5170 has **no substate gate**: the creepwall arm needs
  `BLData+0x95f == 6` only (0x1a5170:66); then the pick is a radius-1.0 sphere at
  `Player_GetHeadPos` + one unit of the view direction (`Mat_GetDir` of the viewer matrix)
  against the collision world (`obj+0x20` cel + the dynamic list at `BLData+0x7e8`,
  `Collide_SphereIntersect` 0x1a5170:100-108), and every hit object walks its class byte
  (+0xff): case `'%'` (0x25, SimpleScript) → `SS_Activate` (0x1a5170:138-141).
- The pick is live on the ladder: `Player_Update` 0x193600's `obj+0xF4 == 1` branch (the climb
  tick's branch — the port's `climb_tick` doc) runs `Player_WeaponFiring` 0x1a5460 every tick
  [M 0x193600:413], which reads the use action `Input_Action(slot, 0xe, 4)` and calls
  `Player_Activate` (0x1a5460:393-402).
- `SS_Activate` 0x1a9ea8: `if (ext+0x22 != 0 && ext+0x1E & 4) { ext+0x22 = 1; if (ext+0x10)
  switch_channels[ext+0x10] = 1; }` — the armed state (0x40) + the E mask bit snap the state to
  1 (the playhead sweeps to the last key) and latch ch 2. The sweep runs in `SS_Update`
  0x1a9948's state-1 arm; the one-shot wrap (param 1 = 1) disarms to 0 holding the open pose
  [M 0x1a9948:119-128].

## Channel 2 has no other writer [D]

- The main map's 36 triggers (nf_game::triggers::build): no `out == 2`.
- Every Nis of the level's packs (07800002.bin, 07900002.bin), every decoded stream command:
  no channel-set event (kind 0xD [M `Script_EventHandler` 0x1d94c0]) writes channel 2. The
  intro's only write is `[0x47, 1]` [D 0x06000084, hatch-02-intro].
- The SS's own write-back (mask bit 0 clear → `SS_Update` echoes the operate result into a
  0-valued channel [M 0x1a9948:18-23]) only ever re-writes what the latch put there.
- The doors near the far end (#268/#330/#331) carry param 9 = 2, but door param 9 is a FLAGS
  word (`par(8) | par(9)`: bit 0 spline, bit 2 starts-open — `crates/nf-game/src/door.rs`), not
  a channel. They are not hatch-driven.

So the use key is the only opener — from either side of the hatch (the bathhouse side or the
ladder top), exactly like the other E-usable grates (#272/#440, grates-02).

## The plates at the ladder top are the Bond moment, not a hatch drive [D]

TouchOnce #500 (out 61) at (−49.71, 4.45, 5.46) and #501 (out 62) at (−49.71, 4.94, 5.46) —
stacked on the climb-out path, mask 0x8 (player). SEQ_AND #396 (type 0xED, kind 4,
`Trigger_Update` 0x1b1a78 case 4) takes [61, 62] in order onto **out 101** — the Bond-moment
band (0x65–0x6E). Climbing out through the opened hatch passes both plates in order and fires
the moment; the closed hatch blocks the climber at y ≈ 3.87 (capsule top 4.46 under the 4.74
panel), below #500, so the moment only fires through the open hatch.

## The port

`fix-npc-doors-escort` (2026-10-13, master ac86a176) ported `SS_Activate` (`ss.rs Ss::activate`)
and the use-key pick arm for class 0x25 in `nf_view/src/scene.rs` (the nearest E-armed SS under
the head ray within 2.2 — the reach stand-in shape the doors use; the game's pick is the sphere
vs the mesh). `nf_game::ss`'s state machine (grates-02) sweeps and holds the open pose; the
posed matrix joins the set_dynamic collision world each tick (`drone.rs script_parts`), so the
open hatch frees the shaft. The full real path — swim in, grab at root 3.0, climb to the wedge,
look up, use key, sweep (state 1 → wrap 0, ch 2 latched, open pose), climb out through both
plates (61, 62, moment 101), dismount grounded in the bathhouse (y 6.64), walk away — is pinned
end to end through `nf_view::load` + `Sim::step_bond` in
`crates/nf-view/tests/bath_hatch_02_live.rs`.

## Live checks for the main agent

PCSX2, 07000002, fresh boot (the save-state resume skips the boot path):

1. The deep-ladder exit: swim the undergarden tunnel to the ladder, climb up, look up at the
   closed hatch, press E — expected: the hatch sweeps open (150 ticks) and the climb tops out
   into the bathhouse. If the game's E does NOT open it from the ladder (the mesh-sphere pick is
   r 1.0 at head+dir — at the wedge the sphere's closest approach to the panel is ≈ 1.2), the
   port's 2.2-ray reach stand-in is looser than the game's pick and needs the mesh test; record
   the stance (root y, pitch) that works.
2. From the bathhouse floor: use the closed hatch from above — expected: the same sweep and a
   now-open shaft below.
3. The Bond moment: after the exit, the moment 101 (the "climb out of the flooded undergarden"
   bonus) should award; if it does not, the plates' touch order differs from the climb path.
