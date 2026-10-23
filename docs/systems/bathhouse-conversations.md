# The bath-house conversations (07000002): the game LOOPS them

Build: ACTION.ELF `e7ffdcb3…`. Tags: [M] code (function@addr), [D] disc bytes, [L] live PCSX2, [S] suggested.
Written by `bathhouse-conversation-loop` (2026-10-22; user question: "bath house conversations should loop?
research"). Prior art: docs/systems/under-bathhouse-voices.md (776 — the combat shouts), docs/systems/map-ambient-sounds.md
(790 — the map ambients + the driver budget). The port: `crates/nf-game/src/drone.rs`
(`monitor_talk_to_mission_obj`, `talk_to_mission_obj`, `msg_now`), tests
`crates/nf-view/tests/hostage_killer_07000002_live.rs`.

## What the "bath house conversations" are

The two captor↔geisha MISSION_TALK dialogues the level places [D placements, kind 0x0F, main map]:

| pair | captor (mode 4) | geisha (mode 5) | wake channel [D param 3] |
|---|---|---|---|
| the bathhouse-floor pair | #447 (−45.6, 5.8, 10.9) | #465 (−45.5, 5.8, 12.2) | 0x17 |
| the tunnel-pond pair | #458 (−11.2, 6.4, −37.8) | #466 (−12.0, 6.6, −39.1) | 0x19 |

Their `+0xC5` DTYPE mode bytes are 4 (killer) and 5 (hostage) [M `NDrone2_GetDTYPENEW` 0x14c938 case 4;
the k16 census: mode 4 ×2 on 07000002]. The pairing is `NDrone2_SetupHostageKiller` 0x144030 (nearest
mode-5 drone within 5.0, both `+0x2B0` links set) on the HostageKiller ENTER [M 0x15c728:21-29].

## The game LOOPS them — the evidence

1. **The sequencer has no terminal step.** `NDrone2_MonitorTalkToMissionObj` 0x144340: step 0 sends msg
   0x1A to the captor's own obj [M :19-21]; step 1 waits on the captor's own `+0x4D0` (a finished line is
   cleared in place) then sends 0x1A to the hostage `+0x2B0` and steps to 2 [M :26-29]; **step 2 waits on
   the HOSTAGE's `+0x4D0` (read live through hostage obj+0xE0, clearing a finished one [M :47-56]) then
   sends 0x1A to the captor and steps back to 1 [M :58-73]** — the 1↔2 cycle is unbounded. It runs every
   tick at the top of the HostageKiller UPDATE, before `EnemyLookForOpponent` [M 0x15c728 case 3].
2. **Every turn re-rolls a fresh random line.** The 0x1A handler `NDrone2_TalkToMissionObj` 0x14a8b8:
   `+0xC5` 4 → `0x29 + Rand(6)`, 5 → `0x2F + Rand(4)` [M :8-21]; `Drone_SfxInfo` 0x29..0x2D carry sfx
   736/729/730/733/727 and 0x2F..0x32 carry sfx 110/108/109/107, **all repeat 0** [D ELF table] — so the
   per-line repeat gate never refuses them, and the exchange never runs out of material.
3. **What ends it:** only leaving the idle state — an alert (`EnemyAlerts`, msgs
   0x11/0x12/0x14/0x15/0x16/0x1e → `Drone_SM_SetState`) or the execution timer (case 10 → the 0x58 anim,
   done-msg 5 → the hostage's DIE) [M 0x15c728]. Bond's distance is not a separate gate: the monitor runs
   wherever the killer is, the lines are 3-D sounds (`SFXSetup`'s inner/outer radii — the captor's lines
   inner 5 / outer 10..60, the geisha's entries inner 0 / outer 0 [D SFXPARAMS]), and sight/alert is what
   breaks the loop.
4. **Same-side repeats are the game's own behaviour.** The captor's 6-line pool contains the silent line
   **0x2E: sfxId 0** [D Drone_SfxInfo]; `NDrone2_SpeechSFX` 0x141b00 no-ops an sfx-0 entry, and the
   monitor's step 1 then sees `+0x4D0 == 0` and advances at once [M 0x144340:10-24] — so a 1-in-6 captor
   turn says nothing and the HOSTAGE legitimately speaks twice. The hostage's pool has no silent line [D
   0x2F..0x32 sfx ≠ 0] and her repeat delays are 0 [D], so the captor can never legitimately repeat.

## What else on the level could "loop" — and does not

- **NIS script streams**: 07000002's base archive carries 6 type-7 scripts (0600006f/5d/7c4/4f/52/27),
  **all `loop 0`** [D `nf-extract nis`]; the two ScriptPlayers play them once — SP #400 (script 0x06000084,
  the loadable-pack intro, mode 1) plays at once, SP #399 (0x0600007f, pack 07900002, mode 0) waits on
  ch 0x35; both are `stay` (p4 = 1) → deleted after their single pass [M `SP_Update` 0x1dbda8 state 2 → 5;
  D placements]. The packs' scripts also carry loop 0 [D].
- **Drone start scripts**: all 25 oDrone placements carry param 1 = 0x06000000 — the no-script sentinel
  [D placements; M `NDrone2_DefaultInit` 0x14b300 `+0x554`].
- **The dedicated chat-guard loop machinery is dead code**: `NDrone2_DSTATE_CastleChatGuard1` 0x1612e0
  (re-runs `+0x554` when ch `+0x13D` = 0x19 rises, writes ch 0x1A on msg 2) and its CastleChatGuard2 init
  are unreached — **no DTYPE-0x18 placement exists on any disc level** [D census;
  docs/port-map/unreached.md].
- The combat barks (AttackTalk 0x13eac0, HeardNoise 0x149920, …) re-arm on state re-entry under per-line
  repeat delays — that layer is 776's, unchanged.

## The port: the loop was there, the timing was not

The port's `monitor_talk_to_mission_obj` already wrapped 2 → 1, but its step-1 hostage send rode
`msg_out` → `Drones::delayed_msgs`, which delivered ~2 ticks later. The game delivers a **due-now message
immediately**: `Drone_Message` 0x13a678 → `Drone_SM_SendMsgSelf` 0x175a38 (delay 0) → `Drone_SM_RouteMsg`
0x172128's dispatch-NOW branch — only future-due messages enter the pending queue [M 0x172128:62-66]. With
the lag, the monitor's step 2 read the hostage's `+0x4D0 == 0` (the line not yet started) and advanced —
**the captor spoke twice per cycle** (two captor lines per geisha line; the per-tick trace showed voice
handles 8 → 11 → 15 → 19 alternating captor/captor/geisha). That is the user-visible "conversations
don't loop properly" shape: the exchange degenerated instead of trading turns.

**Fix** (this task): a `Drone::msg_now` outbox delivered post-walk, before the speech pass, with the
target's line started in the same tick — the port's same-tick model of the dispatch-NOW branch. The
monitor's next step-2 then sees the hostage's voice live and waits: tight alternation, silent rolls only.

## Tests (`hostage_killer_07000002_live`, all ISO-gated, real path `nf_view::load` + `Sim::step_bond` + the audio half)

- `the_captor_and_hostage_conversations_loop_on_both_pairs` (new): both placed pairs run their monitor
  concurrently; each census requires ≥ 8 mission-voice periods (4 full alternations) in the window, no
  overlap [M 0x144340 busy tests], the captor opens [M :19-21], the step machine is seen cycling through
  both 1 and 2 [M :58-73], the hostage-directed sends keep recurring, and alternation holds except across
  a provably silent captor turn (his voice never appears in the gap) — the 0x2E rule above.
- `the_captor_and_hostage_alternate_mission_lines` (existing): its strict-alternation assertion pinned the
  double-fire's pattern and contradicted the decompile — **the TEST was wrong, fixed** with the same
  silent-0x2E exception [M 0x14a8b8; D 0x2E.sfx = 0; M 0x144340:10-24]; its send observation moved to the
  step machine's 1 → 2 edge (the same-tick delivery has no outbox window to catch).
- `a_captor_with_no_hostage_link_sends_nothing` (existing sister): extended — no 0x1A in `delayed_msgs`,
  `msg_now` or `msg_out`.
- `the_vt1_first_sight_line_is_silent_by_game_data` (bathhouse_voices) already pins the sfx-0 mechanism
  for the level's other silent line.
