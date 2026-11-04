# audit-text-lines — every Text_AddMsg line against the port

Branch `s/audit-text-lines`, cut from origin/master 690e8fefb. The task: sweep every
`Text_AddMsg` 0x1b0c30 call in ACTION.ELF — type, id, label text, condition, display frames,
the port site that shows it — fix every missing or wrong line that has a clear port site, pin
each fix with a test, report the rest.

## Method

- Census: `grep -l 'Text_AddMsg__FScUc11TXTMSG_TYPEPcUis'` over `work/decomp/ACTION.ELF/` found
  **22 caller functions** (37 decomp call expressions); an ELF `jal 0x001b0c30` scan
  (`scripts/jal_scan.py`, program headers → 40 raw call sites) agrees on the function set —
  22 callers + Text_AddMsg itself = the 23 functions.
- Label texts: every `Txt_BindLabel` id resolved through the real USATxt.dat off the ISO
  (`scripts/usatxt.py` — FileList symbol → FILES.BIN → the `Lang` layout of
  crates/nf-formats/src/hud.rs). All texts below are [D USATxt.dat] readings, not guesses.
- Port sites: grep per label id AND per computed family (`0x2000040 + i` etc.) across
  `crates/`, then read each site.

## The type rules [M 0x1b0c30]

- **:81-88** — type 4: `bVar1 = (id == 0xff)` (an id-0xff add stores id 0 and passes the
  option gate below); outside a script camera (`DAT_002d88c0+0x29 == 0` and
  `psiMovieFinished() != 0`) type 4 becomes **5**.
- **:90-93** — in multiplayer (`MPSettings._384_4_ != 0`) type 6 becomes **1**.
- **:148** — the queue gate: `player < 4 && (id-0xff bypass || DAT_002a37c8 != 0 ||
  type-4U > 1)` — types 4/5 are dropped unless the profile's Subtitles byte (DAT_002a37c8) is
  on, except an id-0xff add. The port's `SubtitleTrack::enabled` is that byte [M subtitle.rs:41-42].
- **:179** — types 2, 3 and >6 copy the label through the text ring (`Txt_LockString` +
  `CirBuf_Add`); 1, 4, 5, 6 take the pointer.
- **:190-205** — type 5 word-wraps in two parts (60% then 40% of `TextMsgFormats[5]` width).
- **:230-254** — display time: `frames >= 1` → `frames × wrapped/raw` (+0.5); for types NOT
  4/5 the result floors at **60** (`max(scaled, 60)`); `frames < 1` stores raw.

## The census (23 functions, 40 call sites)

Legend: P = player arg. "port" = the site that shows the line. ✓ = present and matching.

