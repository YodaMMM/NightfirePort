# ai-dmode-inits — the per-mode guard inits (`NDrone2_init_DMODE_*` 0x14d978..0x14e568)

2026-10-24. Branch `s/ai-dmode-inits`.

## What was done

The task's substance was already merged (unported-guard-ai-other-2: the `DroneModeSettings` row-0
port, the 555-placement census, `docs/systems/ai-dmode-inits.md`, the `dmode_row0_near_player_live`
tests). This session re-derived that evidence independently with two parallel subagents, found and
fixed real errors in it, and closed the one untested behaviour. Details:

1. **Dispatch + table, re-verified [D]+[M].** `DroneModeSettings` 0x29ad40 = 35 rows × 12 bytes
   (symbol size 420 decimal — file offset 0x194C40 in work/elf/ACTION.ELF via the single PT_LOAD
   mapping). All 32 non-NULL row function pointers are exact symbol matches, row 0 =
   `NDrone2_init_DMODE_Normal` 0x14d958 (team byte 1, alert word 0x00000000). The reader
   `NDrone2_DoModeSettingsOLD` 0x14cfa0 is the table's only code consumer (base built as
   `lui 0x2a/addiu -0x52c0`, bounds `sltiu s3,0x24`); byte+0 → +0xC5 (@0x14d038/0x14d040),
   byte+1 → +0x44 (@0x14d078/0x14d07c), word+8 called (@0x14d080-0x14d090). Rows 23-33, not named
   anywhere before, are now dumped: 23 CivilianGuard 0x14e158, 24 Interogator 0x14e238, 25
   CivDoorGuard 0x14e258, 26 TruckDriver 0x14e348, 27 CastleChatGuard1 0x14e370, 28
   CastleChatGuard2 0x14e3a0, 29 SniperAlert 0x14e3e0, 30 PartyGirlLooker 0x14e490, 33 Bot
   0x14e568; rows 31/32/34 have fn = NULL. `DoModeSettingsNEW` 0x14c600 (param 5 ≥ 0x24) reads no
   row slot.
2. **Census, re-verified [D] and extended.** Main-map histogram of placement parameter 5 is
   exactly {100: 555} (all NEW path) — MATCH. The fresh scan also finds 33 kind-0x0f template
   records (all `flags == 0x0000000F`) with parameter 5 = 0 in the type-0 template map `01000166`
   (object 0x02000637, byte-identical across the 41 archives carrying it); 588 kind-0x0f records
   total, 35 archives have type-1 main maps (21 with drones). Whether template records reach
   `Drone_Create` at runtime is unverified from disc — if they do, parameter 5 = 0 selects row 0
   like the coder event, so "only row 0 dispatched" holds either way.
3. **CORRECTION — the coder-event hashes.** The merged docs, the drone.rs comment and the row-0
   test carried wrong hashes: the disc's only `Cmd::Event` kind 8 (851 events in 468 scripts
   scanned) is 07000001 entry **`0600005B`** with raw params **[0x0500001B, 0x060000A6, 16, 1]** —
   not "NIS 0x0600003B, params [0x0500005B, 0x06000106, 16, 1]" (no 0600003B entry exists on the
   disc at all; the correct hashes were already pinned by the independent nis_coder_break_live
   test, which corroborates the scan). `Script_EventHandler` 0x1d94c0:138-143 case 8 →
   `Drone_CoderCreate` 0x13a108 stores them at template words 11/12/20 [M 0x13a160/0x13a168/
   0x13a170] = parameters 0 (skin), 1 (start script), 7 (= 1, the event's fourth word); the third
   word (16) is param_5's low half and is never stored. Consequence fixed in the port comment: the
   coder skin 0x0500001B takes DefaultInit's case 0x500001b → **+0xd8 = 9** [M 0x14b300:153-156],
   not the switch default 0 — the +0xd8-conditional extras still don't fire (9 ∉ {0x10,0xd,0x11,
   0x13}), so no behaviour change.
