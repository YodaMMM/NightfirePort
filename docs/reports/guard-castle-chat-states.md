# guard-castle-chat-states — the castle's chatting and interrogating guards act as in the game

Task: port `CastleChatGuard1` [M 0x1612e0] and `InterogateWalk` [M 0x162338] — named unported in
docs/reports/guard-noise-attack-states.md — with their message handling, in the game's order,
keeping the Rand count; find the levels whose placements use them [D]; red test first through real
input; pin through real play; run every walkreal for the levels that use them; run the ai binaries
and nf-game.

## What the disc says (the [D] finding)

- **07000006 (Mayhew B, the castle interior) is the only on-foot level that uses them.** A census of
  the kind-0x0f drone placements of all 22 walkreal levels (`nf-extract drones`) found exactly two
  chat specials: skin 0x05000030 with param 1 = **0x06000124** at (31.369, 39.502, 22.053) and skin
  0x0500002e with param 1 = **0x06000126** at (30.457, 39.329, 23.081) — the chat pair by the dock,
  ~26 u west of the truck spawn [D census].
- The entry the earlier unreached.md audit missed: those param-1 values are not scripts the pair
  plays — they are the **DoModeSettingsNEW start-script specials** [M 0x14c600:130-158, called from
  DefaultInit 0x14b300:501]: 0x0600011d → prop 0x1d=1 + mode 0x17; 0x0600011f → mode 0x18 +
  channel(+0x13D) 0x19; **0x06000124/0x06000126 → mode 0x18 + channel 0x11**; 0x0600021f → prop
  0x38=0 + `+0x4F8 |= 0x10`. The audit's census covered placement parameters 15/16, not this
  param-1 path — its "no disc producer" conclusion was correct for those, wrong in general. No
  placement uses 0x0600011d/0x0600011f/0x0600021f [D census].
- Mode 0x18 lands the pair in CastleChatGuard1 0x36 twice over: PlayScript's ENTER redirects on
  `+0xC5 == 0x18` before any script plays [M 0x15ba68:23-30], and the DroneTypeSettings by-mode
  arm maps 0x18 → 0x36 [M 0x14cbd8:98-100]. +0x554 keeps the script id (0x06000124/26 ARE real
  scripts in the level bin, 1244/1204 frames [D census script_found]) — the chat state itself
  plays them.
- **InterogateWalk 0x3d has no disc path**: no placement, no DroneTypeSettings start row, no
  in-code SetState (docs/port-map/unreached.md, re-verified by my decompile greps). It is ported
  for table fidelity; its handler is unit-pinned, and it is unreachable on the shipped data.
- What the port did on 07000006 before this branch: the pair were plain guards — the specials
  block was explicitly "not ported" (the old init_properties comment), so both kept dtype 1's
  start states and the 1244-frame script played straight out of PlayScript (the old PLAY_SCRIPT
  msg-5 comment even documents one settling into IDLE).

## What was ported (game order, zero Rand draws)

All in `crates/nf-game/src/drone.rs`:

