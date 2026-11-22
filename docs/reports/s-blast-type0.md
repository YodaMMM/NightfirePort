# s/blast-type0: the type-0 destroyable's point box

Question: does the port decode destroy[10] on 0700000d (placement #470) wrongly, given its box
is the point (0,0,0)?

Finding: no. The point box is the disc's.

- #470 (index 91, no hashcode) sits at translation (0,0,0), rotation 0, every param 0 [D].
- Its object, map 49 object 91 "bigdoor01", is an empty glist: min (9999999)^3, max
  (-9999999)^3 (the empty-bounds sentinel), centre (0,0,0), radius 0.0001, 0 collision
  triangles, 0 draw leaves [D].
- The game's bullet pick tests the flight segment against the object's world sphere (matrix x
  glist centre, glist radius) [M Collide_PickObj 0x1eae80:92-141, Control_BuildWorldSph
  0x133070]. For #470 that is a 0.0001-radius sphere at the origin, the same place the port
  puts its point box.

Census, Level::parse over dev::LEVELS: 189 0xE1 destroy leaves, 2 point boxes, both at the
origin, both bigdoor01 (#93 and #470, 0700000d). #93 is the C5 door's empty half; the game's
sphere for it sits at its placement (133.1, 5.67, -269.4), not at the port's origin. Left as is:
r 0.0001 makes it unhittable by bullets in either case, and its mask ~0x10 carries 0x20, so a
blast never smashes it [M Explode_Propagate 0x17c518 case 0x21]. Moving the port's ±0.1 point to
the door would make the door's empty half shootable where the game's is not.

No code change. The ignored test stays ignored, with the finding written into its comment.

Tests: fix_blast_scripts_live 3 ok / 1 ignored; walkreal_0700000d_live 2 ok / 2 ignored.

Unproven: whether the empty glist's +8 (collision) pointer is non-null, which gates the pick
[M 0x1eae80:88-90]; it does not change the result (r 0.0001 either way).
