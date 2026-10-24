# The Q-Decryptor (weapons 0x56/0x57): the chain

The Q-Decryptor did not work in the port. This document is the game's own chain, every step
addressed. Tags: [M] decompile, [D] disc bytes, [L] EE snapshot / live measurement, [S] suggested.

## The weapon

- **Ids 0x56 (base) and 0x57 (upgrade).** Name label 0x0500004e; range `weapon_data+0x1C` =
  **2.0 / 4.0**; flags `+0x68` = 0x20a0, `+0x6C` = **0x0000** (no laser/taser beam bits — the shot
  is a plain bullet, `Draw_LaserBeam` never runs for it), `+0x70` = 0 [L ee_start_05.bin @
  weapon_data 0x2bf150 + w·0x10C]. The selectable flag (`+0x04`) is 1 for both [D].
- The older docs/port-map/gadgets.md table (0x4E/0x4F = "Q-Decryptor") is the shifted set that
  fix-qclaw corrected; gadget.rs's header carries the settled mapping.

## The use gate — why the decryptor "does nothing"

`Player_WeaponFiring` 0x1a5460:551-578, on the decryptor's fire tick:

1. The **aim target** `BLData+0x880` must be non-null. It is written by `Check_Target`
   (0x1a7810): every 6th frame (`GameState+0x52 % 6`, :100), unless the player's substate is
   2/4/5/8..0xe (:69-79), it casts the head→view-direction ray **50.0 units** (0x42480000,
   :114-116) through `Collide_RayIntersect` (mask 0xc, :121) and stores the hit's object
   (`HITDATA+0x58`) into `BLData+0x880` (:125), zeroing it first (:107).
2. The target's **type byte `obj+0xff` must be 0x3a** — the lock class, set by `Lock_Create`
   (0x1ae1f8) on every lock (:`*(undefined1 *)(iVar8 + 0xff) = 0x3a`).
3. The lock's **record u16[+0x00] must be 0 or 2** (the decryptor-compatible kinds; 1 = the
   laser-burn lock) and its **state `obj+0xf4` must be 0 or 5** (0x1a5460:559-566).
4. The **distance** `Vec_Dist3D(player+0x30, lock+0x30)` must be ≤ `weapon_data+0x1C`
   (2.0/4.0, :574-577).
5. Any check fails → **return before `Player_RoundToFire`** — no shot, no sound, no UI. This was
   the port's symptom: there was no `+0x880` aim target at all, so the decryptor
   could never fire.

## The shot and the contact

`Player_RoundToFire` → `Bullet_init` 0x12c5a0 sets the bullet's **class byte `obj+0xff` = 5**
(:87) — bullets are the class-5 objects `Lock_Update` scans for. The bullet carries its weapon
id where `Lock_Update` reads `**(short **)(bullet+0x44)`. The contact system adds the bullet to
the lock's contact list (`obj+0xd0`) while it overlaps the lock.

## The decode — `Lock_Update` 0x1ae3d8

The lock's record words (from `Lock_Create`): `u16[+0x00]` = placement **param 0** (the kind:
0/2 = decryptor, 1 = laser-burn), `u16[+0x02]` = **param 1** (the KeyCodes index), `u16[+0x08]`
= **param 5** (the out switch channel), `level_tag+0x44` = param 6 (the Operative auto-unlock).
(The param k ↔ `level_tag+0x2c+4k` mapping is the one pinned on Drone_AIVolume_Create and
FuseBox_Create.)

- **State 0** (case 0): if `record[0]` is 1 → the laser arm (ids 0x4e burn 3/tick, 0x4f 10/tick,
  :57-65). For 0/2: if the player's code-known byte (`glb_blokes[player] + record[1] + 0x70c`)
  is set → unlock at once (:142-148). Else every class-5 contact bullet whose id satisfies
  `id − 0x56 < 2` (i.e. 0x56/0x57, :162) adds **+1** to the lock's progress `obj+0xf6`; any
  contact → **state 5**, progress reset to 0, the display globals filled: the true code copied
  from `KeyCodes[record[1]·5]` into 0x2a37d8, digit count 0x2a37e2 = 4, decoded 0x2a37e3 = 0,
  display 0x2a37dd[0..3] = '*' (:179-189).
