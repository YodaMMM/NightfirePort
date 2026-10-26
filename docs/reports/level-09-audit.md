# level-09-audit — the 07000009 behaviour audit (2026-10-26)

Branch `s/level-09-audit` off origin/master (30598272). Task: every placement on 07000009's main
map against the game — creator in `parsemap_create_dynamic_objects` 0x1d04f0 / the static path,
update + use/damage/event handlers [M], the port's coverage; the level's scripts (streams, camera,
hand-backs) and the mission records; table to `docs/levels/07000009-audit.md`, small gaps fixed
with tests, large gaps as sections.

## What's done

1. **The audit table** — `docs/levels/07000009-audit.md`: all **23 kinds / 288 placements** of the
   main map (map 39) [D `nf-extract level-survey` + `placements-all` over the ISO], each row with
   the creator from 0x1d04f0 (the full 240-entry jump table at 0x304b20, cross-checked against the
   MIPS disasm — the decompile garbles the EE 64-bit prologue), the glist gate
   (`ResetMap_Load` 0x1d6360 passes 1 only on the final load; the caller pre-gate at 0x1d1c84 skips
   0x8000/0x2000 types), the static/fixup path (`parseentity_fixup_entity` 0x1d00c0: water-plane
   flag words 0xc023/0xc047, the 0x8017/0x8018/0x8019 zero-vector rewrites), the game behaviour,
   the port's coverage (created/updated/drawn/usable/damage/sound/channels, file:line), and a GAP
   column. The zero-census list (monitors/sensors/wires/ladders/destroyables/hurt/emitters/music
   triggers/touch/SEQ/MuxOut/grapple/copters/… — 0x30 is zero on **all 40 maps**) is stated against
   the full dispatcher.
2. **All 11 NIS in `07000009.bin` decoded** (per-stream ops [D `nf-extract nis`]): the two lift
   loop scripts (0x0600007E/0x0600007D — 21 streams each: entity casts, positional sounds, 8
   sub-script streams, Ev3 self-holds), the elevator activation FX (0x0600059B), the lasers-state
   switch anim (**0x06000667** — ON→AMBER→OFF→AMBER + S4A1/S4A0), the alarm-pull anim
   (0x0600006F), the two lift sub-scripts (symbols cycle 0x0600064E, call-button flash
   0x0600064F), the pillarglass pane set (0x0600004F), the two glass-shatter choreographies
   (0x06000052: 37 streams, 0x06000027: 24) and one unattached FX (0x060007C4). **No camera stream
   and no hand-back event exists on any 09 script** — the "broken cut-scene cameras" symptom has
   no 09 script-side cause; every op kind used (Entity/EntityEnd, Sound, Light, SubScript, Events
   3/0xC/0xD, End) is in the port's dispatch [M nis.rs:806-925].
