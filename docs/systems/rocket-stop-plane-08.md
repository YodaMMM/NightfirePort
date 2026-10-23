# The 07000008 rocket's stop surface at z ≈ −96.7 (the corridor-end shutters)

GAPS G-049 follow-up (rocket-stop-plane-08). The PS2 rocket fired down the start corridor at the
end window usually explodes at z ≈ −96.2/−96.71 (normal +z) and sometimes passes to the wall at
−100.0 (material 0x0a) [L Explode_Create 0x17cbd0 traces, ra 0x12d0c4 = `Bullet_CollisionHandler`'s
0x4 arm, HITDATA+0x40]. This doc settles which object owns that plane and why the port's probe
missed it.

## The owner: the closed shutter doors (placements 49–52) [D]

The corridor end wall is `Room_Outside_1` (main map 51, object 27) — its collision plane at z
−96.75 (mat 0x0a) tiles x 136..142, y 48.6..52.8 but leaves a WINDOW HOLE at x 138.35..140.72,
y 49.50..51.54 with an arched top to ~52.8 [D `nf-extract coll-scan 139.52 50.77 -96.7`]. The hole
is filled by four kind-0xdb door placements, all with hashcode 0xffffffff (by-index), sounds
285/284, group 6, flags 1|34 (spline + keep-open, starts CLOSED):

| placement | object | glist flags | collision | closed-pose coverage (world x) |
|---|---|---|---|---|
| 49 | 45 `DOOR_Shutter_1` | 0x0 | 262 tris, mat 0x03 (wood stiles/frame) | 138.35..139.54 |
| 50 | 46 `DOOR_Shutter_3` | 0x0 | 262 tris, mat 0x03 | 139.53..140.72 (mirrored, 180° quat) |
| 51 | 47 `DOOR_Shutter_2` | 0x201 | 18 tris, ALL mat 0x10 (glass) | 138.44..139.44 (in leaf 1) |
| 52 | 47 `DOOR_Shutter_2` | 0x201 | 18 tris, ALL mat 0x10 | 139.63..140.62 (in leaf 3) |

[D `nf-extract coll-obj 51 45/46/47`; the placement dump]. The closed pose is key 1 (pos01 0.5) of
each placement's 0x19 anim (main-map `path_data` 4..7 = the block-dump files 7..10, 3 keys each):
all three keys carry the placement translation; key 1's quat is the identity (leaf 1/pane 1) or the
180° yaw (leaf 3/pane 2), so the closed leaves stand IN the window plane, the glass insert riding
inside each leaf [D keys; `door.rs` `Doors::create` poses kind-0 doors at key 1, pos01 0.5]. The
leaf is 0.09 thick (z_obj ±0.045): the closed front face lands at z −96.706, the pane plane at
z −96.753. The open extremes (keys 0/2, quat y ±0.9962 ≈ ±170°) park the leaves flat against the
wall BESIDE the window — the pose the raw placement matrix alone suggests, which is what a
placement-space probe sees and why the window looks open there [D keys].

## Why the rocket stops or passes [M + D + L]

`Bullet_CollisionHandler` 0x12ce80's walk per hit [M 0x12cee0..0x12cfb8]:
- the pane's surface byte is 0x10 (glass) with NO 0x80 bit [D coll-obj] — the glass rule passes the
  round because the rockets' `weapon_data+0x6c` = 0x12204 (0x2c) / 0x1a200 (0x2d) carry no 0x800
  [L ee_start_05.bin @0x2bf150], and NO `Rand_Rand(0x3f)` is drawn (the byte has no 0x80);
- the wood byte 0x03 has no glass/0x40/0x80 bits — the walk ends at the first hit, no draw;
- a stop explodes at HITDATA+0x40 (the hit point) with script 0x6000052 [M the 0x4 arm at
  0x12d0ac..]; the rocket's flight light record rides the rocket's own position one tick behind
  (~0.55 units at the rocket's speed) — the live reads "−96.71" (hit + explode) and "−96.2" (the
  light) are one event, not two surfaces [M 0x12e7b0's light arm; L the light at (139.55, 50.77,
  −96.15)].

So the 2-of-3 stop / 1-of-3 pass variance across the live shots is AIM variance across the
assembly (wood stile vs glass pane), not the surface byte's 0x80 roll — the roll never fires on
this window. A stopped round's numbers: hit z −96.706, normal +z — the live −96.71. A passed round
continues through the pane to `BlockingPoly_PlayerMissile3`'s back wall at −100.08 (mat 0x0a) —
the live −100.0 [D cage triangles; L].

## Port status

The port already carries the same collision and law — no code fix was needed; the earlier probe had
been taken against the static/pre-tick world (the doors enter `CollisionWorld`'s dynamic set on
the first tick — `world.rs` `set_dynamic` via `drone.rs`, closed key-1 pose from `door.rs`
`Doors::create`, the MAX-hashcode shapes indexed under the synthetic `0x8000_0000 | map << 16 |
obj` keys at world.rs 317–338). Pinned by `crates/nf-view/tests/rocket_stop_plane08_live.rs`
(4 tests): the straight fire path pierces the pane (mat 0x10 probe) to the −100 wall; the steered
line probes the wood stile (mat 3, z −96.706 ± , normal +z); `penetration_roll_node` draws nothing
on either surface (0x10 passes by the glass rule, 0x03 stops); Bond is held outside the closed
shutters (they are doors — the game's actor queries test the cel-linked objects' collision [M
`Collide_PickObj` 0x1eae80]).

Tooling added: `nf-extract coll-scan <iso> <level> x y z [r]` (spatial collision owner probe over
placed + raw collision + draw verts) and `nf-extract coll-obj <iso> <level> <map> <object>` (one
object's whole collision block).
