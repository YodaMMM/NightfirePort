# Large object updates read branch by branch

Scope: `Sensor_Update` 0x1af618, `Lock_Update` 0x1ae3d8 (with `Lock_Activate`, `Lock_Unlock`,
`Lock_ReLock`), `Door_Update` 0x1363c8 (no change; with `Door_SetState`, `Door_Interp`, `Door_ReactToHit`,
`Door_Activate`), `Searchlight_Update` 0x1cb900, `GT_Update` 0x180498, `SP_Update` 0x1dbda8 (with
`SP_Hit`), `RB_Update` 0x1d6e38, `Copter_Update` 0x17dbb8. Every branch of the decompile against
the port arm. Copter homing code untouched.

Random stream: one fix touches a draw. `RB_Update`'s buoyancy arm now skips every sixteenth tick,
but no rigid body on 01, 02, 03, 04, 06 or 0d stands under a water surface at load [census test],
so the stream does not move on those levels. Everything else here adds or removes no draw, except
that a turret or a lock can now die or open at a different time than before.

## Fixed, biggest player impact first

1. Wall guns and the deployed Ronin can be shot down. `GT_Update` sums the damage on every object
   of script A, and of script B in state 4, into ext+0xCC [M 00180498:128-176], and the death gate
   takes the gun to state 6 [M :236-258]. The port had no bullet feed, so only blasts reached
   `apply_damage`. Rounds now hit the scripts' objects, stop on them, and a wall takes the round
   first. Record 96 rounds never count against a turret. 0a x6, 0b x5, 12 x3 [D].
2. The laser no longer burns keypads. The 0x4E/0x4F arm is kind 1 only [M 0x1ae3d8:47-101]; kinds
   0/2 read 0x56/0x57 [M :162]. The port burned any lock in the beam, which skipped the keypad or
   decryptor on 0a, 0b, 11, 12, 14 and 15.
   Keypads still open: 0a, 0b, 14 and 15 own the decryptor (0x56) at level start [census test]. 11
   and 12 do not on a cold load, but each has a Q-worm whose param 0 is the lock's code (#216 code 1,
   #684 code 5 [D]), which reveals the code to type. Nine tests pinned the laser route and now type
   the code; one test's comment had called 0x4E the Q-Decryptor. Two more pins followed the game:
   12's decryptor test now expects #694 to open with #656 and re-locks both before its second scan,
   and 14's #776 sister accepts the Operative create-unlock reaching state 2. No other laser path exists:
   `Lock_Unlock` has no caller outside the lock functions and `Player_LaserPointer` reads no lock.
3. The key ring now drives the locks (`Lock::key_tick`). `Lock_Unlock` equips the code
   [M 0x1b0650:19]; case 0 opens a closed kind-0/2 lock with a code when the ring holds its key
   [M 0x1ae3d8:139-147]; `Lock_Activate` opens any kind with the key held [M 0x1aed00:19-33] and
   takes a press in state 5 as well as 0. Player effect: 0a #646/#729, #648/#732 and 12 #656/#694
   are pairs on two door faces [D]. Solving one now opens the other, where the port kept the second
   keypad live. 46's key pickups #10/#56 open locks #9 (use press) and #37 [D]. Keys carry between
   mission parts already (`RamSave`), so 14's code 2 opens 15's #381 as `Player_RamLoad` allows.
4. State 1 of a create-unlocked lock now ends. Master's re-lock pass (`update_burnt_open`, state 2)
   already handles the drop; the Operative create-unlock sat in state 1 for ever because nothing
   moved it on. An empty framelist ends state 1 after its one channel write [M 0x1ae3d8 case 1], so
   `key_tick` now moves it to state 2. The re-lock then works for every unlock, and the pair test
   drops channel 0x31 by hand to show it. Which script drops a lock channel on a real run is not traced.
5. Security cameras are blind during a cut-scene. `Sensor_Update` skips both `Sensor_InCone` calls
   while `glb_viewer[4]+0x29` is set [M 0x1af618:141-143, :204-206]; only `Script_CameraStart`
   writes that for viewer 4 in play [M 0x1d9140:46]. The port kept counting.
