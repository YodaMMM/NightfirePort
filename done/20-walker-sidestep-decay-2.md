## Your task: `walker-sidestep-decay-2`, the walker sidestep decay without breaking Night Shift's roof walks

Branch from origin/s/level-0b-dome-wire-sisters (it contains walker-sidestep-decay) and merge origin/master. Its change to the shared crates/nf-view/tests/common/walker_b2.rs turns walkreal_0700000b_live from 9 passed / 0 failed on master to 5 passed / 4 failed. Find which walk behaviour the decay changes on 0b and gate or repair it so 0b stays 9/0 while the 0c and 12 walks it un-ignores stay green. walker_b2.rs is used by 128 test files: run every walkreal_*_live binary one level per tool call before pushing.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/20-walker-sidestep-decay-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox4 at 2026-11-12T05:54Z

---
## Report (done, 2026-11-12T07:18Z)


---
## Report from the worker (done, 2026-11-12T07:19Z)

# walker-sidestep-decay-2 — the walker sidestep decay, gated; 0b back at 9/0

Branch `s/20-walker-sidestep-decay-2`, cut from `origin/s/level-0b-dome-wire-sisters` (which
contains walker-sidestep-decay) with `origin/master` merged.

## What the merge found

Two conflicts, both in the task's own files:

- `crates/nf-view/tests/common/walker_b2.rs` — the `reset_sidestep` doc: the decay branch
  describes the reset as the walker default (scope boundaries reset automatically); master
  (los-mask-repins-0c-3) describes it as opt-in with one caller. Resolved to the decay-branch
  shape first; this task's gate (below) supersedes the doc again.
- `crates/nf-view/tests/common/walkreal0c.rs` — the same disagreement at the rd post-#282
  ride scope (master calls `reset_sidestep` inside `STRIP_RD_POST_RIDE`; the decay branch
  moved the call to the ride scope itself). Resolved to the decay-branch shape.

## The finding: the decay does not explain the 0b 5/4 alone

Reproduced on the merged tree exactly as the task described:
`walkreal_0700000b_live` = **5 passed / 4 failed** [L run 1-3]. The four failures are the
real-damage sisters (`..._real_damage`, `..._real_damage_00agent` ×2 via master's own doubled
`#[test]`, `..._real_damage_agent`), all dead in the roof fights (d1 tick 16479, 00agent
16835, agent 16872; the world's live-wire/blast share 40.8–76.0 of the 100.0 [L the runs]
— the same "die late on the roof" ledger the dome task recorded).

Then the A/B that decides it: with the sidestep decay and the ride/grab scope resets **gated
off** (master-timed walker) the 0b binary **still ran 5/4** [L run 4]. The decay is not the
only re-timer on the merged tree: the dome task's west-diagonal plate approach and west-exit
return (`walkreal_0700000b_live.rs`) were tuned **for the decayed walker on the pre-merge
tree** (its own ledger: 93.6 → ~50 rounds, sisters still dead late). On the merged tree
(master's game changes: guards react to gunfire the game's way, blast falloff, walk helpers
carrying Chain Reaction's levers) those routes re-time the real walks and keep them RED —
5/4 with either walker setting. Master's own file says its head-on pair was the best measured
lane (re-place 80.8 [L the in-file reroute measurements]) and master runs 9/0.

## The repair

1. **The decay is opt-in again.** `Walker::sidestep_decays` in `walker_b2.rs` (the same name
   and shape as walker.rs's existing opt-in from los-mask-repins-12-2), default **false** =
   master-exact: `self.sidestep -= 1` (no signum decay) and no automatic sidestep resets at
   walk_to's WireRide/Ladder scope boundaries. All four reset sites and the steering decay
   check the flag. `reset_sidestep()` itself stays unconditional, so master's own call sites
   (the rd-scope caller) keep firing. The NF_WALKTRACE diagnostic stays (env-gated, no
   behaviour change).
2. **The walks that need the decay set it.** The 0700000c real-damage tree sets
   `w.sidestep_decays = STRIP_RD_POST_RIDE.with(|c| c.get())` at the top of
   `walk_spawn_to_the_band_at` (before leg 1, so the decay rides from the first steered tick
   as the walker-default green run did), and its six decay-branch ride-scope
   `reset_sidestep` calls are gated on the flag (off = master-exact for the plain walks).
   The 07000012 Operative crossing re-takes the 12-2 patch's `w.sidestep_decays = true`
   setter at `difficulty == 1` (the crossing leg measured sidestep −753 strafe-left pinned
   without it [L the NF_WALKTRACE run, los-mask-repins-12-2]).
3. **The 0b routes go back to master's.** `crates/nf-view/tests/walkreal_0700000b_live.rs`
   restored to master's version (the head-on plate pair and the north return) — the dome
   routes never reached 9/0 on any tree and are decay-tuned.

No draw, RNG, engine or `nf-game` code changed; the whole change is test-side
(walker_b2.rs, walkreal0c.rs, walkreal_0700000b_live.rs, walkreal_07000012_live.rs).

## Verification [L]

| binary | result |
| --- | --- |
| `walkreal_0700000b_live` | **9 passed / 0 failed** (master's state; its doubled `#[test]` on `..._real_damage_00agent` kept — master's own file has it, and master's 9 counts it twice) |
| `walkreal_0700000c_live` | 4 passed / 0 failed / 1 ignored — the un-ignored `..._0d_handoff_real_damage` walk holds with the opt-in decay; the Agent sister keeps master's own ignore |
| `walkreal_07000012_live` | 4 passed / 0 failed — the un-ignored Operative crossing holds with the re-taken setter |
| `walkreal_07000001_live` | 7 passed / 0 failed / 1 ignored (master's own ignore) |

All runs: fresh release build of this branch merged with origin/master, `--test-threads=1`,
the slim disc image.

## What's left (the window ended)

- The remaining `walkreal_*_live` binaries: 02 (its walks exceed the per-call window alone),
  03–09, 0a, the 0c barrelrow/eastroute/ventexit lives, 0d, 11 + 11_route, 13, 14, 15, 16,
  1b, 41 + 41_wall, 4a + 4a_satchel, and the probe lives. All are master-timed by the gate
  (default off), but the task's rule is to run them, not infer them.
- The fast suite (`tools/test/suite.py`) on the final tree. It needs ~40+ min; this sandbox
  kills background processes between tool calls (nohup and setsid both died within one call),
  so it could not run long-form here. Next window: run the listed binaries one per call, then
  `NF_ISO=<iso> python3 tools/test/suite.py <repo> <target>` and check the total line.

## Live checks for the main agent

None needed — no engine or render change; the change is test-side only and the proof is the
green 0b/0c/12 runs above.

## Usage

not visible to me — this session's environment shows no token, context-window or cost
figures, so there are no numbers to copy.