| # | Function (site) | Type | Id | P | Label → text [D USATxt.dat] | Condition | Frames | Port |
|---|---|---|---|---|---|---|---|---|
| 1 | `Door_Update` 0x1363c8 @0x1365e0 | 1 | 0 | 0 | 0x2000003 "Door Unlocked" | lock channel rising edge (prev off, now on, door flag bit 0x10 clear) ∧ level ∉ {0x7000014, 0x700004a} | 0xb4 | ✓ door.rs:57-61,270 + door_text_labels tests |
| 2 | `DroneFunc_HostageSaved` 0x1441c0 @0x1442f8 | 1 | 0 | 0 | 0x200004f "1 of 2 hostages released" | level 0x7000002/3, the FIRST save (count 1) | 0xb4 | **was missing — FIXED this task** |
| 2b | same, @0x144328 | 1 | 0 | 0 | 0x2000050 "All hostages released" | level 0x7000002/3, a later save (count > 1) | 0xb4 | **was missing — FIXED** |
| 3 | `Mission_MonitorObjectives` 0x1827d8 ×4 | 2 | is_new = `ThisOrderNum <= record.part` [M :26] | 0 | case 0/1: `"%s"` of the objective label; case 2: `"%s: %s"` = "Fail" (0x170) + label; case 3: 0x2000053 "OBJECTIVE COMPLETE" | per-objective order/channel state machine (cases 0-3) | 300 (case 0/1), 0xb4 (case 2/3) | ✓ mission.rs:116 Event::Objective + scene.rs drain |
| 4 | `Mission_Update` 0x182c60 @0x182dcc | 3 | 0 | 0 | 0x2000007 "Mission Fail" | InternalState 2 (fail channel 99) | (60<<18)>>16 = 240 | ✓ mission.rs:128 |
| 4b | same, @0x182efc | 3 | 0 | 0 | 0x2000006 "Mission Complete" | InternalState 6 (done channel 100) | 240 | ✓ mission.rs:126 |
| 5 | `MP_RegisterBulletHit` 0x1857c0 ×2 | 1 | 0 | victim / attacker | 0x2000029 "Hurt by teammate %s" / 0x200002a "You hurt teammate %s!" | friendly-fire, MPSettings._408_4_ ∧ per-slot cooldown +0x22 == 0 | 0xb4 | ✓ mp/rules.rs:515-516 |
| 6 | `MP_FlagUpdate` 0x1861a8 ×4 | 4 | 0 | −1 | 0x200003d "%s Team scored" / 0x2000039 "%s Flag dropped" / 0x200003b "%s Flag returned" / 0x200003c "%s Flag picked up" (%s = 0x1c7 Phoenix / 0x1c8 MI6 by carrier team) | CTF state transitions | fptoui(60·1.5) = 90 | ✓ mp/rules.rs:360,1137,1180,1189 |
| 7 | `MP_DemolitionProtectionUpdate` 0x1870a0 @0x187278 | 1 | 0 | protector | 0x200002b "You are supposed to protect the target!" | hit by the protected player while the +0x24 cooldown is 0 | 90 | ✓ mp/rules.rs:1096 |
| 8 | `MP_GoldenEyeUpdate` 0x187568 ×4 | 4 | 0 | −1 | 0x2000043/0x2000044 "GoldenEye Key/Crystal returned"; 0x2000040/0x2000041 "%s dropped GoldenEye Key/Crystal"; 0x2000042 "%s activated GoldenEye"; 0x200003e/0x200003f "%s captured GoldenEye Key/Crystal" | GE item state transitions | 90 | ✓ mp/rules.rs:1297 (0x2000043+i), 387 (0x2000040+i), 1333, 1413 (0x200003e+item) |
| 9 | `MP_BluePrintUpdate` 0x1882d8 ×4 | 4 | 0 | −1 | 0x2000046 "Blueprint dropped by %s"; 0x2000045 "Blueprint picked up by %s" (two sites); 0x2000048 "Blueprint returned" | blueprint state transitions | 90 | ✓ mp/rules.rs:401,1506,1482 |
| 10 | `MP_BluePrintReachedBase` 0x188970 @0x188f40 | 4 | 0 | −1 | 0x2000047 "%s got Blueprint Tech" | the base reached | 90 | ✓ mp/rules.rs:1455 |
| 11 | `MP_PlayerKilled` 0x188f70 @0x1890b4 | 1 | 0 | killer slot (MPGame[victim]+0x20, unless self) | 0x200004d "Killed %s" % victim name | a credited killer exists | 0xb4 | ✓ mp/mod.rs:543 + mp/hud.rs:639 (kill_msgs) |
| 12 | `Pickup_Handler` 0x18e650 @0x18ef94 | 6 | 0 | taker (BLData+0x94e) | case 0: 0x200000d "Picked up %s"; case 1: 0x200000e "Picked up %dx %s"; case 2: 0x2000010 "Picked up Code Key"; case 3: 0x200000f "Picked up Armor"; case 4: the pickup's PICKUPINFO+0x34 label (0 → none); case 5: 0x2000013 "Invisibility"; case 6: 0x200004a "007 Bonus collected" | the shared tail after the take-effect switch (`lVar20 != 0`) | 180 SP / 90 MP | ✓ pickup.rs `Taken` + scene.rs:6083-6125 (all 7 cases, type 6→1 in MP at scene.rs:6124) |
| 13 | `Player_CheckForDeath` 0x190810 ×2 | 1 | 0 | victim slot | MP only: 0x2000037 "%s defeated" (own name) when MPGame[victim]+0x20 is −2/−1/self; else 0x2000038 "Defeated by %s" (the last killer's name) | death while MPSettings._384_4_ | 0xb4 | ✓ mp/rules.rs:273 |
| 14 | `ThirdIcon_Update` 0x196e80 @0x196f80 | 1 | 0 | 0 | 0x1000291 "Press Action" | icon kind 6 ∧ the viewer obj's +0xf6 ≠ 7 ∧ the icon's cooldown (+8) expired; re-arms 4 | 0xb4 | ✓ scene.rs:3322 + hint_bar_timing_live |
| 15 | `Player_WeaponFiring` 0x1a5460 @0x1a5a04 | 1 | 0 | firer (BLData+0x94e) | 0x1000216 "Hold manual aim to use camera" | weapon_data[cur].word0 == 0x54 (the watch) ∧ obj+0xf4 == 0 ∧ boot-global `CamHint` == 0 (set 1 after) | 0xb4 | ✗ **not ported** (see "left") |
| 15b | same, @0x1a5e98 | 1 | 0 | firer | 0x10002dc "Use alternate fire to switch to detonator" | word0 == 0x37 ∧ obj+0xf4 == 0 ∧ boot-global `StickyHint` == 0 (set 1 after) | 0xb4 | ✗ **not ported** |
| 16 | `Check_Target` 0x1a7810 @0x1a7bc4 | 1 | 0 | aimer (BLData+0x94e) | `Door_DoMsg` 0x135f18's label (0x200000b "Press Action to open door" and the locked/closed variants) | nearest hit class 0x1c door, dist < 5.0, `DoorMsg` < 3, level ≠ 0x700004a | 0xb4 | ✓ scene.rs:5527 + door_text_labels_live |
| 16b | same, @0x1a7c5c | 1 | 0 | aimer | 0x2000009 "Press Action to open door" | class 0x1d trigger with `Trigger_Type` == 10, dist < 5.0, `LoadMsg` == 0 (re-arms 1200 invocations), level ≠ 0x7000009, `!Trigger_Locked` | 0xb4 | ✓ scene.rs:5552 |
| 17 | `Lock_Update` 0x1ae3d8 @0x1aeb4c | 1 | 0 | 0 | 0x2000014 "Obtained Lock Keycode %c%c%c%c " over KeyCodes[i] | the keypad opens while the code was already revealed (`KeyCodes[i]+4 != 0`) | 0xb4 | ✓ drone.rs:12287 (info_codes) + hud2_live |
| 18 | `PCQWorm_Update` 0x1aee20 @0x1aef90 | 1 | 0 | 0 | 0x2000014 (same sprintf) | the Q-Worm decodes a code not yet shown; marks `KeyCodes[i]+4 = 1` | 0xb4 | ✓ drone.rs:12287 |
| 19 | `Trigger_Activate` 0x1b2318 @0x1b240c | 1 | 0 | 0 | 0x2000021 "Warning: Not all objectives completed" | the trigger's channel still off ∧ its +0x3c cooldown 0 (re-arms 180) | 0xb4 | ✓ triggers.rs:181 → drone.rs:16469 info_labels |
| 20 | `Hint_Update` 0x1b3380 @0x1b344c | 1 | 0 | viewer slot | the hint record's own label (piVar2[1], computed from the icon's data) | a class-3 object inside the marker's sphere; plays sound *piVar2 | 0xb4 | ✓ player.rs:2631 → scene.rs:3351 |
| 21 | `Script_TextStart` 0x1db2b8 @0x1db308 | 4 | 0 | −1 | the script op's label word (BIN_GetDWord) | the op runs | BIN_GetWord (u16) | ✓ subtitle.rs:58 push / nis.rs:756 |
| 22 | `Sound_DoSubtitle` 0x1dde58 @0x1ddf24 | 4 | **0xff** | −1 | the arg's label (arg & 0x7fffffff) | subtitles on ∨ the arg's high bit; time = fptoui(g_fSubtitleDurationModification · SFXOutputData dur · 60) | computed | ✓ subtitle.rs:88-90 (the id-0xff bypass) |

