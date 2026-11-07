# lock-relock-channel — a decrypted lock re-locks when its channel drops

Branch `s/lock-relock-channel` from `origin/master` (22d5836b7).

## What's done

1. **The case-2 tick is wired into the per-tick lock pass.** `Lock::update_burnt_open`
   (crates/nf-game/src/lock.rs) is now the full `Lock_Update` 0x1ae3d8 case-2 body in the
   game's order, and `Drones`' control-pass walk calls it for every lock right after the
   idle arm (drone.rs, next to the `idle_tick` loop — the two calls are the state
   dispatch's exclusive arms [M 0x1ae3d8:34, the switch]: a state-2 lock runs case 2 and
   not case 0, and the port's `idle_tick` early-returns on `burned`). The re-lock check
   reads the REAL switch channel — `self.channels.on(lock.out)`, the game's
   `switch_channels[record u16[+8]]` [M 0x1ae3d8:231].
2. **The case-2 body is complete.** The burn-byte refill was missing from the port's
   `update_burnt_open`; it now runs first, in the game's order [M 0x1ae3d8:221-227]:
   +0x105 += 2 under the game's `v + 2 < 0x100` overflow guard (a byte at 0xfe/0xff
   stays), then +0x104 += 1 while it is not 0xff — for every kind; only the re-lock check
   is kind-0 [M :228-233, the `u16[+0] == 0` guard].
3. **The caller acts on the returned key id as the game does.** `Lock_ReLock` 0x1b0750
   calls `Player_UnEquipKey(player->BLData, lock->record u16[+2])`, which clears
   `BL[0x70C + id]` [M 0x1a97b8:6]. The per-tick pass clears
   `self.inventory.keys[key as usize]` (the port's +0x70C slots) when `update_burnt_open`
   returns the id — the key ring's only clear (the equip side stays pickup.rs's, the
   recorded gap in the lock.rs header). The game's `player->obj+0xff == 3` guard holds
   structurally: the caller is the player's own sim pass (the same documented limitation
   as `Lock_Activate`'s) [S].
4. **A decrypted lock reaches state 2.** The decompile writes lock state +0xf4 = 2 in
   exactly one place — case 1's end, when the open framelist completes [M 0x1ae3d8 case
   1] — so state 2 is "unlocked and the animation has finished", and EVERY unlocked lock
   lands there (laser burn, decrypt scan, keypad solve, the create-time Operative unlock
   all go through `Lock_Unlock` → state 1 [M 0x1b0650:28-31]). The port's decrypt-scan
   completion and the keypad solve now set `burned = true` (the framelist fold), so the
   case-2 re-lock check is live for them from the next tick. The completion also clears
   `scan_solved` — the game's `memset(&GlobalVars, 0, 0x18)` clears the digit count
   DAT_002a37e3 inside the unlock tick [M 0x1ae3d8:373], which a re-decrypt depends on.