- **State 5** (case 5): the scan-active flag `DAT_002a37e4` = 1 (:295). Every contact bullet:
  id **0x56 → progress +1**, id **0x57 → +3** (:305-315). No contact bullet → progress **−4**
  (floor 0, cap 3000) and at 0 the scan resets: state back to 0, the flag cleared (:327-344).
  While scanning, the next unsolved digit `i` (capped at index 3) randomizes on odd progress
  (`Rand_Rand(10)`+'0' clamped '0'..'9', :351-363) and **solves when progress > (4−i)·0x28**
  (i.e. 160/120/80/40): display[i] = code[i], decoded += 1, progress reset to 5, sound 0x5db
  (:346-369). When **decoded == 4** → `Lock_Unlock` + sound 0x5da (:370-374). The all-4
  comparison is the disassembly's `lbu v1,19(s0); lbu v0,18(s0); bne v1,v0` at
  **0x1aebec-0x1aebf4** (`DAT_002a37e3 == DAT_002a37e2`) — the decompile's `DAT_002a37e2 + 1`
  is a decompiler artifact; the game unlocks at 4 of 4.
- **`Lock_Unlock` 0x1b0650**: the player arm equips key `record u16[+2]` and sets state 1;
  **state 1** (case 1) writes `switch_channels[record u16[+8]] = 1` + the timestamp (:193-196)
  — the door/objective channel — plays the open anim (framelist), then state 2.

## The codes — `Locks_Init` 0x1b0548

51 codes × 5 bytes: four digits `('0' + rand%9)` (so 0..8) and a display flag 0; generated at
init with the shared RNG after a `GameState+0x52 · 10 & 0x1ff`-draw warm-up. `KeyCodes[i·5+4]`
is set to 1 by the PC Q-Worm (0x1aee20) after a PC hack — the code then displays on the keypad
menu. The codes are **display/cosmetic for the decryptor**: the scan completes on progress
alone, never by comparing typed digits (that is the keypad-menu path, case 3/4). The port does
not need the code values to open a lock; it keeps them out of pass/fail.

## The watch display — `Player_WeaponFiring` 0x1a5460:155-194

While a 0x56/0x57 is out: idle (no scan, `DAT_002a37e4` = 0) ticks the view entity's +0xf6 down
and zeroes the digit datums at 0; scanning, it draws `DAT_002a37dd[0..3]` (the display buffer:
'*' for unsolved-and-idle, random digits while progressing, the true digits once solved) onto
the view entity's anim datums 3−i via PdaDigits. This is the first-person "watch face" the
player reads — render-side, exposed as getters.

## What is NOT the decryptor's path

`Player_SSScanMode` (substate 5, 0x193098) is the scan-mode free-look camera (debug toggled at
0x193600:158; camera angles/zoom on BLData+0x30/0x34). No decompiled select/fire path enters
substate 5: `Player_ChangeSubState(…, 5)` has exactly one constant-5 caller (the debug toggle),
and neither `Player_WeaponFiring` nor `Player_WeaponChange` changes substate. The port-map's
"on select: enters substate 5" was a guess and is wrong for the decryptor chain; the decode
runs entirely in `Lock_Update` while the weapon is out and firing.

## Port model

- `Lock` gains `kind` (param 0), `code` (param 1) and the scan state (`scan_progress`, solved
  digits, unlocked) with `decrypt_tick(beams, upgraded)` — the state-0/5 machine above.
- `gadget::is_decryptor` (0x56/0x57) + the range read; `decryptor_tick` on the real fire path in
  scene.rs: the 50-unit crosshair ray vs the locks' world spheres (`Check_Target`'s stand-in for
  the +0x880 target — [S] the sphere for `Collide_RayIntersect`'s object hit), the type/kind/
  state/range gate, then +1/+3 per tick while held; no fire → −4 decay.
- The unlock writes the lock's out channel (the port's one-tick fold of `Lock_Unlock` → case 1,
  as the laser burn already does).
- The gate's target is the NEAREST lock the ray meets, whatever its state; an open or non-decryptor
  one refuses (the game tests the +0x880 target [M 0x1a5460:557-566]). Each door carries a keypad on
  both faces 0.26 apart (0700000a #646/#729, 07000012 #656/#694 [D]); skipping the open near one
  let a held fire start a second scan on the far face through the wall.
- The screen: `Sim::pda_display` / `pda_tick` keep the display buffer and the FRAME_RATE idle
  countdown; the view model's datums 0..3 carry every `PdaDigits` mesh (0x2000a09..0x2000a12, read
  from ACTION.ELF 0x2bedb8) and `frame_state` shows the one the display names. The code digits and
  re-rolls are the game's shared-RNG draws; the port shows them from a hash (cosmetic deviation).
