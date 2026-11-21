# Driving: world handedness

The driving engine (DRIVING.ELF) stores its tracks and cars in a frame that is left-handed against the
on-foot renderer (`Camera::view_proj`: right-handed, screen-right = forward x up).

Evidence [D]:
- The Vanquish's rear number plate (vanquish.ssh atlas, u 0.75 to 1.0 at v 0.625 to 0.75) runs across
  car +x on the strip at car z -2.15. Seen from behind (looking along car +z) it reads left to right
  only when screen-right is +x, i.e. screen-right = up x forward.
- The track's poster boards carry front and back faces whose u runs the matching way for the same rule.
- With that rule the car starts in the right-hand lane of the Paris road (France drives on the right),
  and the left key (steer -1, yaw decreasing) turns the car to screen left.

Port: `Scene::mirror_x` (set by `driving::load_driving_scene`) negates clip x in
`Camera::view_proj_for`. No wgpu pipeline culls faces, so the winding flip needs nothing else.
Positions, collision and physics stay in the game's own frame.

Test: crates/nf-view/tests/driving_handedness_live.rs (fails with the mirror off).

Not checked: 3D sound panning for driving (not ported yet) must use the same rule.