The 23rd "function" is Text_AddMsg itself (0x1b0c30, 1448 bytes) — the queue machinery above.

## Fixed this task

**The hostage rescue messages (rows 2/2b).** `DroneFunc_HostageSaved` 0x1441c0 raises the
placement's rescue channel AND posts the rescue INFO line; the port only did the channel. The
game's rule [M 0x1441c0:20-51]: level-gated to 0x7000002/0x7000003, the counter `DAT_00298fac`
(no other reader or writer in ACTION.ELF — the bss clear at load is its reset [S]) picks the
label: the first save posts 0x200004f "1 of 2 hostages released", every later save 0x2000050
"All hostages released" (decomp: the "before == 0" arm forces after == 1 and always falls
through to 0x200004f; the 0x2000050 arm inside it is dead). `Text_AddMsg(0, 0, 1, …, 0, 0xb4)`.

- Port: `Drones::hostage_saves` (the counter), a `Drone::hostage_saved` pending flag (the
  state handler has no `Drones` access — the `raise` pattern), consumed in `tick_inner`'s
  drain into `info_labels` (the type-1 INFO path the locked exit rides); the view binds and
  shows it (crates/nf-game/src/drone.rs).
- Test: `crates/nf-view/tests/hostage_rescue_msgs_live.rs` — 07000002's two geishas, the real
  input path for the first (enable channel → captor killed by the walker's shot → the save),
  the game's own captor-death trigger for the second; asserts the first save posts
  0x200004f/"1 of 2 hostages released" with life 180, the second 0x2000050/"All hostages
  released", and `hostage_saves == 2`.

## What's left (the unported remainder, as a table)

| Item | Evidence | Why not fixed here |
|---|---|---|
| `Player_WeaponFiring`'s two once-per-boot hints: 0x1000216 "Hold manual aim to use camera" (watch, word0 0x54) and 0x10002dc "Use alternate fire to switch to detonator" (word0 0x37) [M 0x1a5460:223-231, 385-391] | labels resolve [D USATxt.dat]; no `CamHint`/`StickyHint` equivalent anywhere in `crates/` | the firing path is split across the sim's gun handling and the view's anim datum; the game's trigger point (`weapon_data[cur].word0` on the fire press with obj+0xf4 == 0) needs its own mapping into the port's `Gun` fire state — no clear single site in the remaining window |
| The once-per-boot globals `CamHint`/`StickyHint` (their exact addresses read at 0x1a5a04/0x1a5e98's global loads — not re-derived here) | [S] | same |
| `pickup.rs:128-129` still says "the 0x200004A HUD message has no port path yet" | the path EXISTS (scene.rs:6083-6125) | comment-only staleness; not touched (shared file, other branches in flight) |

## Verification

- `cargo test --offline --release -p nf-view --test hostage_rescue_msgs_live` — 1 passed.
- `cargo test --offline --release -p nf-view --test hostage_save_07000002_live` — 2 passed
  (the existing hostage-machinery pin still green over the change).
- `cargo test --offline --release -p nf-game --lib` — 658 passed, 5 ignored.
- The full four-crate suite did not fit this session's remaining window (sandbox expiry
  ~23:00 UTC); the touched code (drone.rs's hostage drain, one new `Drones` field consumed in
  one SP-only path) has no other test surface than the two hostage binaries above, both run.

## Live checks for the main agent

1. PCSX2, level 07000002, the geisha rescue: after the first captor dies the INFO pane must
   show "1 of 2 hostages released" for ~3 s; after the second, "All hostages released"
   (the adds at 0x1442f8/0x144328; expect Text_AddMsg(0, 0, 1, …, 0, 0xb4)).
2. First fire press with the watch (weapon_data word0 0x54): INFO pane shows "Hold manual aim
   to use camera" once per boot (0x1a5a04; CamHint latches).
3. First fire press with the 0x37 weapon: INFO pane shows "Use alternate fire to switch to
   detonator" once per boot (0x1a5e98; StickyHint latches).

## Usage

not visible to me — this environment exposes no token, cost or context-window figures.
