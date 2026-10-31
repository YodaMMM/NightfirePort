# level-14-missing-group-guards — report

Branch `s/level-14-missing-group-guards` (from origin/master b44ac857 + merge of
s/level-14-door-guard-release). Status: **done — the three guards are difficulty-gated, the port
is faithful, the release is pinned at both difficulties.** No parse/attach fix was needed; the
predecessor's "missing from drones.list at load" was the game's own create gate, which the port
already mirrors.

## The answer: `Drone_Create`'s difficulty gate, verified before any code change

`Drone_Create` 0x136be0 (first branch) creates NOTHING for a placement whose level tag exceeds
the load's difficulty: `if ((Drone_bDisableSystem == '\0') && (*(uint *)(param_3 + 0x34) <=
(uint)GameState._40_4_))` [M 0x136be0]. `GameState._40_4_` is Ghidra's DECIMAL-40 field = hex
+0x28 = 0x2a3790 — exactly the port's `Drones::difficulty` (1 Operative / 2 Agent / 3 00 Agent /
4 bonus [D difficulty table 0x2df4c0; M C_SBNFDFCTY_Handler 0x216dc4; boot default 2 M bootup
0x1c7e78:25 @0x1c7f88; mission-start force 1 M Menu_RunMiniMission 0x1ffb90:22 with L the
level-start snapshots' +0x28 = 1]).

The three placements' parameter 2 [D docs/levels/07000014-audit.md :73-76/:85-87]:

| template | param 2 | at difficulty 1 | at difficulty 3 |
|---|---|---|---|
| #723 (-32.78, 28.48) | 0x3 | gated — never created | created |
| #784 (-36.18, 27.76) | 0x2 | gated | created |
| #785 (-31.74, 25.16) | 0x3 | gated | created |

Every template the port DOES carry at Operative has parameter 2 = 0x1: the group-0xd
#721/#722/#724/#783 and the group-9 #778/#780; the group-9 #779 = 0x2 / #781 = 0x3 are gated the
same way — which is why the probe's released group-9 pair was exactly #778/#780. The test loads
via `nf_view::load` → difficulty 1 (Operative), so the port's absent set is exactly the
parameter-2 ≥ 0x2 set. **The port is right at that difficulty**; the mirror is
`level_drones`'s `if param(2) > d.difficulty { return None; }` (drone.rs, cited "[M 0x136be0:49]"
in situ) plus the scene-side ghost-guards-990 gate.

## The pin: `door_guards_locked_door_07000014_live`, per difficulty

Two new tests (released set = enabled ∧ alive ∧ not-enabled-at-load — the wake-0x8C templates
sit enabled=false / health=10.0 until spawners #1333 (group 0xd, max 7, waves 1) / #720
(group 9, max 2) pick them [D audit :1527/:1531; M 0x137cb0]; mux #717 0x8C = 0x87 AND 0x88 [D
audit :1411]):

- **Operative (difficulty 1)** — the release brings **6**: the ambush trio #721/#722/#724
  holding 0x38 at their [D] placements, #783 routing to the door hold, and the group-9 pair.
  #723/#784/#785 do not exist at load and still do not exist after the release (asserted both
  times, enabled or not).
- **00 Agent (difficulty 3, `load_with_carry_difficulty`)** — the tagged three EXIST at load as
  wake-0x8C templates (asserted not enabled at load) and the release brings **9**: 7 group-0xd +
  the group-9 spawner's 2 picks of its now-4 candidates.
- **The ambush set is the QUARTET, not the trio** — found while pinning: #723 shares the
  ambush prop word 0xf1dfc8c1 with #721/#722/#724 [D audit :85-87], so it is AmbushWait too and
  holds **0x38 at 00 Agent as well** (asserted), while #783/#784/#785 (prop words 0xf1dec9c1 /
  0xf1dec8e1 / 0xf1dec9c1) are the full-alert routers — all three reach the door area in the
  00-Agent run (0x73 steps at (-50.50,39.83)/(-51.03,41.17)/(-49.28,39.44)). The predecessor
  report's "only ONE full-alert guard (#783)" was right at Operative and the quartet/trio split
  corrects its reading of #723. The trio-stays-0x38 premise pin the task asked for is in at
  both difficulties [M 0x161560:28-70 — no channel read; Bond held 25+ units away in the hall].
- The pre-existing three tests are untouched (2 active pass; the 0x5f-hold test stays
  `#[ignore]`d — its two named blockers are unchanged by this task).

## What was run (totals)

