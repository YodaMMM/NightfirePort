## Your task: `ai-volume-di28`, guards use the AI volumes' second parameter as the game does

`AIVol` create at 0x1376c0:85-87 stores the placement's param 2 at `DI+0x28`, and the two words after it at +0x2c and +0x26. docs/reports/unread-params-drones.md on origin/s/unread-params-drones found the port never reads param 2, across 203 records on 12 levels, and did not trace the reader. Trace every reader of +0x28, +0x2c and +0x26 on that record in the decompile. The values look like script hashes (the sample reads 0x06000000), so check what runs them and when: a guard entering the volume, a state, or a message. List in the report each reader and what the port does there, plus a census of the values per level [D]. If a reader changes behaviour, port it from the placement read through to the behaviour, keeping the count of Rand draws the same. Pin through a real level with a guard entering such a volume, red on the old code. If no reader exists, say so with the search you ran and close it. Also confirm whether drone param 22 is consumed by the stats tail, as that report left open. Run the ai binaries, walkreal for the levels you pin and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/ai-volume-di28`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox7 at 2026-11-07T16:13Z

---
## Report (done, 2026-11-07T18:01Z)


---
## Report from the worker (done, 2026-11-07T18:01Z)

# ai-volume-di28 — report

Branch `s/ai-volume-di28` from origin/master (10956badb). Task: trace every reader of the
AI-volume record's `DI+0x28` (placement param 2), `+0x2c` (param 3) and `+0x26` (param 4)
— stored by `Drone_AIVolume_Create` 0x1376c0:85-87 [M] — decide whether any reader changes
behaviour, port it if so, and confirm whether drone param 22 is consumed by the stats tail.

## Result 1: no reader of `DI+0x28` / `+0x2c` / `+0x26` exists — closed

The record: `Drone_AIVolume_Create` 0x1376c0 creates the object (`control_create_object`
0x1344e0), takes `DI = *(obj + 0xE0)` (the node from the LLIST pool at 0x2706C8, node size
0x120, `LList_Init` in `Drone_PreLoad_Init` 0x136888:13 [M]) and stores, from the placement
(param_4): +0x2c → `DI+0x20` (×0.01, the visibility factor), +0x30 → `DI+0x24` (the kind,
remap 0→0,1→2,2→1,4→3), **+0x34 → `DI+0x28`** (:85), **+0x38 (byte) → `DI+0x2c`** (:86),
**+0x3c (halfword) → `DI+0x26`** (:87), plus the half extents at `DI+0x10..0x1c` [M].

The search, so the closure can be audited:

1. **Decompile grep** (all 3038 ACTION.ELF files) for reads of `0xe0) + 0x28`, `0x26`,
   `0x2c`: two hits, neither on a volume record — `Drone_Delete` 0x136ce8:46-47 (class 0xD
   records whose `+0x28` holds an object pointer zeroed on delete) and `Searchlight_Update`
   0x1cb900:99,424 (the searchlight's own light-chain record, the LList at 0x2c74cc).
2. **Binary scan of every reference to the volume list** (capstone over the whole `.text`
   0x107100..0x241c24, lui+effective-address pairs): exactly 7 references to 0x2706C8 [M],
   a closed set — `LList_Init` (0x1368b4), `LList_Add` in Create (0x1378ac),
   `LList_Remove` in Delete (0x13a1e4), the two list-head reads in `Drone_ProcessOpponents`
   0x1390c0 (0x13910c, 0x1391c0), `Drone_VisibilityForPosition` 0x13a1f8 (0x13a218),
   `Drone_PointInAnyAIBox` 0x13a290 (0x13a2c4). No other code can obtain a volume node
   from the list.
3. **The six functions that can hold a volume node read whole**: the three walkers read
   only `+0x24` (kind filter), `+0x20` (the factor multiplied into visibility /
   opponent accuracy), `+0x10` (half extents, passed to `Intersect_PointOOBox` 0x1e8ec0),
   `+8` (obj) and `+4` (next). None reads `+0x28`, `+0x2c` or `+0x26`. A guard entering a
   volume never runs the word: the walkers multiply `+0x20` and compare `+0x24`, nothing
   else. No state or message handler touches the record either.
4. **Object-side reach**: the volume object's class byte 0x3E (`obj+0xFF`, stored at
   0x13778c [M]) is compared nowhere outside `Script_CreateEntity` 0x1d8968 (the script
   creator) and Create's own store — a binary scan for `lbu reg,0xff` + 0x3E compares
   across `.text` finds no other site; there is no class-0x3E update dispatch.
5. **Chain scan** (`lw x,0x28(y)` after `lw y,0xe0(z)` across `.text`, 542 candidates):
   every candidate maps to another class's record (drone DIVars, the searchlight/light
   chain, sprite/script-player records); none is reachable with a volume node.
6. **Script-created volumes** (entity type 0x3E in `Script_CreateEntity` 0x1d8968) pass
   the static level_tag template at 0x349948 [M disasm 0x1d8c58: `lui 0x35 / addiu
   -0x66b8` before `jal 0x1376c0`]. All 9 references to 0x349948 in `.text` are argument
   materializations inside that one function; nothing ever stores to it, so the BSS
   template stays zero and script-created volumes carry param 2 = 0.