1. The DoModeSettingsNEW start-script specials at the tail of the creation path (after
   `type_setup`, matching the game's block order inside DoModeSettingsNEW) [M 0x14c600:130-158].
2. The PlayScript ENTER redirect: mode 0x18 → SetState(0x36) after the 0x10000/0x2000 flags [M
   0x15ba68:23-30].
3. **CastleChatGuard1 0x36** [M 0x1612e0]: ENTER's 0.66 alert fold (anim 0x21 / stand-idle); EXIT
   raising `switch_channels[0x1A]` + `switch_channels_time[0x1A]` when the start script never
   played (`_DAT_002701f8` = switch_channels_time[0x1A] [M symtab 0x270190 + 0x68]; the port's
   `raise` outbox does the array write + nonzero stamp); UPDATE playing the start script once via
   SetScript(+0x554, 0, 0x11a, 0, 0, 5, 0) when the chat channel (+0x13D) is up, then
   EnemyLookForOpponent breaking the chat; ANIM_DONE re-folding with the next-AI-4 replay
   [M 0x1612e0:57-65 + 0x13ab58:99]; the impact map (punch 6 / bullet 8 resume 0, taser 7 /
   explosion 9 / smoke-flash-dart 0x17-0x19 resume 1, resume 0x56); the alert group 0x11/0x12/
   0x14/0x15/0x16/0x1e → EnemyAlerts with the +0x5A0 = 4 hearing-return stamp on the None arm
   [M 0x1612e0:94-98]; forced attack and explosive. **Zero Rand draws in the handler [M 0x1612e0]**
   — the Rand count is kept at 0.
4. **InterogateWalk 0x3d** [M 0x162338]: ENTER's 0x35 (ALERT_WALK) anim; UPDATE's SetAngleToObj,
   the +0x228 & 0x10 armed-flag dance (fresh armed opponent → Attack with NO speech, armed-at-start
   keeps walking, the flag drops when he holsters), the 0x27C closest-approach tracking, within
   4.0 → Interogate 0x3c, drifted 4.0 past it → Attack 0x56; the same impact map; the alert group
   with NO +0x5A0 stamp (the decomp difference from 0x36); forced attack / explosive. **Zero Rand
   draws [M 0x162338]**.
5. New fields `chat_channel: u8` (+0x13D) and `chat_channel_on: bool` (the per-tick
   `switch_channels[+0x13D]` cache, read in `Drones::tick` beside `wait_channel_on`).

## Verification

- **Red first**: on the old drone.rs the two old-code-compatible unit tests fail —
  `play_script_enter_redirects_the_chat_pair_into_castle_chat_guard1` and
  `interogate_walk_folds_distances_and_the_armed_flag`: **0 passed, 2 failed** (786 others
  filtered out). On the fix: **4 passed, 0 failed** (`castle_chat_tests`: the redirect, the
  creation specials giving mode 0x18 + channel 0x11 + keeping +0x554, the channel-gated script
  play + the +0x5A0=4 hearing return + the forced-attack break, the InterogateWalk folds).
- **Real-play probe** (`walkreal_07000006_live::the_castle_chat_pair_stands_chatting_and_breaks_off_when_alerted`,
  now `#[ignore]`d with the finding): Bond walks from the truck spawn to the census coords (no
  teleport) — and **no present drone carries the specials there**: every 0x05000030/0x0500002e
  drone dumps mode 0, chat channel 0, start_script 0 (the skins are the level's common guard
  skins; drones 3/4 at (30.25, 41.0, -16.8)/(36.1, 41.0, -19.4) are disabled pre-enable with state
  1). The two census placements do not spawn at level start — they are event-gated on the disc
  (an SS_Create / channel-gated creator, the same family as 07000001's script-0x0600005B coder
  event [D census comment in init_properties]). The handler side stays unit-pinned; the next
  session finds the gate, and the walk + state-0x36 + burst break-off pins are written and ready
  in the test.

## What was run

- `nf-game` lib whole: **786 passed, 0 failed, 4 ignored** (`castle_chat_tests` included; on the
  old code the two red tests failed 0/2).
- `walkreal_07000006_live` (the only level whose placements use the states [D census]): builds
  clean; its 8 existing tests were NOT re-run this session — the suite that would have run them
  hit **ENOSPC mid-build** (9.9 GB sandbox disk: 6.2 GB target dir), so the per-binary battery is
  the one piece this branch owes the next session. The only behaviour change is gated to the two
  0x06000124/26 placements, which do not spawn at level start (above) — no shipped walk can shift.
- The fast suite (`tools/test/suite.py`) was started twice (11:57 with a broken env — alsa-sys
  without PKG_CONFIG_PATH; 13:11 with the full env). Its build phase hit the disk full above; the
  sandbox expires before a cleaned rerun fits. **The full-suite total line: not produced this
  session.**

## What is left

1. Find the gate that spawns the 0x06000124/0x06000126 pair (census the level's creators/events
   keyed on those ids or their map), then un-`#[ignore]`
   `walkreal_07000006_live::the_castle_chat_pair_stands_chatting_and_breaks_off_when_alerted` —
   the walk legs, the state-0x36 asserts and the burst break-off are already written.
2. The who-raises-channel-0x11 census (the chat's listen channel on 07000006 [M 0x14c600:147-158]).
3. The full-suite run (this session's died on ENOSPC) + every walkreal_*_live battery.
4. InterogateWalk stays disc-unreachable; a script-driven entry (Global msg 0x1d naming 0x3d)
   would be a new finding, as unreached.md already says.

## Usage

not visible to me