- `door_guards_locked_door_07000014_live`: **4 passed, 0 failed, 1 ignored** (17.4 s).
- Every `*07000014*` binary, 16 of them (capture_placement 1/0, census 0/0+1 ign, cutscene_camera
  1/0, cutscene_door 3/0, destroy_wreck 2/0, door_guards 4/0+1 ign, door_wait 1/0,
  escape_scene_actors 2/0, fusebox_geometry 1/0, fusebox_slots 2/0, interactables 9/0,
  invisible_guard 2/0, panel_sequence 2/0, panels 2/0, walk 2/0, walkreal 6/0): **37 passed,
  0 failed, 2 ignored**.
- `nf-game`: **953 passed, 0 failed**.
- Full suite (tools/test/suite.py / chunk_suite.py): **could not complete in this sandbox — the
  9.9 GB root disk hit 100% (ENOSPC) during the remaining first builds (nf-audio tests, nf-view
  examples)**. Freed ~5 GB of unrelated caches (browser caches, node_modules, unused python
  wheels) but the container's `df` did not return the space (snapshot-quota behaviour); the
  warm-target chunk run still died on ENOSPC. Nothing behaviour-changing is untested: this
  branch's only code change vs master is the new test file (asserts only), and the touched
  binaries all pass above. Resume on a roomier sandbox with:
  `NF_ISO=<abs> CARGO_TARGET_DIR=$HOME/NightfirePort/target python3 tools/test/chunk_suite.py /home/z/suite14_state.json --budget 400`
  repeatedly until ALL DONE (note: chunk_suite defaults CARGO_TARGET_DIR to /home/z/target-cache
  — override it or it rebuilds the world cold).

## What is left / for the next session

1. **The group-9 route divergence (the ignored 0x5f test's blocker (2), reframed).** At
   Operative the port's #783 reaches the door hold, but its group-9 pair stays near the spawn
   area (0x73 cycling at (-29.25,15.12)/(-22.87,40.10)) while the game's door pair is TWO guards
   2.4 apart [L 2026-10-30] — presumably #783 + one routed group-9 guard. Why the port's pair
   does not route to Bond is the remaining gap; the release population itself is now proven
   correct.
2. The full-suite chunk run on a sandbox with disk headroom (command above), to carry the
   suite standing for the test-only change.

## Live checks for the main agent

1. **Released population at Operative vs 00 Agent.** On a fresh 07000014 boot, note the
   difficulty the session plays at (the mission start forces GameState+0x28 = 1 [M 0x1ffb90:22]
   unless a higher difficulty was chosen), then set 0x87+0x88 (fuseboxes #383/#385), hold Bond
   in the generator hall, and count alive released guards over 3000 frames plus each guard's
   state word. Expected from the [D] parameter-2 table + [M 0x136be0]: **6 at Operative** (the
   quartet's #721/#722/#724 at 0x38, #783, and two group-9), **9 at 00 Agent** (quartet at 0x38
   — #723 included — + #783/#784/#785 + two group-9). This supersedes the predecessor's
   live-check expectation of 9 at the default difficulty: at mission-start difficulty 1 the
   game can only create the param-2 = 1 templates.
2. **#723's state at 00 Agent** — the pin says it holds 0x38 with the trio (ambush prop word
   0xf1dfc8c1 [D audit :85-87]). If the game ever shows #723 at the door or in a combat state,
   the mode-resolution chain (0x14c938:10-14 with the AMBUSH prop) needs a redo.

## How this was verified

- Decompile reads: 0x136be0 (the gate, quoted above from work/decomp/ACTION.ELF this session),
  with the port mirrors read in situ (drone.rs level_drones, scene.rs ghost-guards-990).
- Disc data: docs/levels/07000014-audit.md :73-76/:85-87 (drone placements with parameter 2 and
  the prop words), :1411 (mux #717), :1527/:1531 (spawners #720/#1333).
- Port runs: the two new pin tests plus the 07000014 family and nf-game totals above; the
  released-set dumps (Operative: 6 with the trio 0x38; 00 Agent: 9 with the quartet 0x38 and
  three routers at the door area) are in the test eprintln output quoted in the WORKLOG entry.
- No decompile-backed test was weakened; the only existing-test edit is the new file content
  appended to door_guards_locked_door_07000014_live.rs.

## Usage

not visible to me — no token, credit or context-window figures are exposed to this session. The
only proxies visible are the shell's wall-clock timestamps: setup + branch 09:52-09:57 UTC, the
[D]/[M] gate verification 09:58-10:05, the pin-test write/build (incl. an ALSA dev-lib workaround
and a disk-full fight) 10:06-10:50, the runs 10:50-11:00, the report 11:20-11:30 UTC 2026-10-31.
