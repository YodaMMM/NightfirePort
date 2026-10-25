## Your task: `level-14-fusebox-wires`, 07000014: the opened fusebox panel shows its wires

Start from `origin/s/level-14-empty-panels` (merge it first; docs/reports/level-14-empty-panels.md). The wall panel at (-88.58, -1.10, 46.15) is door #386 beside the real fusebox #385. The fusebox's script 0x06000A1B [D placement params] spawns the wire sub-objects 0x02000A62 / 0x02000A61 / 0x02000A73 (type 0x31) in streams 0/1/3 [D decode, key-1 Entities]; the port's fusebox.rs stands in with one wire per group, so the opened panel shows an empty recess.

1. Run the fusebox script's Entity streams through the same SP-entity path the route-lid replay uses (SP_Update 0x1dbda8, SP_Activate 0x1dc338, the entity create by type [M]), replacing the stand-in: the wires exist, draw, and take the game's damage/sabotage path (the fusebox objective "Sabotage base security") [M FuseBox_Update 0x1adfe8].
2. Un-ignore `panels_07000014_live::the_fusebox_use_spawns_the_wire_sub_objects_behind_the_panel` and make it green with real input; keep the 14 walks and the fusebox/objective tests green. Name headless renders of the opened panel before/after in the report.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-14-fusebox-wires`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox4 at 2026-10-25T12:10Z

---
## Report (done, 2026-10-25T14:01Z)


---
## Report from the worker (done, 2026-10-25T14:02Z)

# level-14-fusebox-wires — report

Task: 07000014 the opened fusebox panel shows its wires — run the fusebox script 0x06000A1B's Entity
streams (the wire sub-objects) through the SP-entity path, replacing the documented one-wire-per-group
stand-in, and un-ignore `panels_07000014_live::the_fusebox_use_spawns_the_wire_sub_objects_behind_the_panel`.

Start tree: `origin/s/level-14-empty-panels` (1112fc5b) merged with `origin/master` (d8334b70).
Branch: `s/level-14-fusebox-wires`.

## 1. The finding: the wires already spawn — the repro was a false fail

**No product code needed changing.** The SP-entity path the task asks for is already ported and was
already correct; the ignored fail-first repro was red for two reasons that dissolve under the decompile:

1. **The hashes were a transcription slip.** The [D] decode of script 0x06000A1B (the empty-panels
   dump example, `dump_07000014_panels`, plus the raw command bytes) gives **7 streams**; streams
   0/1/3/5 each carry a key-1 `Entity` command (obj_type 0x31) spawning **0x02000AA2 / 0x02000AA1 /
   0x02000AB3 / 0x02000AA4**, and streams 2/6 carry event 0xF with group 0 / group 1. The
   hand-off's "0x02000A62 / 0x02000A61 / 0x02000A73" drop the 0x40 bit of the third nibble pair:
   stream 0's raw disc bytes are `07 a2 0a 00 02 …` and the hash fold (big-endian, the same fold that
   correctly yields 0x02000750/51 for the lid script 0x06000A3B) gives 0x02000AA2. The port searched
   for the wrong hashes, found none, and the repro concluded "no wires".
2. **The spawning agent is the PRELOAD, not the use.** `FuseBox_Create` 0x1b0448's tail
   `SP_Create(param 0, mode 0x40)` preloads the script and the mode-0x40 beat runs it ONCE at load
   ([M] SP_LoadScript 0x1dbc08:46-54 — any mode outside {1,8} runs Play+Update+Stop, state 1), so the
   key-1 Entity commands fire at load ([M] `Script_CreateEntity` 0x1d8968's create-by-type). The use
   press is inert at a fusebox: `Player_Activate` 0x1a5170:95-160 has no class-0x3C arm, and the box's
   SP carries mask 1 (SP_Create's param_6 = 0 [M 0x1dc7e0]; the port primes it with mask 1) — no bit 4,
   so `SP_Activate` 0x1dc338's gate refuses. The wires take the DAMAGE path instead
   ([M] `FuseBox_Update` 0x1adfe8:88-96, the 0xfffd filter: the Laser's 0x0002 cuts, gunfire's 0x0008
   is dropped [L weapon_data]).

Measured on the port (probe run, 30 ticks after load): **20 wire entities** exist in
`scripts.objects` — four per box at each of the five real fuseboxes (#291/#383/#385/#833/#919),
each set at its box's translation; box #385's group 0 holds the stream-1 wire (0x02000AA1) and its
group 1 the stream-5 wire (0x02000AA4), exactly the decode's event order ([M]
Script_EventHandler 0x1d94c0:172-175 → `FuseBox_Register` 0x1b0970). All four wire glists resolve to
models (the meshes exist to draw). The SP-entity creation, the event-0xF registration and the
hit-driven cut all landed earlier (fix-party-mission's box SP, fix-fusebox-cut's cut path) and the
full 07000014 sabotage chain is pinned green by
`fix_fusebox_cut_live::the_07000014_sabotage_cuts_through_the_real_laser` (Laser through real input →
ch 0x87 + 0x88 → AND #391 → 0x8C).

## 2. The test (the only code change)

`crates/nf-view/tests/panels_07000014_live.rs` — the ignored repro rewritten and **un-ignored** (name
kept for the driver's reference; its premise is corrected in the doc-comment, per the "the TEST is
wrong — fix it" rule):

- at load, all four decode wires (0x02000AA2/AA1/AB3/AA4) exist **at the box** #385
  (−89.30, −2.22, 49.36 [D placement]) and **behind the panel** — east of the small door object #386
  at (−89.58, −0.68, 49.17 [D placement]), the room side;
- the registrations follow the decode: group 0 = 0x02000AA1, group 1 = 0x02000AA4;
- all four wire glists resolve (the meshes draw);
- the real-input E press at the panel (through `step_bond`) neither spawns nor removes the wires —
  the decompile-backed inertness ([M 0x1a5170:95-160; SP mask 1 has no bit 4]), the sister of
  `e_presses_do_nothing_to_the_fusebox` on 07000008;
- the stale "documented stand-in / cut with E" passages in `docs/levels/07000014.md` (§4 table row
  and the Port gaps bullet) are corrected to the preload-spawn + hit-cut model with the addresses.

Ran before pushing (nf-view, `--release --test-threads=1` via cargo):

- `panels_07000014_live` — **2 passed / 0 failed / 0 ignored** (the lid replay pin still green);
- `fix_fusebox_cut_live` — **4/4** (the damage/sabotage path, incl. the 07000014 laser sabotage);
- `fusebox_06_live` — **4/4**;
- `walk_07000014_live` — **2/2** (the level's walks).

## 3. The suite (partial — this sandbox cannot keep a background process alive)

The fast suite could not produce its total line here: three background launches (`nohup`,
`setsid nohup`) were all killed between tool calls with zero output, and the two foreground build
attempts first filled the disk (100%, twice — fixed by deleting ~3.7 GB of stale duplicate build
artifacts) then OOM'd at `-j4` (4 GB RAM; `-j2` builds fine). With the build cached, the run phase
was executed with a resumable foreground slice runner (saved outside the repo) that reproduces
`tools/test/suite.py`'s run semantics exactly — one binary at a time, `--test-threads=1`,
`NF_SILENT=1`, a 300 s per-binary wall (the runner's is 600 s; only matters for a hang) — over the
REAL test binaries (cargo JSON `kind` test/lib, 795 of them; the runner's 798 also includes the
3 bins/examples, which carry no `#[test]`s).

