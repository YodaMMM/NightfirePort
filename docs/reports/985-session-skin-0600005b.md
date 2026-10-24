# 985-session-skin-0600005b — 07000001's script skin 0x05000037 loads

Branch `s/985-session-skin-0600005b` (from origin/master 26543436).

## What was asked

Script 0x0600005b stream 7 of 07000001 asks `Anim` skin 0x05000037; the skin rides only
07000005/06 (plus the non-story 07000049/07000024 archives [D scan]). The game resolves it from the
session skin table [M G-021]; the port's merge (`nf-view/src/lib.rs` `wanted` + `session_container`)
only collected skins from drone placements (0x0f param 0) and 0xd9 ScriptPlayer placements' DIRECT
`Cmd::Anim` ops, so the skin was missing after load (`raw-missing script skins NOT merged at load:
0x05000037`).

## What the game actually does (the two halves of the rule)

1. **The session table fill** — the skin's availability has nothing to do with script starts.
   `AnimLoadFile` 0x1bdf40 registers every container's kind-3 skin first-wins and nothing frees them
   between levels [M G-021]; the load chain brings the front-end container plus every EARLIER story
   level's archive [M ResetMap_Load 0x1d6360:23-36]. The story order is 05, 06, 07, 08, then 01
   [M `mission::LEVELS`; L ee_start snapshots; the campaign rows' end levels, briefing_plays_live],
   so when 07000001 loads, the hashtable already holds 0x05000037 from 07000005/06.
2. **How 0x0600005b gets started on 01 — it doesn't.** Exhaustive disc + code scans found NO start
   path: no placement param references it (all 0x06-prefixed params on 01 belong to the 0xd9
   placements 0x06000000/16/17/86/87 [D probe]); no `Cmd::SubScript`, `Cmd::Entity` or `Cmd::Event`
   op references it in any of 01's 11 scripts [D probe; 01 has no SubScript ops at all]; it is in no
   0xC loadable record (01's only record is 0x06000087 flag 1 → 07800001.bin [D]) and in no pack;
   no code table holds it (the C_NIS_Handler 0x205f10 cell table, the `Snd2Lbl` 0x2d25d8 table, no
   immediate in ACTION.ELF/DRIVING.ELF/SLUS_205.79 [D byte scans]); DRIVING.ELF and the boot ELF
   carry no reference either. The game still LOADS the script: `LoaderProcess` 0x1d4880 case 7
   `Script_Load`s EVERY kind-7 entry of a container [M], so 0x0600005b sits in the runtime script
   table — an orphaned briefing script (the census docs already said "orphaned" [D
   docs/systems/glass-breach.md, script-census.md]). Its one `Anim` op (`Anim{script 0x06000428,
   skin 0x05000037}`) references an anim script that NO archive on the disc carries [D scan], so
   even a hypothetical start would resolve the skin and then fail the anim script.
   A stream's `Anim` op resolves its skin through the hashtable at run time
   (`Script_AnimStart` 0x1d8f40 → `AnimObjectNew` 0x1be910's `hashtable_getitem` [M, G-021]).

**The port's rule therefore follows the loader, not the start paths**: the current archive's
kind-7 script set (exactly what `LoaderProcess` case 7 loads) is the set of scripts whose `Anim`
ops can ask for a skin; the session-gated merge (G-021's `session_container`) is what makes the
skin available. No wider scan: foreign containers' own scripts are not walked, and a skin the own
archive carries never triggers a merge.

## The fix (loader only, nf-view/src/lib.rs)

The `wanted` collection's 0xd9-placement lane is replaced by the census's script walk over
`level.nis` (the current archive's kind-7 set), transitively through `Cmd::SubScript`
([M `Script_SubScriptStart` 0x1db050 `Script_Load`s + plays the sub into the same runtime]), sorted
for determinism. The drone-placement (0x0f param 0) lane and the G-021 session gate are unchanged.

- 07000001: `wanted` = [0x05000037] → the gate picks 07000005.bin (the first session container
  carrying it; 07000049/24 fail the story gate [D]) → `g-021: merged 07000005.bin for skins
  [05000037]` → the sim's skin table holds 0x05000037 after load (verified: 42 skins post-load vs
  28 raw; was missing before).
- No other level gains a `want`: the census's raw walk pins `skinless_script == 0` on all of them
  (the only raw-missing script skin on the disc is 01's 0x05000037), so no golden-bearing load
  changes.

## Where stream 7's entity would be visible

Nowhere in the live scene: nothing starts 0x0600005b ([D] above), so stream 7's `Anim` entity
(a character in skin 0x05000037) is visible at no tick and no channel of the port's 07000001 run —
and the shipped game never draws it either (the same data). If the script were ever started, its
stream-7 `Anim` op keys at tick 121 of the 225-tick scene (stream 7: `Wait(121) Anim(06000428,
05000037, 1)` [D]). No drawing code changed.

## Tests

- `missing_models_census_live.rs`: the raw-missing diagnostic is now a pin — every script skin a
  session container carries must be in `sim.skins` after load (assert per level, the evidence
  chain in the comment). The RAW-archive pin for 01 (`skinless_script == 1`) stays: the disc fact
  is unchanged; its message now says the load-time merge covers it. New sister
  `the_session_merge_adds_nothing_where_no_skin_is_missing`: on 07000005 and 07000007 (both need
  nothing: `skinless_script` 0, `skinless_drones` 0) the loaded skin table must EQUAL the raw
  archive's set — the merge adds nothing.
- Green: `missing_models_census_live` (2 passed), `models_world_live` (goldens unchanged — they
  pin RAW parses, which the load-time change cannot touch), `drone_skins_all_levels_live`,
  `walkreal_07000001_live` + `async_level_load_live` + `campaign_flow_live` (the 01-exposed load
  paths, 3 passed).

## What ran (suite situation)

`tools/test/suite.py` (fast runner): the sandbox kills background jobs between tool calls (re-confirmed:
the nohup'd suite and a nohup'd cargo both died with empty logs), so the suite ran FOREGROUND in
`--only` chunks. Chunk 0 (146 nf-view/nf-formats binaries, mp_* excluded): **75 binaries run, 0
failed** before the call's 9-minute wall; the remaining chunks did not fit the window. The MP
network/stress binaries (`mp_online_failures`, `mp_stress_explosives_live`) get SIGKILLed (exit -9,
OOM/hang on 127.0.0.1 UDP) in this sandbox — pre-existing, unrelated to this loader change [S:
both exercise the net/MP paths this task never touches]. `cargo test --no-run` for all four crates
builds clean (4m50s). Ran the crates my tests touch: nf-view (the tests above).

**Left for the next round**: the remaining suite chunks (nf_game's unit binary and the rest of
nf-view's live tests — none touch the changed lane beyond what ran green), and the 07000001 live
render if the driver wants eyes on the load (nothing new renders — see above).

## Live checks for the main agent

Nothing visual changed; no render needed. Optional log check: a fresh boot of 07000001 should print
`g-021: merged 07000005.bin for skins ["05000037"] (…)` once during load. If you want the game-side
counterpart: a fresh-boot PCSX2 run of 07000001 with a watch on the hashtable
(`hashtable_getitem` 0x1ca5b0) for hash 0x05000037 at level start — expected present (registered by
07000005's container earlier in the session) — and optionally confirm no script ever starts
0x0600005b (its SCRIPTINFO never plays).

## Remaining known gaps

None new. The orphaned 0x0600005b stays loaded-but-unplayed in the port too (`level.nis` holds it;
the sim never starts it) — matching the game. 0x06000428 (its anim script) is on no archive of the
disc [D scan] — the game could not resolve it either.

## Usage

not visible to me — this session's harness shows no token, cost or context-window figures anywhere
(no /usage command, no status line, no API response metadata is exposed to the worker).
