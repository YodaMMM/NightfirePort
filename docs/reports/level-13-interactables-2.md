# level-13-interactables-2 — every player-touchable kind on 07000013 behaves like the game

Branch `s/level-13-interactables-2` from origin/master (e79794da; master did not move during the
task, so the branch is master + this task's commits).

## What was done

**Census [D every placement kind].** Besides the in-repo census tool
(`crates/nf-view/tests/census_07000013.rs`, opt-in), the level's placements were re-walked with an
independent parse of the same disc bytes (ISO9660 → ACTION.ELF's `FileList` @0x2446a0 → FILES.BIN →
the 07000013.bin archive directory → each map's block 0x1A, per docs/formats/placement.md and
docs/formats/level-bin.md): **439 placements across 42 maps** — both methods agree with
docs/levels/07000013.md §7 on every kind and param (doors, glass, pickups, hints, SS, SP, touches,
exit, spawners, sound posts). The kind table in §7 stands as written. (Numbering note: the §7
placement numbers follow the rust census's ordering; the independent walk used archive order, so
its global indices differ for cross-map items while every position/param matched.)

**Four new live tests** in `crates/nf-view/tests/interactables_07000013_live.rs` (6 → 10, all
green), one per kind not yet pinned on this level, each driven through the real input path with a
sister that fails on the wrong result:

1. **Doors (0xdb ×2)** — `the_security_door_walks_open_and_the_stairwell_door_refuses`: the E opens
   the security-room door #355 [M `Door_Activate` 0x135be0's toggle], the walk-up with no use key
   opens NOTHING, and the stairwell door #274 (lock ch param 3 = 0xc8, no writer on the disc [D
   census]) refuses the same E while ch 0xc8 is off [M `Door_Activate`'s locked arm]. Sisters: both
   doors rest closed; ch 0xc8 reads off at load.
2. **Pickups — the remaining takes (0xF0)** — `the_tripbombs_arm_and_the_bond_bonus_takes_with_the_
   medal`: the three Tripbombs #432-434 (weapon 0x3a, 1 round each [D census], clustered just north
   of the spawn) arm on the real walk-over [M `Pickup_Handler` 0x18e650 case 0] and are consumed
   [M :338-348]; the kind-6 BOND_BONUS pair #420/#421 (label 0x200004a [D]) is created ONLY with the
   level's gold medal — BaseMap **0x07000011** [L ee_start_05.bin @0x2a45a8: the mission/base pair
   07000011 07000011, the same record shape as 0700000b's 07000009 @0x2a4490] — and the lobby bonus
   is consumed by the walked take [M 0x18dda0:93-95 medal gate; 0x18e650:234-236]. Sister: the same
   load with the fresh-boot zero medal word creates no kind-6 at all [M 0x18dda0:93-95].
3. **Break, the filtered pane (0x20)** — `the_securityroom_glass_refuses_the_p2k_word`: the
   securityroom_glass #286 (params [0:0x2, 1:0xa, 2:0x30] [D census]) stores ~0x30 = 0xffcf [M
   `Break_Create` 0x12c230:31-39]; the held P2K's round word 0x0008 [L ee_start_05.bin weapon_data
   @0x2bf150 + 0x10·0x10C + 0x10] ANDs the 0xffcf | 0x106 mask non-zero [M `Break_Update`
   0x12bfd8:15], so `Collide_FilterBullets` pulls the contact [M 0x1ece30:24-33] and 150 trigger
   pulls leave the pane whole — though its health 10.0 would die to the first passing round. The
   sister contrast is the test above it: the unfiltered pillarglass (health 6.0) dies on the first
   round under the same fire. The 0x30-filter family pin is
   `interact_07000009_live::the_security_room_glass_refuses_the_same_gun`; this is the level's own
   pane.