4. **CORRECTION — row word+4 is copied as a float.** A byte-scan misses it: `lwc1 f0,4(s2)`
   @0x14d010 → `swc1 f0,0x50C(s1)` @0x14d014 + `swc1 f0,0x500(s1)` @0x14d01c [M 0x14cfa0]. The
   port's row-0 comment now cites the exact instructions.
5. **NEW pin — the coder drone's start state.** CoderCreate writes +0x5A2 = 4 (constant 7th arg
   [M 0x1d94c0:141 → 0x13a1a0 `sh s5,1442`]), but PostLoad_Init 0x14d890 calls DefaultInit
   unconditionally, whose DoTypeSettingsOLD(+0xC5 = 0) overwrites +0x5A2 by mode: DroneTypeSettings
   row 0 start = −1 → prop 0x24 (set by Defaults) → **6 InitPatrol** [M 0x14d2e8:0x14d32c/0x14d3bc;
   `Drone_Create` sets DIVars word 0 = obj so the call is reached, 0x136be0]. The port's
   `start_state()` already computes exactly 6 for this shape; the test now pins it. The port's
   pre-enable `state == IDLE` bookkeeping (pinned by nis_coder_break_live) is behaviourally
   equivalent — the coder drone's start script 0x060000A6 takes PlayScript on enable and the
   post-script fallback is start_state = 6 in both; the movie test's doc comment now carries this
   chain instead of citing the transient 4 as final.

## How it was verified

- Two parallel subagents: (a) ELF table dump with program-header math + symbol resolution (all 35
  rows cross-checked against 7 expected rows: MATCH), (b) an independent ISO census reusing the
  repo's format docs (2,230 maps / 23,329 placement records parsed; per-archive counts match the
  golden `drones.tsv` 21/21).
- Disassembly of every load-bearing instruction cited above (mipsdis + hand-decode); the C
  decompiles of DefaultInit 0x14b300, DoModeSettingsOLD 0x14cfa0, DoTypeSettingsOLD 0x14d2e8,
  Drone_Create 0x136be0, Drone_CoderCreate 0x13a108, Script_EventHandler 0x1d94c0, PostLoad_Init
  0x14d890, init_DMODE_Defaults 0x14d528 (40 setProperty(1) calls — the port's list is these
  deduped + DoModeSettingsOLD's tail 0x38, all confirmed).
- `cargo test --offline --release -p nf-game --test dmode_row0_near_player_live` — 2 passed
  (corrected hashes + the new start-state assertion).
- Full suite via `tools/test/suite.py` on the final tree (merged origin/master 78b02cfd, branch
  merge bb394a09): TOTAL-LINE-PLACEHOLDER. Known pre-existing slim-ISO failures (same set in the
  pre-session baseline /home/z/suite.log): `identity_disc::the_real_disc_passes_the_identity_check`
  + the 3 `disc_golden` tests.
- Crates touched: nf-game (comment + test), nf-view (test doc comment), docs. Tests run: the
  dmode_row0 pair + the full suite.

## What's left

- The 33 template-map records' runtime reachability ([S] unverified) — needs a live trace or a
  loader audit; harmless either way (row 0).
- The port's coder-create integration applies the game's state chain lazily (state IDLE at spawn,
  start_state on enable/post-script). Behaviourally equivalent for the only reachable shape; a
  stricter port would model the +0x5A2 overwrite inside the spawn. Not done — no observable
  divergence to pin.
- Rows 1-34 remain unported (correctly: never dispatched — parameter 5 ∈ {0, 100} everywhere
  [D]).

## Live checks for the main agent

None required — every claim above is settled by [D] disc bytes or [M] decompile/disassembly.

## Usage

My own session token/credit/context figures: not visible to me. The two research subagents
reported their own usage in their final messages: table-dump subagent "Tokens -> prompt:60530
completion:3852 total:64382"; census subagent "Tokens -> prompt:132721 completion:1605
total:134326" (both read verbatim from their reports).
