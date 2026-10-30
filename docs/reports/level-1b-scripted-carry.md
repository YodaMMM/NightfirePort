# level-1b-scripted-carry

Branch `s/level-1b-scripted-carry` (from origin/master 9deb3338). Task: decode the SP#211 chain
(0x060007fd → 0x060008f0) that supposedly carries Bond past the 1b boundary wall (plc #128
OBJECT_Boundary_1, x 4.433), port any missing piece, and un-ignore
`walkreal_0700001b_live::walk_arrival_to_drakes_defeat`.

## What the task found

**The scripted-carry premise is refuted on the disc, and the shaft box is sealed for the player
body. The walk stays `#[ignore]`d with the full evidence; the sealing is pinned by a new green
sister test.**

1. **No script on 0700001b moves the player [D].** A sweep of every kind-7 script entry in the
   archive, decoded through the port's Nis parser (the 5,263-stream-validated command decoder),
   finds exactly ONE player-touching event row: NIS 0x06000a20 stream 4, event 05
   (`Drone_EnableAll`) at t300. No event 4 (`Player_Disable`), no event 6 anywhere. Event 6 is the
   only player-matrix writer in the script system [M `Script_EventHandler` 0x1d94c0 case 6: it
   anchors `SCRIPTINFO+0x40` at the player's (`glb_players+0x90`) or the current stream entity's
   matrix (`Mat_Copy` 0x1e3348 = `memcpy(dst=arg2, src=arg1)`), and sets flag 0x100; `Script_Free`
   0x1da880 then re-enables the player AT `SCRIPTINFO+0x40` under that flag] — never armed on 1b.
   The tool landed as `crates/nf-view/examples/sweep1b_events.rs`.
2. **The SP#211 chain decodes clean — it is a sky event, not a carry [D].** 0x060007fd (len 4780):
   one sky entity (glist 0x02000823, type 31) on a 240-key track (1051,174,527)→(−187,158,52), an
   event-3 hold pair, the engine sound — the station-exterior fly-by. 0x060008f0 (len 470): eight
   sky entities (0x02000919..0x02000920 band) scattering from (144,26,21), SUBs 0x06000971
   (t384-391) and 0x0600096e (t392-399) — each creates a type-0xe1 destroy prop and pulses its
   channel via event 0x0D ([75,1]@t2/[75,0]@t4 and [74,1]@t2/[74,0]@t4) — and the fight gate
   event 0x0D [10,1] at t400. No player-position write, no attach, no teleport.
   ch 74's only reader in ACTION.ELF is `NDrone2_HandleTalking` 0x13f688 (radio chatter) [D sweep].
3. **The float body cannot filter the wall [M, re-verified].** `Player_Collision` 0x194df0's
   default arm (substate 9 falls here) writes the shape word +0x7D4 = 0x800 and never the filter
   word +0x7D8 (only substate 15 zipline writes 0x1C0, :382), so `Collide_Filter` 0x1ed888's
   `(surf & 0x40) && (flags & 0x800)` reject can never fire for the body — the boundary's 0x40
   material stays body-solid in the float.
4. **The boundary box is closed on every face [D, segment probes on the ported collision].** West
   wall x 4.433 (y −16.56..+16.74, z ±20.4); the shaft's top slab y 16.74 (normal down, x 4.5..24
   — no chimney at the wall); the pit floor y −16.56; and **end-walls at z ±20.40** (m40) spanning
   the shaft (x 4.4..28, y −16.5..+16.5) — the prior scan's "open z-ends" were an artifact of the
   segment query's one-sidedness ([M 0x10a8c8:168]; both directions must be probed). The
   end-walls' ONLY openings are deck-level windows 0.8..1.0 tall (x 7.5/8.0: open y −1.8..−1.0;
   x 10.0: −2.2..−1.2; the window's east edge sits between x 10.5 and 11.0; a second window at
   x 14). The float body is ~1.58 tall [M the default arm's capsule A = pos+up·0.55, B =
   pos+up·(+0x7CC−cc), r 0.55; standing cc 1.0347 [L ee_start_05.bin BL 0x16cf0f0,
   docs/fields/obj-tag-player.md]] — no window passes it. Additional obstacles mapped: a north
   ledge beam (z +20.40, y ≈ 2..3 — stalls a y-2.5 float at z 19.85) and two south rails (z −8.13
   y −1.6..−1.0; z −14.25 y −1.9..−0.7) — the pilot threads both under/over and still wedges at
   the end-wall inside the window band (min z −19.85 = −20.40 + 0.55).
5. **Landed code/tests** (all green): `the_end_walls_seal_the_shaft_box` (new sister: pins the
   end-walls both directions, the window bands, and the body-height impossibility); the walk's
   `#[ignore]` message rewritten to the sealed-box conclusion; the pilots (real-input closed-loop
   float steering, the same clamp the walk legs use) kept in the ignored body for the next task;
   the two probe tools as examples (`sweep1b_events`, `geo1b`).

## What was verified and how

- `walkreal_0700001b_live`: **9 passed / 0 failed / 1 ignored** (`--test-threads=1`, slim ISO).
- No production code changed on this branch (only the one test file + two examples + docs), so the
  suite exposure is that file; `nf-view` is the crate touched.
- The full-suite fast runner does not survive this sandbox (background jobs are killed between
  tool calls — re-confirmed; the foreground `chunk_suite.py` chunk exceeded its 560 s budget
  inside the build+run and was cut by `timeout` without reaching a milestone). No fresh full-suite
  total this window; the next message should run `tools/test/suite.py` (or chunked
  `chunk_suite.py` with the state file) on this tree.

## What's left

- The three live checks in the walk's ignore message, for the main agent:
  1. Fresh-boot 0700001b, float at the slit — does the game's Bond pass x 4.43 westward at all,
     and in which substate?
  2. gdbwatch the float substate's HITTEST (+0x7CC/+0x7D0.., cc at obj+0xDC+0xCC) during the
     crossing — is the float body a smaller/different shape than the walk's? (This is the one
     remaining port-gap candidate: if the game's float body is smaller, the window at
     (7.5..10, −1.8..−1.0, −20.40) becomes the route and the port's capsule constants for
     substate 9 need the fix.)
  3. Watch `switch_channels[74]` through 0x060008f0's t392-399 — expect the allies-talk line
     (NDrone2_HandleTalking), not a carry.
- If the live game does NOT cross either: the level's route past x 4.43 may not exist on foot
  (the mission's completion path via the defeat NIS — fix_finale's ch 0x0b + ch 100 — already
  ends the mission without the arena); then the walk's un-ignore needs the driver's call on what
  the walk should pin.

## Usage

not visible to me