So the values are stored and never run: the port's `aivolume.rs::build` (which reads
params 0/1 and the scale, never 2/3/4) already matches the game, and there is nothing to
port — no behaviour changes, so no pin is possible or required ("red on the old code"
would need a behaviour difference; there is none). DRIVING.ELF/SLUS_205.79 contain no
AI-volume code (no `AIVolume`/`AIBox` functions) and cannot reach ACTION.ELF's statics.

## Census of the values per level [D]

New assertion-free probe `crates/nf-view/tests/aivol_param2_census.rs` (`--ignored
--nocapture`), all 34 archives through `Level::parse` off the disc:

- 290 AI-volume placements (0xf5/0xf8) total. **203 records on 12 levels carry param 2 ≠ 0,
  and every single value is 0x06000000**; param 3 = 0 and param 4 = 0 on all of them.
  Per level: 07000002: 1, 07000003: 1, 07000005: 8, 07000006: 109, 07000007: 1,
  07000008: 3, 07000009: 1, 0700000a: 53, 0700000c: 4, 07000014: 7, 07000016: 14,
  0700001b: 1. (Matches the unread-params-drones census: 203 records / 12 levels.)
- 0700001b's record is the one kind-4 placement (the creator's 07000005/param-1=4 negate
  path); it carries the same 0x06000000.
- Interpretation [S] unverified: one constant shared by 203 records is a marker, not a
  per-volume script hash; since no reader exists, the value is inert either way. Recorded
  so the next reader hunt starts from the full value set.

## Result 2: drone param 22 IS consumed by the stats tail — confirmed, closed

`Drone_Create` 0x136be0 memcpy's all 33 placement param words to `DIVars+0xC34+4k`
(:59, :81 [M]), so param 22 lands at `DIVars+0xC8C`. `behaviour_util_get` 0x1c7a98 [M]
walks the `DoModeSettingsNEW` words: type = param 15, mode = 16, count word c1 = 17, the
c1 block, alert mode, count word c2, the c2 block, the tag word, and — when the tag's high
half is 0x8001 — three kinds × `(tag & 0xffff)/3` bit-packed words at the widths of the
`.data` table 0x303330. The probe reproduces that walk exactly (gate order, memcpy ranges,
count words, tag, bit-pack) over every 0x0f placement on the disc:

- **All 555 records with param 22 ≠ 0 (21 levels) are read by the walk** — each walk's
  last-read param index is 26 or 32, both ≥ 22. Param 22 is a stats-tail word.
- The port already consumes it through `DroneStatsTable::behaviour_get`
  (nf-formats/src/drone.rs), which implements the same walk. No gap.

## What I ran

- `nf-view --test aivol_param2_census` (the new probe, `--ignored --nocapture`): 1 passed.
- The full-suite fast runner could not be used as a background job in this sandbox
  (background processes die between tool calls — confirmed twice, also hit by the
  homing-copter-3 session), so the 436 test binaries of the four crates were listed once
  (`cargo test --offline --release -j4 --no-run --message-format=json`) and run in
  foreground batches with `--test-threads=1`, `NF_SILENT=1`, `NF_ISO` set, under a
  240 s per-binary cap (scripts/run_suite_chunk.py). **351 of 436 binaries ran: TOTAL
  1174 passed, 3 failed, 104 ignored.** That includes every `walkreal_*_live` (46/46),
  `ai_traces_live`, `fix_npc_doors_live`, `fix_guard_misc_live`, the door/guard tests,
  the lib tests (`nf_view`: 138 passed; `nf_audio`: passed) and the long tail.
- The 3 failures are the GPU-less sandbox, not this branch: `missing_scenery_live` (2) and
  `glass_guards_14_live` (1) panic at `wgpu request_adapter().expect("adapter")`
  (missing_scenery_live.rs:27, glass_guards_14_live.rs:28) before any scene work. The
  branch contains no `crates/*/src` change at all (git: docs + the ignored probe only),
  so these binaries see byte-identical code to origin/master.
- 2 binaries hit the 240 s per-binary cap (`net_soak_faults_live`, `campaign_flow_live`;
  `tools/test/suite.py`'s default limit is 600 s) and 85 binaries of the long tail did not
  fit the remaining window; none of them is touched by this branch's change surface.

## What's left

Nothing for this task's question: both open items from docs/reports/unread-params-drones.md
that it names are closed above (param 2 has no reader — search documented; param 22 is
consumed by the stats tail). The other left-overs of that report (path-node `raw[0]`
histogram vs the 0x2468abce sentinel, the AI-path extra area, the link-flag fold
re-verify) are untouched — different questions, not started here.

## Live checks for the main agent

1. Only if a future task revisits param 2: on 07000006 (109 volumes) put a write/read
   watch on a volume object's `DI+0x28` during play. Expected per this trace: the store
   from 0x1376c0:85 lands with 0x06000000 and no instruction ever reads it — across a
   guard entering the volume, entering any guard state, and any script traffic.

## Usage

Not visible to me: this session exposes no token, credit or context-window figures (no
usage panel is available in the sandbox or the IM gateway metadata).
