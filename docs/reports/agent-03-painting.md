# agent-03-painting: the dark slab with a white rim in Mayhew's bedroom (07000003)

Result: no sim change. The artifact is not the painting. It is placement 502's neighbour,
map 50 placement 495 `HOTOBJECT` (kind 0x20, glist flags 0x421), an x-ray-only mesh that the
posed breakable pass draws in normal view. Only the draw is wrong.

## How it was found
Headless shot `--bond 12.13,6.28,-40.63,13.000,0.049 --frame 240`, with a temporary per-object
batch skip (not committed):
- skipping every batch of `bedroom_triggerpicture` (obj 342) removes the Fuji painting to the
  right of the crosshair; the dark slab and its white rim stay;
- skipping `door_secret_bedroom` (placement 4) opens the passage behind the slab; the rim stays;
- skipping `HOTOBJECT` (placement 495, its tex 242) removes BOTH the dark slab and the white rim;
  the secret door then reads as plain wall and the painting is unchanged.
A vertex sweep over every map placement in the box x 13.3..16.1, y 5..8.4, z -35.8..-35.0 [D]
finds only the corridor/bedroom room meshes, the door, and HOTOBJECT: two tex-242 strips at
z -35.50 framing the door (x 13.44..15.91, y 5.25..), vertex RGB ~255 with alpha 51 to 0.

## The game, at load [M]
- `SS_Create` 0x1a9c58 for placement 502: key count (level_tag+2) = 2, rate (+4) = 0, so 75
  (`(2-1)/75` per tick), state p0 = 0x40, wrap p1 = 1, mask p2 = 1, channel p3 = 0x31, sound
  p4 = 0xFFFF, rest p5 = 0. `Control_SetGList(obj, glist)` takes the placement's own object
  (map 50 obj 342), and obj+0xFA |= 0x44 draws it at key 0. Key 0 equals the placement
  transform exactly [D path block 0x14]. The port's `SsList::create` gives the same state, the
  same key-0 matrix and the same object. Already pinned by
  `interactables_07000003_live::the_e_plate_unlocks_the_proximity_door_and_swings_the_picture`.
- Texture 15 [D]: 32x32, 4 bpp, 16-colour palette, all palette alpha 0x80 (opaque), dark brown
  (21,13,11) with a lighter centre. Batch 0 of obj 342 uses it for a 14-vertex ring on the
  wall-side plane (z +0.043, faces -z like the canvas), vertex RGB 0 with alpha 0 outside to
  115 inside, ALPHA_1 0x44: a soft drop shadow on the wall around the frame. The port draws it
  dark, as it should.
- Flag 0x80000 (glist +0x44, copied to obj+0xF0 by `Control_SetGList` 0x134960): the near-flash
  tint. `View_SetupRenderModes_Tweak` 0x1e5888:22-44 scales the object's tint RGB by `FlashMod`
  (a 0.5..1.0 shimmer) while the options byte DAT_002a38d4 is set and the viewer's eye is within
  4.0 of obj+0x80 (squared distance < 16). It can only darken, never whiten, and the shot's eye
  is 5.2 away, so it plays no part in this artifact.
- Flag 0x400 (HOTOBJECT's glist): `View_AddObjects` 0x1e62d8:38-47 (and
  `View_AddForcedObjects` 0x1e5fb8:79-84) submit such an object only while the drawing viewer's
  +0x236 == 2 (x-ray goggles). `Break_Create` 0x12c230 copies the glist flags through
  `Control_SetGList`, so the 0x20 HOTOBJECT is x-ray-only: in normal view the game shows nothing
  there. A census over every `nf_view::dev::LEVELS` main map [D] finds it is the only non-cel
  0x20 placement whose glist carries 0x400 or 0x10.

## What the draw must do (not changed here: draw work)
`crates/nf-view/src/scene.rs`, the posed breakable slot loop after the SimpleScripts
(`for p in ... filter(|p| p.flags & 0xff == 0x20)`): after resolving the object, `continue`
(keeping the `bi`/slot numbering) when `obj.flags & 0x400 != 0` or `obj.flags & 0x10 != 0`,
the same gate the grapple and generic dynamic sweeps already apply. The x-ray slot pass
(`XRAY_SLOT_BASE`) already draws HOTOBJECT while the goggles are in x-ray mode, so nothing else
is needed. The sim's breakable list stays as is: the game creates the object too.

Secondary, unproven: the port applies the 0x80000 near-flash only to the armour pickups; the
picture (and `masterarea_dragon`, 0xd9, also 0x80000) would shimmer within 4.0 of the eye in the
game. Whether the TAmbient tint reaches objects without obj+0xF0 & 0x800 was not traced.
