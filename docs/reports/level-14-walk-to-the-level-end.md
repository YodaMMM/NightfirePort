# level-14-walk-to-the-level-end

**Branch `s/level-14-walk-to-the-level-end`, 2026-10-30, worker session 08:37–10:15 UTC.**

## What was done

`walkreal_07000014_live::walk_the_infiltration_to_the_delta_exit` was extended from the delta
exit — where the merged walk ends on the E-press that arms the exit — to the level's actual end:
the next level's load, run the game's own way.

1. **The exit hold drains on real ticks.** The E-press on exit #944 pushed
   `GS_PauseGame(1)` + `GameFlow_PushState(300.0f, 7, 0xff)` [M 0x1b2318:90-92]. The walk now
   keeps ticking (`Input::default()`) until `drones.exit_hold` clears: the port's `step_bond`
   runs the mission layer (`tick_mission_during_hold` — `Mission_Update` sits at 0x1c9790:17,
   outside the pause gate, which is where objective 5 folds [M 0x1827d8 case 3]) and then the
   300-frame fade [M 0x1c9790 tail; 0x1c98c8:173-189 pops state 8 onto state 3].
2. **The next level loads WARM on the game's own save.** When the hold clears the walk captures
   `warm_exit_save(sim)` — `Trigger_Activate`'s `Player_RamSave` 0x195670 capture, exactly what
   the host's `start_async_load` campaign arm hands the loader [M 0x1b2318:97-116] — and calls
   `load_with_carry("07000015.bin", Some(&save))`, the warm arm (the plain wrapper's
   difficulty 1 / bonus 0 differ from the front end's picks; the warm `Player_RamLoad` passes
   boost 0 on all eight call sites [M 0x1a7ef8], so the difference is unobservable here — [S]
   noted, not asserted).
3. **The arrival is pinned, every claim cited:**
   - Bond spawns on the silo gantry start #1 (93..96, 40..43, −2..2) [D spawn #1] — the warm
     load changes only the player state [M 0x1957a8:9 arm];
   - the switch request equals the exit's (0x2) — case 0x07000015 restores the saved weapon as
     the request [M 0x1a7ef8 case 0x07000015; 0x1957a8];
   - the health equals the exit's [M 0x195670:16 + 0x1957a8];
   - the owned weapons ride (P2K — the gate that skips the fresh-14 kit [M 0x1a7ef8 case
     0x07000015] — crossbow 0x11, laser 0x4e) [M 0x195670:7];
   - the carried channels 0x54 / 0x85 / 0x8c / 0xce stay on [M 0x195670:13 + 0x1957a8], and
     0x4b was still off at the exit (the walk's own channel state — the silo's spawn plate owns
     it);
   - part 1's five records ride untouched (the exit values compared exactly) — `Mission_Init`
     resets only the current + future parts [M 0x182270:68-129];
   - after the arrival settles, the spawn plate #484 fires ch 0x4b and
     "Find the exit to Omega Sector" announces (`states[21][0] == 3`) off the carried load —
     MissionData row 21 is the part's only record [D vaddr 0x2a42d8; M 0x1827d8 case 1].

   **No port gap was hit**: the drain → capture → warm-load path worked as ported, first run.

4. **The stale "STILL #[ignore]" comment above `walk_infiltration_to_handback`
   (walkreal_07000014_live.rs ~323) is corrected.** The helper is no test and carries no
   `#[ignore]`; both tests that share it are un-ignored and green. The old note's leg-2 wedge at
   the facility's west double door was the walk's own wrong premise, not a port gap:
   Door_Slide_11A/B (plc 151/152, ch_lock 0x85 [D placements]) is sealed until the sabotage
   cascade's death 0x85 unlocks it (auto-open on the rising edge [M 0x1363c8:88-107]), and the
   west wing is reached the game's own way — the south loop, which never crosses the 0x85 plane
   [D nav path 0], to the Kiko touch #381 whose cut-scene 0x0600062F carries Bond west; the
   doorway is crossed only after the cascade (the delta-exit walk's legs 2-4).

## How it was verified

- `walkreal_07000014_live` — **6 passed / 0 failed / 0 ignored**, run three times: on the
  pre-merge tree, and twice on the final tree merged with `origin/master` (batch-1030b,
  04820075), once standalone and once inside the chunked suite (binary total quoted in the
  suite section). The extended walk itself: 33 s standalone, 53 s in the 6-test binary.
- The full suite on the final merged tree, via `tools/test/suite.py`'s logic run in
  **foreground chunks** (see the environment note below): **237 of 934 test binaries recorded —
  772 passed, 1 failed, 38 ignored**. The one failure is `probe_pylon_render` (0/1), a
  **no-GPU environment failure**, not a code regression: `wgpu::Instance::request_adapter`
  returns `NotFound` (`supported_backends: VULKAN | GL`, none exposed) in this sandbox — it
  renders 0700000c on the GPU and cannot run here (lesson 6: the worker sandbox has no GPU; the
  main agent's render pass covers it). No FAILED/TIMEOUT lines otherwise.
- Not recorded in this window: `net_soak_faults_live` needs a ≥600 s slot (its 10-game-minute
  soak outgrew the chunk cap; the state's premature 120 s TIMEOUT was discarded) and ~697
  binaries were never recorded — 121 of them were linked and RUN by the first chunk before a
  tool-call kill lost its (then end-of-chunk-only) state, the rest never linked: the sandbox
  disk (9.9 GB) cannot hold the ISO + the extracted trees + the full 934-binary release build at
  once (the build alone peaked >5.3 GB and link-stage ENOSPC produced transient bus errors
  before space was freed). I freed ~2.6 GB (orphaned duplicate rlibs, stale examples, the
  re-downloadable `work/`) and ran the remainder chunked with delete-after-run. The chunk
  runner is `scripts/suite_chunk.py` (kept out of the repo — scratch, per the commit rules).

## What's left

- A full-suite pass on a machine that can hold all 934 binaries (or the same chunking with more
  window), plus `net_soak_faults_live` in a ≥600 s slot, and `probe_pylon_render` on a GPU box
  — none blocked by code; every recorded binary but the GPU one is green.
- Nothing else. Every test in `walkreal_07000014_live` is green and the walk now covers part 1
  end to end, spawn to the next level's load.

## Live checks for the main agent

None needed for this task (no visual work; the carried-load arrival is pinned by channels,
mission records and positions, not pictures). Standing question from the earlier merge, not
this task's: door #834's role (the walked route passes west of its wall run).

## Usage

not visible to me — this session's environment shows no token, credit, context-window or cost
figures anywhere I can read. (The tool-call transcript lengths are the only proxy and they are
not a listed figure, so per the rules I don't estimate.)
