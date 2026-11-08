# agent-cable08: 07000008 zip line drawn at the game's matrix

**Change.** The wire placements (type 0x3E) now draw at `View_RotTransMatrix` with the rotation's z
negated, as `Wire_Create` 0x1981d8 and `control_create_object` 0x1344e0 build `obj+0x90` [M]
(`scene::wire_draw_matrix`). The old draw used the plain placement matrix with rot.z as stored.

**Effect.** 07000008's OBJECT_Zip_Line_1 (rot.z 0.2094 [D]) sloped the wrong way: its far end sat at
y 43.58, under the low building's roof. It now runs from the castle wall (110.40, 45.29, -101.12) UP to
the wall fitting at (133.94, 50.25, -101.09), matching the PS2 capture from (133.75, 48.97, -97.30).
The ride line (`Wire::zipline_segment`, already from the decompile) runs (110.50, 44.44) to
(134.02, 49.43), so the drawn cable and the ride now agree. Yaw-only wires (07000002, 07000005) draw
unchanged (same endpoints to float precision).

**Test.** `cable08_endpoint_live`: pins both drawn endpoints from the disc placement; fails on the old
matrix (far end y 43.58), passes now.
