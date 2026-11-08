# agent-09-lift-call: what the 07000009 car-park call button's use press hits

Built on `s/agent-use-pick` (merged into this branch over master; the one conflict, the lock arm
in `drone.rs`, keeps master's key-ring logic with use-pick's pick). No game code changes here:
the port already targets the object the game targets. The branch adds one test.

## What the game's pick hits [M]

- The button art `liftswitch_off` 0x020006b2 is a sub-script entity (0x0600064F stream 2).
  `Script_SubScriptStart` 0x1db050 calls `Script_Load` with t0 = 0 (asm 0x1db0ec), and
  `Script_Load` stores t0 at SCRIPTINFO+0x80 (asm 0x1d826c → 0x1d8458). `Script_EntityStart`
  0x1d8d78 copies +0x80 to obj+0xE4, so the art has owner 0. `Player_Activate` 0x1a5170 skips a
  class-0 object with obj+0xE4 = 0. The art has 42 collision triangles and r 0.20, but it is
  never a target.
- No placement and no other class sits within 8 of the button except lights [D placements-all].
  The only owner-stamped entity in reach is the outer door leaf beside the button. For the left
  column (SP #22, script 0x0600007D) that is 0x020006a9 (stream 8). For the right column (SP #21,
  0x0600007E) it is 0x020006a8 (stream 7). Each leaf is class 0 (entity type 0x31, the default
  `control_create_object` arm of `Script_CreateEntity` 0x1d8968). The glist is a 1.07 × 2.70
  slab at z 26.28-26.35, with 50 triangles including the edge faces. Its world sphere has
  r 1.451 at the matrix translation [D].
- `Player_GetHeadPos` 0x195aa0 has no forward offset: it is the body position raised by
  +0x908 − +0x90c. The pick point is that head plus the view direction.

## The port

The use-pick branch's SP arm (`UseObjects` → `entity_use_picks` + `usepick::picks`) already
picks that leaf. When Bond stands directly in front of the button art (x = button x) and looks
at it, the press lands from a stand 0.6 to 1.1 off the wall (z 24.8-25.34). A walk straight up
to the button ends against the wall inside that range, so the press lands. It misses only from
about 1.3 off the wall (the old test's z 24.6 stand). There the leaf's edge is 1.06 from the use
point, and by the same code the game misses too. A miss "from in front of the button" happens
only from that far back.

## Test

`garage_lift_calls_live::a_walk_up_press_at_either_call_button_lands_on_its_outer_door`, for
both columns. Bond walks up to the button with forward input, looks at the art and presses E
once. The test checks four things:
- the pick keeps exactly the leaf beside the button;
- this column's hold releases;
- the other column stays held;
- the ON art lights within 20 ticks.

It is not red on the old code, because no port code changed. It is a pin of the traced behaviour.

## Unproven

- The exact far edge of the reach depends on the port's eye height. The real game's reach from
  in front of the button has not been measured live.
