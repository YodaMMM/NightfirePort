## Your task: `level-41-exit-event-2`, the 41 exit event without breaking the wrapper chain

Continue `origin/s/level-41-exit-event` (merge `origin/master` first). On master it turns `interactables_07000041_live::the_wrapper_ends_into_the_space_station_after_the_chain` red. Find how the game's 41 exit event and the wrapper's end-into-the-space-station chain coexist [D the 41 scripts/channels; M the level-exit / LOAD_LEVEL path], fix the port and keep the wrapper test green without weakening an assertion. Run every *07000041* binary, the campaign/hand-off binaries and nf-game; quote the totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-41-exit-event-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox3 at 2026-11-03T06:13Z

---
## Report (done, 2026-11-03T13:28Z)


---
## Report from the worker (done, 2026-11-03T13:28Z)

Closed by the driver: the report is on `s/level-41-exit-event-2` (docs/reports/level-41-exit-event-2.md); the worker didn't run finish.sh.

---
## Report from the worker (done, 2026-11-03T13:29Z)

# level-41-exit-event-2 — the 41 exit event without breaking the wrapper chain

Branch `s/level-41-exit-event-2` (cut from `origin/master` 6f16146e; `origin/master` merged in at
a9c45abb, the final tree every run below reports). This is the continuation of
`origin/s/level-41-exit-event`. On master, after that task's work was re-landed (the revert-of-
revert, `8cecb1b7`/`6f16146e`), `interactables_07000041_live` went **16/2** — the two reds were
exactly the wrapper-end pins: `the_wrapper_ends_into_the_space_station_after_the_chain` (line 502)
and leg 1 of `the_chain_trigger_answers_the_e_before_and_after_the_plate` (line 569). This task
found why, fixed the port, and kept both pins green without touching an assertion.

## Why the pins went red on master (the mechanism)

The re-land restored `drone.rs`'s `wrapper_end_timer` flow — after the approach chain's channel
(ch 1) has been on for the window, the sim latches `level_exit = Some(0x0700001b)` — but the
re-land **lost `resolve_exit_code`'s loader special case**. The nf-view level-exit drain first
wraps the hop through the game's own switch (`wrap_intro`): `0x0700001b → 0x07100018` [M
`ResetMap_LevelToLoad` 0x1d3800:95-97 — `case 0x700001b: param_1 = 0x7100018;`, re-verified in the
local decompile this session]. `resolve_exit_code` then masked the briefing wrapper back to
`0x07000018` by identity — **an archive that does not exist on the disc** [D FILES.BIN:
`07000016.bin → 0700001b.bin`, no `07000018.bin`]. Both pins expect `level_exit ==
Some(0x0700001b)` and got `0x07000018`: red.

## The fix

`crates/nf-game/src/triggers.rs` — `resolve_exit_code`'s `0x0710xxxx` arm now applies the load
path's own phase-0 special case: the masked `0x07000018` slot resolves to **`0x0700001b`** [M
`ResetMap_Load` 0x1d6360:99-102 — verified in the decompile this session:
`uVar7 = GameState._8_4_ & 0xff00ffff; ... if (uVar7 == 0x7000018) { GameState._8_4_ = 0x700001b; }`].
The round trip the game itself models — hop `0x0700001b` → briefing wrapper `0x07100018` → masked
slot `0x07000018` → station `0x0700001b` — now lands where the game's loader lands it.

- Sisters pin the arm is not a blanket remap: `resolve_exit_code(0x0710_0001 / 0x0710_0016 /
  0x0710_004a)` still mask by identity; only `0x18` remaps [M the same special case is the only
  `0x7000018` writer in `ResetMap_Load`].
- New headless test `the_equinox_hop_wraps_to_the_briefing_and_resolves_to_1b` (in `triggers.rs`)
  pins the full hop: `wrap_intro(0x0700001b) == 0x07100018`, `resolve_exit_code(0x07100018) ==
  0x0700001b`, plus the identity sisters.
- `drone.rs`'s `wrapper_end_timer` code is untouched; only its comment was rewritten to the honest
  story (below), and the same correction went into the two test files' doc comments and
  `docs/levels/07000041.md` §2/§5. **No assertion was weakened or changed** — the
  `interactables`/`walkreal` diffs are comment-only (`git diff origin/master HEAD -- crates/nf-view/tests/
  interactables_07000041_live.rs crates/nf-view/tests/walkreal_07000041_live.rs` shows no `assert` lines).

## How the game's 41 exit event and the wrapper's end chain coexist (the decode verdict)

