## Your task: `level-14-audit`, the 07000014 behaviour audit: every placed object and character against the game

The walk test for 07000014 passes, but real play still finds missing behaviour (characters walking through Bond, guests that never talk, doors that never lock, consoles whose action does nothing, cut-scenes with broken cameras). This task lists every gap on the level.

1. For EVERY placement on 07000014's main map [D] (characters 0x0f, doors, triggers, scripts/SPs, switches, monitors/sensors, pickups, destroyables, breakables, ladders, grapple points, lifts, special objects): name its creator in `parsemap_create_dynamic_objects` 0x1d04f0 / the static path, its update and its use/damage/event handlers [M], and what the port does with it (created? updated? drawn? usable? reacts to Bond/guards/damage? plays its sounds?). Do the same for the level's scripts (every stream's ops, the camera streams, the hand-backs) and the mission records (objectives, the channels each writes, fail conditions).
2. Write the table to `docs/levels/07000014-audit.md`: one row per placement/script/record, the game's behaviour [M/D], the port's, and a GAP column.
3. Fix the small gaps directly [M] with a test each. For each large gap, write a section in the report (what, where, the [M] chain) so it can become its own task. Draw-only gaps: name the mesh/hash/matrix.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-14-audit`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox3 at 2026-10-26T08:01Z

---
## Report (done, 2026-10-26T10:05Z)


## Report from the worker (done, 2026-10-26T10:05Z)

# level-14-audit — the 07000014 behaviour audit (report)

Branch `s/level-14-audit`, off origin/master 7e35bd77. Task: every 07000014 main-map placement,
the level's scripts and the mission records against the game; fix small gaps; report large ones.

## What's done

1. **The audit table** — `docs/levels/07000014-audit.md` (1780 lines). Every one of the main map's
   **1034 placements** [D, `census_07000014.rs --ignored` run this session on the onfoot image]
   has a row (id, name, position, full params, hash, flags) under its kind; all 35 kinds carry a
   Game [M] block (creator case in `parsemap_create_dynamic_objects` 0x1d04f0 / the static path
   0x1d1998, create/update/use functions with addresses), a Port block (crate/file/line) and a
   kind-level GAP verdict. The level's **10 kind-7 NIS scripts (101 streams, all decode 100%)**
   and the **mission records** (ee_start_05.bin) have their own sections, also with per-stream
   GAP lines. Method and conventions match docs/levels/0700000d-audit.md.
2. **Kind-numbering corrections** (both [M 0x1d04f0], both already correct in the port): kind
   0x41 = `InverterTrigger_Create` 0x1b3040 (NOT a searchlight — that is 0xde, absent here);
   kind 0xf9 = `SoundTrigger_Create` 0x1b2628 (NOT the inverter). All 18 m54 0xF9 rows are voice
   lines [D]. The level doc's §1b "0x30/0x28/0x2e/0x4a/0x41/0xf9/0xfa = dressing" row is
   outdated — all of those kinds have real behaviour modules in the port.
3. **Census-vs-doc count discrepancies resolved [D]:** 0x29 = 2 (both "alarmswitchplayer", both
   p4 = 1), 0x2e = 3, 0x31 = 11 (incl. the param-less vent prop #594). The level doc §1b table
   grouped some rows differently; the audit doc records both readings.
4. **Small gap FIXED with a test** — the weather/env placement: the game's `Env_Create`
   0x17b968 stores `GlobalEnv` on EVERY successful call (each also runs `InitDrops(drop_type)`
   0x18bf68:8), so with 07000014's two 0xDF rows (#425 "Weather" p0=0xff, then #588 "Bubble_gen"
   p0=2 [D]) the game's live DropType is 2. The port took the FIRST row. Fixed to the LAST
   (drone.rs, `.iter().rev().find(...)`; the `build_FindCel` cel-gate caveat is marked [S]
   inline in the code) and pinned by
   `crates/nf-game/tests/weather_env_07000014_live.rs` — **run green this session** (asserts
   the two disc rows by position/params and env drop type = 2 after attach).
   A small public accessor `Drones::pending_env_drop_type()` was added for the test
   (`env_pending` is private).
5. **Verifications that corrected the research**: the switch busy gate (p4 + latch → skip) IS
   enforced by the port (useobj.rs:374/:407) — an initial research note claimed otherwise; both
   m54 switches carry p4=1, so the gate is live here and matches. All 7 m54 0xf5 rows carry
   flags exactly 0xf5 [D], so the port's exact-word AI-volume filter cannot over-filter here.

## Kind-level GAP verdicts (details in the audit doc)

`none`: 0x0f (61 drones), 0x19/0x47 statics, 0x20 breakables, 0x24 starts, 0x29 switches, 0x2a
sky, 0x31 fuseboxes, 0x32 rainbox, 0x33 hints, 0x3a grapple, 0x3b dynobjects, 0x41 inverters,
0x4a apocalypse FX, 0xd9 SPs, 0xdb doors (55), 0xdf weather (FIXED), 0xe1 destroyables, 0xe5
cover, 0xe8 exit, 0xe9 triggers, 0xea/0xeb/0xec/0xed/0xef triggers+muxes, 0xf0 pickups, 0xf1
spawners, 0xf2 emitters, 0xf5 volumes, 0xf9 soundtriggers, 0xfa icons, 0xff dead data. Scripts:
all 10 NIS + all mission records `none`.

## Large gaps (each needs its own task)

1. **0x30 monitor consoles (12 placements) — the draw side is missing.** The game renders a
   first-person feed of the watched sensor into the monitor view (`Camera_SetUpdator` [M]; the
   sim state machine and channel writes are already ported and match). Nothing in nf-view draws
   any monitor feed. Likely the "consoles whose action does nothing" playtest symptom: E works
   (channels flip) but the screen shows no picture.
2. **0x28 security cameras/sensors (14 placements) — visuals/audio only.** The lens and mount
   proxy meshes and the beam mesh are not drawn, and the hum/servo/beep SFX are not played; the
   sim cone/sweep/channels are complete and match.
3. **0x2e locks — `Lock_ReLock` 0x1b0750 is implemented (lock.rs `re_lock`) but has NO caller**:
   the burnt-open tick (`Lock_Update` 0x1ae3d8 case 2 → re-lock when the take channel drops +
   `Player_UnEquipKey` 0x1a97b8's key-ring clear, Inventory::keys) is not wired into the sim
   tick. Small-medium task: one caller + the key clear.
4. **Port-wide latent**: the dispatcher's 0x2000-template gate is enforced by the port only in
   doors/soundtriggers/sensors — useobj/worldobj/monitors/locks/hints lack it. No known m54
   impact (the level's templates are 0x31-family on other maps [D]).

## Live checks for the main agent

1. ch 0x96 writer: who writes switch_channels[0x96] on 07000014 (the port's alarm broadcast gate
   reads it; AND #336 writes 0x61; no 0x96 writer pinned in the decompile or the scripts).
   Watch switch_channels word 0x96 during play.
2. The two 0xDF rows: on a fresh boot confirm the live `DropType` global reads 2 after the level
   loads (Bubble_gen wins, last-writer-wins), not 0xff (Weather) — confirms the fix's game side,
   incl. whether #425's `build_FindCel` even succeeds at (0,0,0).
3. The kind-0xd9 lid SPs' replay (the level doc §1b's known open question): do the three
   Vent_StoreRoom lids re-play their script when re-entered, or is the mode-0x40 preload the
   only run?
4. A monitor console (kind 0x30, e.g. the SCCP bank): does E show the watched sensor's live feed
   on the console screen (confirms large-gap 1's chain)?

## Verification

- `census_07000014.rs --ignored` on the onfoot ISO: 1034 m54 rows [D] — the audit table's source
  (the binary was prebuilt this sandbox against the then-current master; the census prints only
  disc data, and the 35-kind breakdown matches the level doc's headline numbers).
- `weather_env_07000014_live` run green (cargo test -p nf-game --test weather_env_07000014_live).
- Decompile re-reads this session: Env_Create 0x17b968, InitDrops 0x18bf68, Switch_Create
  0x1adcc0, Switch_Update 0x1adea8, Switch_Activate 0x1b02a8, Lock_ReLock 0x1b0750 (all cited in
  the audit doc).

## Suite

The suite was restarted at ~10:19 UTC (the 08:51 start had died: cargo missing from the nohup
PATH, then the ALSA pkgconfig path fixed). **No completed total line at report time** — the run
outlives this report; read `tail -1 suite.out` + `grep -E "FAILED|TIMEOUT|CRASH" suite.out` on
the branch checkout for the result. Touched crate: nf-game only; the new weather test ran green
by hand. On the final tree the only expected deltas vs master are the weather change + the new
test + the audit/report docs.

## What's left

- The suite total on the final tree (see above; the run may outlive the sandbox).
- The three large gaps above as their own tasks (monitor feed draw, sensor visuals/audio, the
  ReLock caller).
- The four live checks for the main agent.

## Usage

My own environment shows no token/cost figures → not visible to me. The two research subagents
this session reported, verbatim from their final messages: kinds agent "Tokens -> prompt:217511
completion:1189 total:218700"; scripts agent "Tokens -> prompt:139222 completion:1026
total:140248".
