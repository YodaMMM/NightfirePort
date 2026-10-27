## level-4a-ride-cast-b (2026-10-27)

Branch s/level-4a-ride-cast-b from origin/s/level-4a-ride-cast (the plan-only session;
master base ec917093). Task: find the mechanism that places the 0700004a ride landing
cut-scene's cast (pack 0x060006cf, SP #289) in the car despite the car-local sequence roots
(0x040005fd root (2.35, 0.85, -0.64) [D]) — Script_AnimStart 0x1d8f40 creates the actor at
the origin and Script_Interp 0x1d9a08 case 2 composes no base [M] — port it exactly, keep
every other cut-scene's cast where it is.

## level-4a-ride-cast-b (2026-10-27, the placement decode)
The anim-object placement decoded to the metal: obj+0x30 = the sequence's root track
(AnimGetFirstTrans 0x1c7348 at start + AnimFrameResolve 0x1c35f8's delta), Script_Interp case 2
composes no base (0x1d9a08:140-199), the anim-start GetInterp carries the SP-base flag but its
output is dead (disasm 0x1d90b4-0x1d9118), the palette rides obj+0x90..0xcc
(psiBuildMatrixPalette 0x111498), and the parent pointer obj+0x1c is never set for script anims
(Control_CreateObjEx 0x1343c8:26 is its only writer). The car-local roots pinned [D]:
0x040005fd's root ch0/1/2 = (2.35,0.85,-0.64) flags 0x0; facial 0x040006e6 flags 0x8. The
compose site is the live check (the guard's obj+0x30/0x90/0x1c mid-ride vs the 07000014 cast's)
— no behavior landed, every other cast stays put. nf-game lib 578/0; ride test 1/0; the targeted
suite set 31/0/4i (cutscene_models, ride_cutscene, cutscene_08, walkreal 01/0a/05/1b).
