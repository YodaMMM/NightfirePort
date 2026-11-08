# agent-gadgets: on-foot gadgets against the game

## Fixes

1. **The Micro-Camera takes a photo only while aimed.** `Player_WeaponFiring` 0x1a5460:512-514
   [M]: with the fire held, a record of base 0x54 (the pair 0x54/0x55) returns before the pull
   unless the aim bit (`anim+0x60 & 1`, raised when the aim-raise script stops [M 0x1a2e38:554-561])
   is up. The port photographed on a held fire without aiming (the press's edge toggles the pair,
   and the fire held after that switch took the photo). The weapon matrix's camera cell now holds
   the aim, the way a player takes the photo.
2. **The Stunner no longer tases through walls.** `Player_WeaponInitBullet` 0x1a4bf0:152-200 [M]:
   the target is the FIRST hit of the head → aiming-point ray; its object must be class 2/3 inside
   the record's +0x1C (5.0 / 10.0 [L]). The port's sphere stand-in ignored the world, so a guard
   behind a wall within 5 units drew the beam and drained the charge. A wall in front of the
   guard now makes the tick the game's free miss (no beam, the spend refunded).

Test: `crates/nf-view/tests/gadget_use_gates_live.rs` (0700000c camera, 07000005 roof guard #2
behind the wall west of it, with the clear east side as the sister). Both red on the old code
(1 unaimed photo; the guard behind the wall a target).

## Checked, same as the game

- Gadget ids and names from the disc [L USATxt + weapon_data]: 0x4A/0x4C Stunner (clip 100,
  range 5/10), 0x4E/0x4F Laser (clip 500), 0x50/0x51 Grapple, 0x54/0x55 Micro-Camera, 0x56/0x57
  Decryptor (range 2/4), 0x58 Q-Worm (range 2), 0x59/0x5B Shaver (+0x5A/0x5C its detonator),
  0x37 Remote Mine (+0x38 detonate), 0x3A Laser Tripbomb, 0x3B..0x40/0x6C Satchel Charge.
- Charge regen `Player_Update` 0x193600:380-408 [M]: Stunner 0x4A and Laser 0x4E +1 every 4th
  frame, 0x4C/0x4F every 2nd, toward the clip size, only outside view state 9. Port matches.
- The camera's 0xEC shutter (0x1a4bf0:261-263) is already played by the port's fire_player.
- Remote-mine detonator (−10.0 fuse on every placed mine), satchel fuse per twin
  ((id−0x3b)·5+5 s, 0x6C 900), decryptor use gate: ported as the decompile reads.

## Not done here

- The Q-Worm use gate (0x1a5460:530-549, PC type 0x49 in state 0 within 2.0) is already on
  `s/agent-lvl-09-0a`, not yet on master; left to that branch.
- No gadget named a car-key remote exists in the disc's weapon names; nothing to compare.
- Night/thermal vision (goggles.md) and the Q-Claw were not re-audited (Q-Claw belongs to another
  task).

## Unproven

- The Stunner's guard shape is still the port's 1.05 sphere at the guard's position, not the
  guard's own hit capsule; only the wall test is new.
- The camera gate uses the port's aim bit; `Check_Target`'s every-6th-frame cache is not modelled.
