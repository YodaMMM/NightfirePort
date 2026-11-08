# agent-lvl-09-0a: 07000009 / 0700000a level pass

Note: the drawn-ray change below can move the shared random stream on any level where a walking
guard is seen through glass (he now animates instead of sliding, so his anim-timed AI steps land
on different ticks). Kept on this branch.

## Fixes

1. **The patroller behind the 0700000a lounge window walks with his gait.** The guard is the SP3
   template 2 (released by plate #716 → ch 0x86 [D]), state 20 (patrol), danim 392 (walk): the
   animation state was right. The sim's stand-in for the renderer's drawn byte (`obj+0xFC`,
   written by `View_AddObjects` 0x1e62d8 [M]) casts bullet rays, and those stop on glass
   (surface byte 0x10 — the lounge pane at z −4.71 [D ray hit]). Undrawn, `NDrone2_DoAnimation`
   0x149f88 [M] slides a walking drone at the danim speed (0.02) with no gait, so he crossed the
   window as a statue. The drawn rays now skip glass too (`CollisionWorld::segment_sight_clear`).
   Test: `window_guard_0a_live` (red on the old code: 0.00 moved while drawn).
2. **The Q-Worm fires only at a free PC in reach** [M Player_WeaponFiring 0x1a5460:531-545]: the
   crosshair target must be a class-0x49 PC in state 0 within the record's +0x1C reach, or the
   pull returns before the shot. The port fired it anywhere. Test:
   `interactables_0700000a_live::the_qworm_only_fires_at_a_pc_in_reach` (red on the old code).
   Two 0a install tests moved their stand inside the 2.0 reach; the 0b console test's second
   stance now stays at the desk (its old teleport stand was 2.25 from the PC, past the reach).
3. Carried from the earlier run on this lane (rebased onto master with the security countdown):
   the warm load keeps the keyboard's security record, so the laser keyboard works on a run from
   the lobby; the Q-Claw latch stops on a closed lid; the hang turns and looks.

## Checked, no defect

- **The corridor camera near (11.32, 0.65, 6.38)** is sensor S1 (placement #627, (7.52, 2.76,
  17.43)): 5 keys, yaws 90 → 180 → 90 → 0 → 90, rate word 600 [D]. The game's step is
  `(count−1) · REC_FRAME_RATE · (FRAME_RATE / rate)` at create [M 0x1af010:89-96]; all three
  frame-rate globals come from one `GS_SetRefreshRate` [M 0x17d2b8], so the create-time 30 cancels
  (REC = 1/30, FRAME_RATE = 30) and the step is 4/600 either way. Play adds `step ·
  FRAME_RATE_MUL` per tick (MUL = 1 at 60) [M 0x1af618]. One cycle = 600 ticks = 10 s, 360° of
  swing = 36°/s. The port measures 35.9°/s over 1200 ticks. Same as the game.
- Channels: every trigger lock on both levels has a writer (09 ch 0x39 = SP #70; 0a ch 0x37 =
  the carried laser channel / SSys, ch 0x3A = the keyboard SP).

## Unproven

- Whether the PS2 keeps the window guard drawn the whole time he crosses the pane (portal walk);
  the port now treats glass as see-through for the drawn test only.
