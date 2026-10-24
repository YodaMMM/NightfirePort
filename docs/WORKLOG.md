### level-0a-office-hatch-5 (started 01:41 UTC, 2026-10-24)

Task: 0700000a — the post-spawn settle (the game's spawn ray from (−3.9319, 0.1, 10.3978) hits
y 0.0000 in the spawn cel, surface 0x15 [L fresh boot, Collide_RayIntersect breakpointed from
build_PointOnFloor] — the SAME floor the port finds — so Player_StandAtNewPosition stands 1.1000
in both, and the snapshot's 1.0891 [L ee_start_0700000a 0x18694e0] is the AFTER-spawn settle:
−0.0109 from the collision push against the floor at the capsule's cc 1.0347 band, the
feet-on-point settle, the anim-driven cc [M Player_Collision 0x194df0 reads AnimObject+0xcc
every tick]; the -0.0109 surface hunt is moot). Then the hall glide (game 1.1147 after the
doorway-base climb at x −8.304) and the corner at (−17.87, 4.87) (the game walks through at
y 1.127 → 1.120); un-ignore the three sisters and the main walk in walkreal_0700000a_live.

Plan: log the port's root per tick from the spawn with no input and find the lowering step;
port it [M]; then the hall glide line, the corner push landing, and the un-ignores.