4. **Hint plates (0x33 ×2)** — `the_hint_plates_carry_the_disc_params` (parse-model pin, the 0b
   music-trigger pattern): #423 label 0x10001f6 with no arm channel, #424 label 0x2000003 armed by
   **ch 0x23** — the gate panel's channel, confirming the census's read of ch 0x23 as "the security
   system released" [D census; M `Hint_Create` 0x1b3248:33-37 puts param 3 in inst+0x08, the
   on-channel; `Hint_Update` 0x1b3380:17-22 gates on it]. Level-real: exactly 2 hints.

**Correction (test + doc, with the address).** docs/levels/07000013.md §7's doors row said
"straddle param 4 = 1 → proximity open", and the pickups test's prose said the door "opens as Bond
walks up". The decompile says otherwise: `Door_Update` 0x1363c8's sphere scan (an actor within
√2·(r_door + r_actor) of the door's live world sphere [M 0x136678-0x136684]) only holds or closes an
OPEN door — both its arms sit behind a `state & 6 != 0` test — and `Door_Activate`'s caller census
(the channel arms, the open-timer arm, the player's E) leaves no closed-door proximity route. The
straddle param-4 bit is `Door_Link2Portal` 0x136788's cel-link flag, not an opener. The new door
test pins the corrected behaviour (the approach alone does not open it), the §7 row is rewritten,
and the pickups test's prose now cites the E. No decompile-backed assert was weakened — the old
prose never had an assert of its own; the door opens in the existing tests because their legs press
E.

## How it was verified

- `interactables_07000013_live`: **10 passed, 0 failed, 0 ignored** (17.3 s).
- `walkreal_07000013_live`: **1 passed, 0 failed** (8.7 s) — the full warm chain 12 → 4A → 13,
  spawn → the panel E → the gate rises → the entrance → Mission Complete → the 0x07300013 latch.
- `nf-game`: **959 passed, 0 failed, 9 ignored** across 98 binaries (`cargo test --offline
  --release -j2 -p nf-game -- --test-threads=1`).
- The fast suite (`tools/test/suite.py`) re-run on the final tree (branch == origin/master + this
  task): see the total line at the end.

## What's left / port gaps found

- **Hint firing is not in the sim** (the port's only hint code is the parse-time model,
  `nf_game::player::hints`). The game's `Hint_Update` 0x1b3380 fires a one-shot TXTMSG type 1
  (INFO pane, life 0xB4) + `Sound_Play(100, param 1)` and sets the delete bit when a class-3 object
  enters the marker's obj+0xD0 chain. The new test pins the disc params; implementing the live
  firing (occupancy + the text drain) is the remaining gap for the kind.
- Sound triggers #410/#411 (0xF9, id 0x289): presentation only; the headless sim has no audible
  path (the census's standing note).
- Spawners #323/#327: start channels 0x29/0x2a have no writer on the disc [D census] — dead data,
  nothing player-triggered to pin.
- SimpleScript #149 / ScriptPlayers: the real-path E (the SP preload panel → ch 0x23 → the SS
  sweep) is pinned by `gate13_live.rs` and walked end-to-end in `walkreal_07000013_live`; the
  interactables file's harness-poke test stays as the SS-response pin.

## Live checks for the main agent

1. Hint firing on hardware (fresh boot, 07000013): walk Bond into the marker at (−19.17, −368.60,
   26.00) (hint #423, label 0x10001f6) without pressing anything — expect the INFO-pane text for
   label 0x10001f6 within a few ticks (`Text_AddMsg(BL+0x94e, 0, 1, label, 0, 0xb4)` [M 0x1b3380:24-
   42]) and the marker gone afterwards (obj+0xFE bit 0). Repeat for #424 (−19.85, −368.63, 26.04)
   BEFORE the panel E — expect silence while ch 0x23 is off [M 0x1b3380:17-22], then the text after
   the gate panel's E raises ch 0x23.
2. The door correction on hardware (fresh boot): walk to the security-room door #355
   (−26.65, −368.36, 37.44) and stop 2 m short, no E — gdbwatch the door's obj+0xF4; expect it to
   hold CLOSED until the E (this settles the §7 prose the same way the decompile does).

## Usage

not visible to me