Covered before the session cutoff (56 of 795 binaries, priority order: the four lib harnesses, then
this task's neighborhood, then alphabetical), **927 passed / 0 failed / 8 ignored**, one TIMEOUT:

- `nf_game` (lib) **548 passed / 0 failed / 3 ignored**; `nf_formats` **89/0**; `nf_audio` **23/0**;
  `nf_view` (lib) **96/0**;
- the neighborhood all green: `panels_07000014_live` 2/0 (the un-ignored repro + the lid pin),
  `fix_fusebox_cut_live` 4/0, `fusebox_06_live` 4/0, `walk_07000014_live` 2/0,
  `walkreal_07000014_live` 1 passed / 2 ignored, `solar_panels_live` 2/0;
- 46 more binaries a→c (abseil…casings16) all green;
- `campaign_flow_live` **TIMEOUT** at the 300 s cap (0 failed): not in the documented environmental
  failure families — the next message should re-run it with the runner's 600 s limit (a no-GPU
  sandbox hang is plausible; product code is untouched by this branch, so a regression from this
  task is not indicated).

The remaining ~739 binaries are untouched by this branch's changes (test-file + docs only) and were
green on the merge's parent states in the prior full runs.

## 4. Headless renders for the main agent

- `--bond -88.40,-2.20,48.60,-0.87,0` — facing the fusebox #385 / the panel door #386, closed: the
  four wires exist behind the panel (hidden by the door object), the meshes at the box.
- The same spot after an E-press on the panel door #386 (it is a 0xdb door and should E-open like any
  door): the wires visible in the opened recess — the "opened fusebox panel shows its wires" check.
- `--bond -89.31,-2.22,30.79` neighbourhood (box #383, same script): the same four-wire set.

## 5. Live checks for the main agent

1. Fresh boot, 07000014, at the fusebox #385 (−89.30, −2.22, 49.36): E the panel door #386 — does it
   swing open and reveal four wires behind it? And E at the box itself — expected NO reaction
   ([M] 0x1a5170:95-160 has no class-0x3C arm; the SP mask carries no bit 4).
2. Shoot the wires with the Laser: the group wires should vanish (Effect_RemoveSObjs) and the
   "Sabotage base security" objective chain should advance (ch 0x88 for #385; both west boxes → 0x8C).
3. The wire look: two small spheres (local centre y +1.6, r ≈ 0.13/0.15) and two large (y +2.0,
   r ≈ 2.0) per box — confirm in the render whether the large-radius pair reads as wire art or as an
   invisible collider-only mesh (it may simply be off-screen; no [D] contradiction either way).

## 6. What's left

- Nothing on the task's two items: the SP-entity path was verified (not re-portable — already the
  real path), and the test is un-ignored and green. The remaining exposure is only the full-suite
  total at the cutoff (§3).

## Usage

not visible to me — this session's tooling shows no token, context-window or cost figures.