3. **The mission record** [D ACTION.ELF MissionData @0x2a4350, table 0x2a3f78]: 3 records — the
   alarm fail-objective (0x04000016, ch 0x61, flags 0x02 = channel-ON = FAIL [M 0x1827d8 flags&2
   branch]), the elevator objective (0x04000017, ch 0x46 via AND mux #274) and a **third record
   0x04000018 (ch 0x47, enable ch 0x46)** that the level doc wrongly placed on "later levels".
   The port's `mission/c07000009.rs` already matches the disc (all 3 rows) — code right, doc stale.
4. **Doc errata fixed in `07000009.md`** (each with the census evidence): the lasers switch anim is
   0x06000667 (not 0x06000627 — no such NIS in the bin); the lift SP loop scripts are
   0x0600007E/0x0600007D (not 0x060005FE/0x060005FD); the hint text is 0x01000291 (not 0x1001051);
   OBJECT_Srceen is placement #69 (not #326); the objectives table now carries record 3; the stale
   "nothing in the port writes ch 0x7C/0x7D" note replaced (the switch E path writes them and is
   tested).
5. **One small gap fixed with a test**: no 09 test exercised the ch-0x7D side of the alarm OR mux
   #278 (both existing alarm tests press ch-0x7C pulls). New
   `interact_07000009_live::a_ch7d_alarm_pull_raises_the_alarm_too` drives the real input path —
   E pulses at lobby pull #76 (ch 125 [D placement #76]) → ch 0x7D [M Switch_Activate 0x1b02a8] →
   OR mux #278 → ch 0x61 [D #278 params] → objective 0 (flags 0x02) fails the mission
   [M 0x1827d8]. **16/16 green** in the binary.

## The remaining gaps (all documented in the audit's GAP list)

- **GAP-1 (draw-only)**: the switch press animations (alarm pulls' meshes 0x02000499→0x0200049a
  via anim 0x0600006F; the lasers art 0x020005ec/0x020005ed/0x02000a29 via 0x06000667) — the
  channel toggles but the SP-model art is a pinned draw miss. Needs the main agent's render pass.
- **GAP-2 (draw-only)**: the office_N_hologram rotor (#130) never visibly spins — the port
  integrates the angle (worldobj.rs) but the renderer draws the static placement matrix.
- **GAP-3 (large, pre-existing)**: the E route through the closed lift doors — the port's use pick
  runs from the render camera while the game builds it from `Player_GetHeadPos`
  [M 0x1a5170:101-106]. Already its own session item in `07000009.md`.
- **GAP-4 (draw-only)**: glass shatter plays the port's particle burst instead of the keyed shard
  NIS 0x06000052/0x06000027 (37/24 keyed shard-entity streams + Light + S3E/S1F6 [D]); the
  kill/sound/pierce behaviour is covered.
- No behavioural gap was found in the interactive kinds: every placement kind's game behaviour on
  09 is implemented and (where headless-testable) already pinned by an existing 09 test
  (walk/walkreal/security_alarm/interact/elevator_buttons/lift_indicators/census).

## How it was verified

- Census: `nf-extract level-survey / placements-all / nis / triggers / drone-props` over the slim
  ISO [D]; the mission record parsed from ACTION.ELF directly (no nf-extract subcommand exists).
- Dispatcher: decompile of 0x1d04f0 + hand-check of the jump table and call sites in the ELF asm
  [M]; port side greps with file:line for every kind.
- Tests: `interact_07000009_live` **16/16 green** post-change (includes the new ch-0x7D test);
  `campaign_flow_live` re-ran green in isolation (**5/5, 503.74 s**) after its suite TIMEOUT.

## Suite status (the honest ledger)

- Full suite on this tree's test content (pre-outage run, same binaries): **320 passed, 4 failed,
  25 ignored** — the 3 `disc_golden` failures are the allowed slim-ISO set; the 4th was
  `campaign_flow_live` TIMEOUT under the census build's CPU pressure (passes in isolation, above).
- The final tree adds only the new test + docs. The one binary whose content changed
  (`interact_07000009_live`) was rebuilt and re-ran **16/16**. A fresh full-suite re-run on the
  final tree was started (build ok: 811 binaries) but the 2-core sandbox forced the run cut after
  the first ~34 binaries (each foreground call is capped at ~9.5 min and background processes are
  killed between calls; env drift between shells also forced two 380 s rebuilds). Same cut as the
  mission-outros-2 entry in docs/WORKLOG.md documents. The suite evidence above stands for every
  unchanged binary.

## What's left

- GAP-1/2/4 need the main agent's render (see "Live checks"); GAP-3 remains its own session.
- A full-suite re-run to completion on the final tree when a box with more wall time is available
  (`NF_ISO=... python3 tools/test/suite.py`).

## Live checks for the main agent

1. **GAP-3 (the E route)**: fresh boot to 07000009, stand Bond inside the lift shaft before the
   doors open (−3.9, −368.3, 12.5), face the Load Point (−3.93, −368.32, 10.26) and press E —
   expected: the level loads 0700000a even with the doors closed [M Trigger_Activate 0x1b2318:76-90,
   param 1 = 0 → no key gate]. Watch `switch_channels[0x32]` stays 0.
2. **GAP-1 (switch anim)**: press E at alarm pull #253 (25.05, −375.33, 37.8) — expected: the pull
   art swaps 0x02000499 → 0x0200049a for ~10 ticks [D anim 0x0600006F] and ch 0x7C latches on.
3. **GAP-2 (rotor)**: look at the office hologram from the lobby (−22.2, −367.3, 30.9) — expected:
   the rotor mesh visibly rotates around its axis [M rotor_init 0x18f4b8].

## Usage

not visible to me — this environment shows no token, credit or context-window figures for the
session (no usage HUD is exposed to the agent; the per-subagent token totals seen in tool results
were internal and are not session billing figures, so per the instructions I don't estimate).