5. **The `[S mapping]` comment for `burned` is replaced** by what the decompile shows
   (the update_burnt_open header cites case 1's single state-2 store).
6. **The state-1/state-2 channel-hold split.** `Lock_Update` case 1 writes the out
   channel EVERY tick but only in state 1 [M :192-196]; state 2 never writes. The port's
   per-tick hold loop (objective-77's warm-load fix) filtered on `unlocked || burned`,
   which held a burnt-open lock's channel high forever and made the re-lock arm
   unfireable even in principle. The filter is now `unlocked && !burned` — the
   create-time Operative unlocks keep the per-tick hold (burned=false; the 07000012 warm
   leg the loop exists for), while event unlocks (whose channel write lands on the
   completion/solve tick — the keypad solve's write moved from the next-tick hold into
   the solve branch, one tick early vs the game's case-1 write) are ownerless from the
   next tick, exactly the game's state-2 table. The disc census (below) finds no writer
   that ever drops a lock channel, so the split is observation-free on shipped data [S —
   the fold choice; the timing shift is one tick on a channel nothing reads twice].

## The census [D]

`crates/nf-view/tests/lock_census_probe.rs` dumps every 0x2E placement in every level
archive (FILES.BIN, all maps). Kind-0 locks with a non-zero out channel (param 5), all on
the main map:

| level | placements (code / out) |
|---|---|
| 0700000a | #646, #729 (code 2, out 0x31); #648, #732 (code 1, out 0x30) |
| 0700000b | #178 (code 4, out 0x2f) |
| 07000011 | #177 (code 1, out 0x12, tag 1) |
| 07000012 | #656, #694 (code 5, out 0xa, tag 1) |
| 07000014 | #774 (code 2, out 0x2c), #775 (code 3, out 0x2d), #776 (code 0, out 0x3c, tag 1) |
| 07000015 | #381 (code 2, out 0xf9), #435 (code 0x32, out 0xf7), #458 (code 0xa, out 0x95), all tag 1 |
| 07000046 | #37 (code 5, out 0x3e); sister #9 is kind 3 — the case-2 guard is `== 0`, it never re-locks |

**Who can drop the channel: nobody on the disc.** `lock_census_writers.rs` (main map,
through the port's own trigger builder) and `lock_census_allmaps.rs` (every map, every
writer placement's channel params) matched the lock channels against the AND/OR/SEQ/OUT
muxes, the OUT copies, the SS operate latches (0x25), the switches (0x29), the doors
(0xdb, ch_open p2 / ch_lock p3), the sensors (0x28, whose out is CLEARED every tick),
the sound triggers (0xf9, on/off), and the fuseboxes (0x31); `lock_census_scripts.rs`
decoded every kind-7 script stream of the level bins for event 0xD/0x11 writes. The only
hits are the pairing the locks exist for: each lock's out channel is a DOOR's ch_lock
input (0a #648/#646 → doors #724/#725; 0b #178 → #151; 11 #177 → #192; 12 #656/#694 →
#754; 14 #776 → #111/#112, and #774/#775 feed AND mux #777 → out 0x2e; 15 #458/#381/#435
→ #353/#355/#359; 46 #37 → #38). The door reads ch_lock; it never writes it. The lock's
own case-1 write is the only writer, and it only ever writes 1.

**Conclusion: the case-2 re-lock arm is unreachable in the shipped game** — after any
kind-0 lock unlocks, its channel stands forever, so `switch_channels[u16[+8]] == 0` in
state 2 never fires on shipped data. The wiring is still landed (the game runs the tick
per-tick; the port now does too), and per the task the unit pin carries the arm's proof.
Residual honestly stated: the ch-0x3c load pulse on 07000014 (the level-start scripts'
channel write that the interactables test pins) comes through the port's parsed scripts
by a path my event-0xD/0x11 sweep does not decode, so the script-side census is not
exhaustive by construction; no sweep I ran found any 0-writer for any lock channel.

## Verification

- `cargo test -p nf-game --lib`: **688 passed, 0 failed** (includes the three lock unit
  pins: the existing `burnt_open_decryptor_locks_rearm_when_the_channel_drops`, the new
  `case2_refills_the_burn_bytes_before_the_relock_check`, and the re-lock cycle
  `a_decrypted_lock_relocks_when_its_channel_drops_and_decrypts_again` — decrypt to
  completion, drop the channel, `Some(code)` back, state 0, decrypt again from zero
  digits; plus the kind-1 sister: refills, never re-locks).
- **One existing test was wrong against the decompile and is fixed:**
  `gadget::tests::decrypt_scan_solves_the_four_digits_at_the_game_thresholds_then_writes_the_channel`
  indexed `solved_at` by the POST-tick digit count, but the completion memset clears
  DAT_002a37e3 inside the unlock tick [M 0x1ae3d8:373] — post-tick the count is 0 and the
  index underflowed (255). It now reads the pre-tick count; all four solve-tick
  assertions are unchanged (162/278/354/390).
- Live binaries (NF_ISO, release): `decryptor_07000012_live` **1 passed** (the full
  Q-Decryptor chain through the real path — the state-2 fold in play);
  `interactables_07000014_live` **10 passed** (the keypad-lock burn and its Door_Slide
  pair leg — the new hold filter under the real doors); `keypad_unlock_lod_live` **1
  passed** (the keypad solve fold + the LOD swap); `gadget_laser_live` **8 passed** (the
  kind-1 laser burns, the heal, the safe bolts).
- The fast suite (`tools/test/suite.py`) was started in the first minutes per the task,
  but this sandbox kills background jobs between tool calls — `suite.out` stayed empty,
  and a foreground run does not fit the window. The touched binaries above are what ran.
  **The suite total line is not available this round.**

## What's left

- The full `walkreal_*_live` sweep and the suite total (the shared-test-helper rule is
  not triggered — nothing under `crates/nf-view/tests/common/` changed — but the suite
  run itself is owed).
- The render side is untouched (the lock meshes stay undrawn; `Control_NextLOD`'s
  model effect is carried by `lod_next` exactly as before — on re-lock the game's own
  NextLOD is a no-op at the chain end [M 0x1b0750:11, the port's documented chain], so
  the model KEEPS the unlocked variant; the keypad state needs no reset — states 3/4
  cannot coexist with state 2, and the decrypt side restarts from the memset).

## Live checks for the main agent

The arm is disc-unreachable, so these need a poke to exercise (PCSX2, fresh boot,
0700000a — kind-0 keypads #648/#646 out 0x30/0x31, doors #724/#725 on ch_lock):
1. Decrypt keypad #648 (the Q-Decryptor), then with the debugger write
   `switch_channels[0x30] = 0` (0x26fc90 + 0x30) while the lock's state word
   (`obj+0xf4`) reads 2: expect the NEXT lock tick to store `+0xf4 = 0`, clear
   `BL[0x70C + 1]` (the code index 1 key slot) to 0, and call `Control_NextLOD` (no-op at
   the chain end); the door #724 re-locks (its ch_lock reads 0).
2. Re-run the decryptor on the same lock: expect the scan to start from zero digits (the
   memset) and unlock again after the full four-digit sequence.
3. Sister: burn a kind-1 lock (07000008 safe bolt, out 0x86) and drop 0x86 the same way:
   expect NO re-lock at any state (the `u16[+0] == 0` guard).

## Usage

not visible to me (no token, context-window or cost figures are exposed to this session;
the driver's message is the only source and carries no numbers for this worker).
