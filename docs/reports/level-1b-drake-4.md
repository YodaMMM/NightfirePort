# level-1b-drake-4 — 0700001b: the scripted carry into the shafts, then Drake

Task: on master the zero-G float works, and the wall at x 4.433 is the game's own plc #128
OBJECT_Boundary_1 (body-solid, bullet-pass) [D probe1b_wall]. The shaft mouths are two booths at
(5.23, 9.45, ±8.82): a glass pane (mat 0x40, bullet-pass) at x 5.91 and iris doors plc #30
OBJECT_IrisDoor1 / plc #32 OBJECT_IrisDoor3 (kind 0x2031). Work in progress — updated as steps land.

## Plan

1. Find the mechanism that carries Bond past the boundary into a shaft: the ch-74 pulse candidate, the
   iris doors' opening channels, any script stream with a player-carry event [M Script_EventHandler
   0x1d94c0]. Port it.
2. The landing, the fight gate, the defeat — real input only; un-ignore `walk_arrival_to_drakes_defeat`
   and `drake_defeat_by_real_fire_ends_the_mission`.
3. The interactables kill-channel gap: walker-driven kill leaves ch 0x0e off while a raw step_bond kill
   raises it — find why.
4. If a live read is required: name the address and the moment (fresh boot) and stop there.

## Status

- (in progress)