6. Searchlight spot ray no longer lands on Bond. Mask 0x18 skips the dynamic walk and the
   ForcedList [M 0x1eb320:314-328, 370-374]. The port added a sphere test of its own, so Bond up in
   the beam armed the light; now only the ground spot within 2.0 arms it [M 0x1cb900:251].
7. Searchlight alarm write is guarded [M 0x1cb900:146-148]. All seven 05 heads carry alarm 0 [D],
   and the port raised channel 0 while tracking. The lost-sight limit is strict, `900 < n` [M :259].
8. `RB_Update`'s gate is `GameState+0x34 & 0xf`, the frame counter [M 0x1d6e38:27-28]. The port read
   the level code.

## Read with no fix needed

- `Sensor_Update` state machine, `Sensor_InCone` (the range read is sensor+0xC0 to player+0x80, disasm
  0x1af534), `Sensor_SetBeam`.
- `Door_Update` open-channel arm, auto-close, group OR, `Door_SetState`, `Door_ReactToHit`.
- `SP_Update` all eight states, the out-channel write and the damage tail. `SP_Hit` bits 8 and 0x10
  (player and drone contact) have no port arm, and no main-map SP mask uses them: masks are 0 (40),
  1 (110), 2 (3), 4 (19) [D].
- `Copter_Update` all states and the damage tail.
- `GT_Update`'s stun test (ids 0x4A/0x4C to `GT_Disable`) reads the class byte of the script object,
  not of the hitter: `lw a0,0xd0(s0)` then `lbu v0,0xff(s0)` on the same s0 [M 0x180564, 0x180584].
  A script object is not class 5, so no bullet disables a turret. Not ported, on purpose.

## Recorded, not changed

- `Door_Update` lock edge: the game skips it while the busy marker is up [M 0x1363c8:98-100]; the port
  does not. It needs the open and lock channels to rise on one tick. Door #213 on 11 uses 90 for both,
  but its own `Door_SetState` clears 90 before the lock block reads it, in the game and the port
  alike. Door #251 on 07 (130 and 200) is not closed at load. No reachable case was found, so no change.
- `Door_Activate` doorway arm: the game skips the 2-tick timer store [M 0x135be0 LAB_00135ef8]; the
  port sets it. It only gates `Door_ReactToHit`, no player effect was found and no test can show one.
- `Door_Interp` kind 0: the game clamps only while OPENING [M 0x135738:33-44]; the port clamps in any
  state, by an earlier fix for a runaway. Same result unless a closing leaf leaves 0..1.
- Searchlight relight: the game goes straight to the contact walk [M 0x1cb900:399-406]; the port runs
  one part update that tick. The off-kill tick also skips the contact walk. One tick each.
- Copter smoke: `Rand_FRand` takes 0x40c90fdb = 2 pi [M 0x17dbb8 smoke arm]; the port passes pi. Same
  draw count, smoke spin only. Visual, so left.
- `Lock_Update` case 5 draws `Rand_Rand(10)` on odd scan ticks for the scrambling digits. The port
  does not draw it. Adding it moves the shared stream, so left.
- Lock burn sparks (0xc00000a per contact) and the kind-1 open framelist pose are not drawn.
- `GT_Update` case 4 with no target slerps the gun back; the port's `ease_pose` runs only with a
  target. The elided decompile arguments were not traced, so the target pose is unproven.

## Unproven

- Whether any script drops a lock channel on a real run: re-lock is tested by dropping the channel
  by hand.
- Whether `Player_RamLoad`'s warm path carries the key ring on every part change the port treats as
  warm. `RamSave` carries it; the 14 to 15 case was not walked.
- Turret contact uses the shared `sp_bullet_targets` spheres, not the objects' collision models.
- Drone rounds do not damage turrets; the port's drone bullet pass stays out of this read set, as it
  does for SP damage.
