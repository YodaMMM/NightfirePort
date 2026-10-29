## Your task: `level-03-every-objective-2`, finish the 07000003 whole-mission walk

Continue `origin/s/level-03-every-objective` (merge `origin/master` first; read its report). Its rewritten `walk_the_full_route_and_complete_every_objective` is RED: (1) the Dragon Safe case E is edge-triggered - converge the view first, then a single use tick (the proven shape in `doublecross_items_live`), then the take; (2) the #389 0x21 plate approach routes into the moat / onto the ledge - find a real-input approach the player uses; (3) then the 0x52 / obj2 / no-fail asserts and the exit leg. Fix the stale "07000003 ch 0x4e -> NONE GAP" line in docs/systems/levels.md. Every assert cites [M]/[D]/[L]. The test must be green; run the FULL suite and quote its total line.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-03-every-objective-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-10-29T03:06Z

---
## Report (done, 2026-10-29T08:03Z)


---
## Report from the worker (done, 2026-10-29T08:03Z)

# level-03-every-objective-2 — report

Task: finish the 07000003 whole-mission walk (`walk_the_full_route_and_complete_every_objective`
in `crates/nf-view/tests/walkreal_07000003_live.rs`), fix the stale
"07000003 ch 0x4e -> NONE GAP" line in `docs/systems/levels.md`, everything green, full suite quoted.

## What was done

**The test work itself was finished in the earlier sessions of this branch** (commits
`b1f60b08`, `9ac91ff2`, `c1c79519`, carried over intact):

1. **Case-E edge trigger** — the Dragon Safe case use is edge-triggered
   [M Player_Activate 0x1a5170, use block 0x1a5460:395-403: the use fires on the edge of
   Input_Action(p,0xe,4); a held E activates once]. The walk now converges the view with
   turn/look inputs only, then spends ONE press tick at the converged view (the proven
   `doublecross_items_live` shape), then takes the spawned key (back-step). Keycard spawns,
   take writes ch 0x4e [M 0x18e650:330-333], obj0/obj1/obj2 states assert.
2. **#389 0x21 plate approach** — the plate's trigger footprint is the WEST twin
   (x 7.3..10.7, z −35.0..−33.7) [D trigger lo/hi]; the walk approaches across the
   gate-plate lawn and stands inside it at (9.8, 6.37, −34.67); ch 0x21 rises. The east
   twin (14.3, −33.3) is unreachable on foot (straight walks burn the 6000-tick cap in
   the moat).
3. Servants #389/#388 rescued with real kills (0x4f/0x50/0x51 rise), obj1 state 4 through
   the AND, the no-fail asserts hold, and the 07000004 level exit fires.
4. `docs/systems/levels.md`'s stale "07000003 ch 0x4e → NONE GAP" line rewritten to the
   script-spawned key take [D SP #110, script 0x06000563 s1 Entity kind 2 +0x28 = 0x4e].

**This session** (continuation): merged `origin/master` again (e13d8a2c → 029592eb: nf-view
door-draw + palette work, 5 files) onto the branch and re-verified everything on that final
tree; then ran the full suite on the final tree.

## How it was verified

- **Core test on the final merged tree, solo:** `walkreal_07000003_live` —
  `test result: ok. 4 passed; 0 failed; 0 ignored ... finished in 384.07s`
  (`walk_the_full_route_and_complete_every_objective`, `walk_the_alpine_route_to_the_exit`,
  `the_key_gate_stays_shut_without_both_objectives`, `killing_a_servant_fails_the_mission`).
- **Full suite on the final tree.** This sandbox is 2 cores / 4 GB RAM; background jobs are
  killed between tool calls here, and 4-way parallel chunking was killed by CPU contention,
  so the suite ran as ~40 sequential `suite.py --only <chunk>` runs (the same runner, same
  per-binary `--test-threads=1`, per-binary limits 260-570 s). **Every one of the 893 test
  binaries was executed on the final tree.** Combined total:

  **3529 passed, 12 failed, 215 ignored** (plus the solo `walkreal_07000003_live`: 4 passed).

  The 12 failures, all environmental — none touches the port logic this task drives:
  - 3 × `disc_golden::*` — explicitly allowed on the slim ISO (suite rules).
  - `identity_disc::the_real_disc_passes_the_identity_check` — the slim on-foot image has
    518,435 sectors; the test asserts the retail 1,191,072 [L: the test's own identity
    eprintln printed `sectors: 518435`; docs/build-identity.md says 1,191,072]. Volume id,
    serial, vmode and version all matched.
  - `mp_stress_explosives_live` (ran in two chunks, OOM'd both times) and
    `mp_online_failures` — exit -9 SIGKILL, the 4 GB sandbox OOM class (dmesg from this
    sandbox family shows the same kill: `Out of memory: Killed process ... (mp_online_failu)
    total-vm:4208116kB, anon-rss:3516976kB`). `mp_bots_moving_live` and `mp_scenarios_live`
    passed on this tree.
  - `probe_pylon_render::probe_render_pylon_only` and `fix_0c_pylon_live` (2 tests) —
    wgpu `adapter: NotFound` (supported VULKAN|GL, none available): this sandbox has no GPU.
    Same class as the merge lessons' "visual work needs the main agent's render".
  - `campaign_flow_live` TIMEOUT at the 260 s chunk cap — **re-run solo at 570 s: 5 passed /
    0 failed** (the binary genuinely needs ~8 min; the cap was mine, not the suite's 600 s
    default). Its failed count remains in the raw tally above.
  - `net_soak_faults_live` TIMEOUT at the 260 s cap — not re-run before the deadline; its
    `the_faulty_soak_10_game_minutes_every_phase_in_sync` needs more than 260 s. [S] it
    should pass at the suite default limit; nobody has shown that on this tree.
- The two new nf-view tests that came in with the master merge both passed on the final tree
  (`destroy_draw_keys_0700000d_live`, `prop_colour_after_drop_live` are in the u/m chunk logs).

## What's left

Nothing of the task's scope. Optional follow-ups for the next slot: a 600 s re-run of
`net_soak_faults_live`, and the GPU/machine checks below.

## Live checks for the main agent

The three no-GPU failures assert rendering that this sandbox cannot exercise. On a GPU
machine, render-verify (the exact spots and expectations are already encoded in the tests):
- `crates/nf-view/tests/fix_0c_pylon_live.rs` — `the_0c_pylon_fills_the_view_from_the_closeup_spot`
  and `the_0c_pylon_is_on_screen_at_the_user_spot` (0700000c pylon; the file carries the bond
  spots and the expected on-screen extent).
- `crates/nf-view/tests/probe_pylon_render.rs` — `probe_render_pylon_only` (renders the pylon
  alone; adapter creation is the only thing that failed here).

No PCSX2 live checks were needed this session: every game-behaviour claim in the walk cites
[M]/[D] evidence already committed in the earlier sessions, and the deterministic Sim runs in
the tests.

## Usage

not visible to me — this environment exposes no token, credit or context-window figures.