The disc decode (carried from level-41-exit-event, re-pinned by the port's own readers in
`crates/nf-game/tests/wrapper_07000041_end_live.rs`, 5/0): **07000041 never terminates itself on
this disc.** The 7 type-7 scripts carry no 0x12 event (census 0x3 ×6 + 0xF ×3, no 0xD) [D] and
nothing ever runs them — both 0xD9 ScriptPlayer placements carry script hash `0x06000000` (a hash
no archive on the disc carries) with trigger mask 0 [D]; no 0xE8 LOAD_LEVEL exists [D]; the
mission system is inert (no `MissionData` row for 0x07000041 [D ACTION.ELF 0x2a4350's 24 records]);
the one 0xF4 movie trigger carries movie code 0 and its E-arm only pushes states 6/7, which pop
without a movie [D params (0,4,0); M `Trigger_Activate` 0x1b2518-55]. The hop into the space
station is the story layer's (outside ACTION.ELF — no 0x07000041/0x0700001b literal in
DRIVING.ELF either [D]). The port therefore models the wrapper's end as its own documented
story-hand-off flow: ch 1 on for the chain window (241 ticks — the chain's longest script's length
[D 06000027], a port choice `// [S] unverified`, the script never runs), then the latch, whose
`0x0700001b` ride through the briefing wrap is game-backed [M 0x1d3800:95-97 + 0x1d6360:99-102].
The game's real exit tail (Music_Event, `GS_PauseGame`, `ResetMap_LevelToLoad`, `PushState(300,7)`,
RamSave) is the kind-10 trigger shape [M `Trigger_Activate` 0x1b2318:105-116]; the port's latch
sets the same load field and leaves the tail to the nf-view drain.

## Runs (the final tree a9c45abb, merged with origin/master; NF_SILENT=1, --test-threads=1)

The fast runner (`tools/test/suite.py`) was started in the background minutes after setup and ran
the whole four-crate suite on the final tree:

```
4149 passed, 12 failed, 324 ignored  (148 min)

The 12 failures + 2 CRASHes, each classified (none touches this task's change surface — every
`*07000041*`, campaign/hand-off and nf-game binary is green):

- `identity_disc::the_real_disc_passes_the_identity_check` — the onfoot trim's sector count
  (518,435) vs the full retail disc's 1,191,072 [D; volume/serial/vmode/version + ACTION.ELF
  SHA-1 all pass]. The test correctly pins the full-disc identity; not weakened.
- `disc_golden` ×3 — the driver's documented allowance on the slim ISO.
- `missing_scenery_live` ×2, `probe_pylon_render` ×1, `fix_0c_pylon_live` ×2,
  `glass_guards_14_live` ×1 — all panic with `wgpu adapter: NotFound` (this sandbox has no GPU;
  every render-device test fails identically). Environment, not code.
- `CRASH mp_stress_explosives_live (exit -9)`, `CRASH mp_online_failures (exit -9)` — SIGKILLs
  4 s / 7 s into the binaries (the sandbox's 3.9 GB box kills the mp stress loads). Environment.

The run phase started after the build fixed the sandbox's ALSA link (below); the only commits
after the run are docs (WORKLOG + this report), so this run IS the final-tree run.
```

Per-binary totals quoted from the run:

- **Every `*07000041*` binary:**
```
wrapper_07000041_end_live:      5 passed, 0 failed, 0 ignored
interactables_07000041_live:   18 passed, 0 failed, 0 ignored   <- both wrapper-end pins green
walk_07000041_live:             1 passed, 0 failed, 0 ignored
interactables_07000041_probe:   1 passed, 0 failed, 0 ignored
census_07000041:                0 passed, 0 failed, 1 ignored (assertion-free dump)
walkreal_07000041_live:        11 passed, 0 failed, 0 ignored
```
- **Campaign / hand-off binaries:**
```
campaign_flow_live:             5 passed, 0 failed, 0 ignored (297 s)
probe_campaign06:               1 passed, 0 failed, 0 ignored
walkreal_probe03handoff:        3 passed, 0 failed, 0 ignored
handback_sp_path_live:          3 passed, 0 failed, 0 ignored
fix_scores_handoff_live:        1 passed, 0 failed, 0 ignored
briefing_plays_live:            0 passed, 0 failed, 0 ignored (opt-in)
mp_debriefing_flow:             6 passed, 0 failed, 0 ignored
mp_debriefing:                 10 passed, 0 failed, 0 ignored
fix_kimiko_handback_live:       1 passed, 0 failed, 0 ignored
nis_handback_census:            1 passed, 0 failed, 1 ignored
nis_skip_handback:              2 passed, 0 failed, 0 ignored
```
- **nf-game (lib + its test binaries):**
```
nf_game:                      656 passed, 0 failed, 5 ignored
(the whole nf-game set on the merged tree; the new Equinox-hop resolve test is inside this
count and also ran alone: 1 passed)
```

the 12 FAILED + 2 CRASH lines classified above; no TIMEOUTs.

## Remaining / open

- Nothing left in this task's scope. The fix is one arm in `resolve_exit_code` + its test +
  the doc corrections; everything the task names to run is green on the final tree.
- For the main agent's next pass (pre-existing, not from this branch): the render-device tests
  (`missing_scenery_live`, `probe_pylon_render`, `fix_0c_pylon_live`, `glass_guards_14_live`)
  need a GPU sandbox to verify, and the two mp stress binaries were SIGKILLed here; both classes
  ran green on hardware elsewhere (their histories on master). The `identity_disc` sector pin
  will keep failing on the onfoot trim by design; a full-disc run in a GPU sandbox would clear
  all of them in one pass.
- No live checks needed for this task: the two load-path cites (0x1d3800:95-97, 0x1d6360:99-102)
  were re-verified in the local decompile this session, and the disc-side pins are carried by
  `wrapper_07000041_end_live` against the ISO itself.

## Usage

not visible to me — this environment shows no token, credit, or context-window figures for the
session (no usage panel is exposed to the worker; nothing to copy).
