# level-03-interactables-2 — every player-touchable kind on 07000003 behaves like the game

Branch `s/level-03-interactables-2` (from origin/master 6042ee53). Worker session 2026-11-01
19:31–20:30 UTC.

## What was done

1. **Fresh [D] census** of 07000003 main map 50 (`nf-extract placements-all Nightfire-onfoot.iso
   07000003.bin`): 511 placements. The kind table in `interactables_07000003_live.rs`'s header was
   re-derived from it and extended: the player-touchable kinds are doors 0xDB ×13, plates 0xEA ×19
   + Touch triggers 0xEB ×2, pickups 0xF0 ×5, breakables 0x20 ×97, destroyables 0xE1 ×5, ladder
   0x22 ×1, script players 0xD9 ×3, **sound triggers 0xF9 ×4, music triggers 0xFB ×3** (the last
   two were in the audit doc but had no live test). Present but NOT player-touchable, now marked
   in the header: 0x33 hints ×2 (display-only labels), 0x3B endleveldoor ×1 (`DynamicObject_Create`
   0x18dbf8 — no interaction arm; p2=0 no distance fade; the ch-1 delete is script-driven
   [M worldobj.rs]), and 0xF9 #490/#491 (their in channels 0x68/0x69 have no writer among the
   level's placements [D outs]). Zero on this map: switches 0x29, monitors 0x30, sensors 0x28,
   locks 0x2E, copters 0x2F, turrets 0x27, grapple 0x3A, lifts.
2. **Three new live tests** in `interactables_07000003_live.rs`, each driven through the real
   input with a sister:
   - `touch_triggers_fire_on_the_walk_and_the_lock_record_matches` (0xEB `Trigger_Touch` 0x1b2aa8,
     type 2): walking into #473's volume (out 0x85, no lock [D p0=133 p6=0]) turns ch 0x85 on and
     it latches [M `Trigger_Update` 0x1b1a78 case 2 — the `|=` write never clears]; sister: ch
     0x85 off at load. #471's record pins lock ch 0x90=144 [D p6; M the lock gate :51-57].
   - `sound_triggers_play_through_the_plate_chains` (0xF9): the four records' creation fields
     match the disc [D #490 in 0x68→333, #491 in 0x69→333, #500 in 0xAC→1515, #507 in 0x4D→1520;
     M `SoundTrigger_Create` 0x1b2628:44-53]; the spawn plate #476's ch 0x4D plays 1520
     [M `SoundTrigger_Update` 0x1b2790's play arm], and the real walk onto plate #499 (out 0xAC)
     plays 1515; sisters: the 0x68/0x69 pair stays silent (no writer [D]), ch 0xAC off before the
     walk.
   - `music_triggers_arm_through_the_plate_chain` (0xFB `MusicTrigger_Update` 0x1b31a0): the
     creation params [D #459 ch 0x21→event 33, #461 ch 0x25→34, #492 ch 0x4E→2 p3=1] and the real
     walk onto plate #415 (out 0x21) arms #459's 30-tick countdown [M]; sister: the countdown
     rests at 0. (#492's driver is the dragon key take — the existing dragon test writes ch 0x4E.)
3. **Two harness facts the tests document** (no port change needed):
   - The type-2 write latches (`channels[ch] |= hit`, [M 0x1b1a78 case 2]) — asserted after leaving
     the volume.
   - The global-voice slot: a global 0xF9 trigger (p5=1) dies when the slot is busy
     [M 0x1b2790:19-28 → obj+0xFE |= 1; the port's `tick` matches]. The game's per-frame
     `Sound_UpdateSounds` ages voices so 1520 frees the slot before the player reaches the far
     plate; the harness replays that aging explicitly (`audio.update` with a driver-stub closure,
     12 ticks > the 5-tick start grace) — the port's split of the audio aging to the app frame path
     is unchanged.
4. **A walk approach fix in the new test**: the north approach to plate #499 wedges on the
   bedroom furniture (probe: stuck at (8.38, −30.89), d 2.56); the test uses the west-inside
   approach the plates test already documents.

## What was verified (totals)

- `interactables_07000003_live`: **11 passed, 0 failed, 1 ignored** (was 8+1; the three new kind
  tests all pass).
- `walkreal_07000003_live`: **4 passed, 0 failed**.
- `cargo test --offline --release -p nf-game -- --test-threads=1` (NF_ISO set): **1028 passed,
  0 failed, 9 ignored** across 100 binaries.

## What's left

- **The pre-existing ignored test stands** (`locked_door_refuses_the_e` — the mask-4 use-pick
  admits the use sphere into plate #504's walk-in volume, so an E meant for locked door #4 fires
  the plate). Its documented fix site is the use-trigger arm's volume test
  (nf-game `player_use_triggers`, drone.rs:16023 — the port fires ALL AABB+tris hits; the game's
  `Player_Activate` 0x1a5170 pick semantics need the one-vs-all question settled against
  `Collide_Pick` 0x1eb320 first). Next session: settle the pick semantics, fix, un-ignore.
- **The fast suite did not produce a total line** in this sandbox: background jobs are killed
  between tool calls here, and the suite (build ≈ 6 min + run > 10 min) does not fit one 10-minute
  tool-call window — `suite.out` stayed empty again. The named suites above are the session's
  quoted totals. A pre-build was started to warm the target dir for the next session (it also hit
  the window cap mid-nf-view; cargo resumes from there). The next session should run suite.py once
  on this tree (the build is nearly warm) and quote the total line.

## Live checks for the main agent

1. The #473/#471 touch triggers: `gdbwatch switch_channels 0x85/0x83 write --seconds 30` on a
   fresh 07000003 boot while walking into the oDroneTrigger3 volume at (27.4, 10.3, −5.4) —
   expect ch 0x85 to rise on entry and hold (the [M] latch). If the game CLEARS it on exit, the
   port's `|=`-latch reading of 0x1b1a78 case 2 is wrong and the test must change.
2. The locked-door E (the ignored test): on a fresh boot, stand at door #4 (14.68, 6.65, −35.40)
   with ch 0x31 off and press E twice — the port fires plate #504 (ch 0x31 rises). Watch which
   objects the game's use pick touches (`gdbwatch` on `Trigger_Activate` 0x1b2318's entry and
   `Door_Activate` 0x135be0's entry): does the E reach the plate at all, or does the door win the
   pick? That settles the one-vs-all question above.

## Usage

`not visible to me` — this session's environment shows no token, cost or context-window figures.
