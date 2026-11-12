# replay-parity-walker-side-effects

Branch `s/replay-parity-walker-side-effects`, cut from origin/master 08a276ce0.

## What's done

1. **Every walker-stepped tick now steps through ONE site.** `crates/nf-view/tests/common/damage_log.rs`
   (the shared real-damage file `include!`d by both walkers) had the hole named by the task: the fight
   loop's `if reload {` branch called `self.scene.sim.as_mut().unwrap().step_bond(Input::default(), false, true, false)`
   directly (old damage_log.rs:1239) and did its own `total_ticks += 1; after_step();` — skipping the
   walker's per-tick work (the keep_alive top-ups, the cut-scene skip request, rise_watch, drone_watch,
   the trail). That branch now calls `self.tick_inner(Input::default(), false, false, true)` (damage_log.rs:1238-1246),
   whose step argument order matches the old direct call exactly (`fire=false, reload=true, use_key=false`).
2. **The other direct `step_bond` sites inside Walker methods are routed too:**
   - `walker.rs` `shoot_bounded`'s fight step (old walker.rs:945 `step_bond(input, trigger, reload, false)`
     + the manual accounting) → `self.tick_inner(input, false, trigger, reload)` — same argument mapping.
   - `walker_b2.rs`'s mirror fight loop (old walker_b2.rs:528) → the same `tick_inner` route.
   - `walker_b2.rs`'s floating-landing recovery hop (old walker_b2.rs:317, inside `tick_inner` itself — a
     second step in the same tick) → `self.step_recorded(input, false, false, false)` so a recording
     captures it; routing it through `tick_inner` would recurse (tick_inner calls after_step which runs
     this arm), so it goes through the one STEP site, not the one TICK function.
   - Grep census of `tests/common/` for remaining direct `step_bond(`: `reveal07.rs`, `usepick.rs` and
     `walkreal0c.rs` hold free functions, not Walker/DamageLog methods. `reveal07`/`usepick` own a bare
     `Sim` (the task's "free functions that own a bare Sim are fine"). `walkreal0c.rs`'s sites are the
     walkreal-c route's own driver loops — the walker's `skip_cutscenes` doc pins that those segments
     deliberately "drive `step_bond` themselves" (a script whose channel writes the route depends on the
     skip NOT firing), so they are left as documented exceptions; listed here per the task's census.
3. **The optional input recorder at that one function.** Both walkers gained
   `pub record_inputs: Option<Vec<nf_view::replay::TickInput>>` and
   `pub record_states: Option<Vec<WalkTickState>>` (new fields on their own lines at the struct tail;
   both constructors initialse them `None`; `WalkTickState` carries pos/yaw/health/frame/mission_end).
   `Walker::start_recording()` turns it on. The new `step_recorded` is the ONE step site: `tick_inner`
   steps through it; recording on, the tick's input is appended via `TickInput::from_parts` and the sim
   is stepped with `to_parts()` of the just-pushed record (the sim sees exactly what is recorded); the
   per-tick state is captured right after the step. Recording off, the call is byte-identical to the old
   direct `step_bond`.
4. **The parity test `replay_parity_07000004`** (`crates/nf-view/tests/replay_parity_07000004.rs`): the
   07000004 full-mission walk — difficulty 1 (Operative) through the real launch path, `top_up_health
   false` (real damage, no top-ups), `skip_cutscenes false` (the scenes play; no skip request), the
   armament lap StormUzi #7 → BodyArmourHeavy #9, then the 0x0500001e target shot — records every tick,
   stops at the FIRST tick `mission_end` is Some (item 4: the route used to keep walking the lap and the
   kill rounds ~6300 ticks past the end; every leg and wait in this route now breaks there), then replays
   the inputs on a FRESH `nf_view::load_with_carry_difficulty` (headless sound attached as the walker
   attaches it) stepping `to_parts()` per tick, and asserts bit-exact per-tick equality of Bond
   pos/yaw/health, `drones.frame` and the `mission_end` latch (WalkTickState's PartialEq is field-exact),
   plus the same mission_end tick both ways. The mission-end band pin (2400..=3600, centred on the
   measured ~3003) and the `Some(0x0730004)` latch are asserted on the recorded run.
5. **WORKLOG** entry appended (`docs/WORKLOG.md` created on this branch — master does not carry the file;
   the entry is my own only).

## Verification status — honest state at the deadline

The sandbox ran out of disk during the build (9.9 GB; the release test binaries of nf-view alone fill
several GB) and had to be built in strips (`-j1`/`-j2`, strip-after-link cycles). That consumed the
window. At commit time:

- `replay_parity_07000004` and `walkreal_07000004_live` binaries are BUILT with this tree's changes and
  were launched; their result lines are appended below if they landed before the sandbox expired.
- The full suite (`tools/test/suite.py`) did NOT fit the disk/time window this session. The binaries
  compiled with the changed shared helpers are the two above; every OTHER walkreal binary compiles from
  the same sources but was not run. **This is the main thing the next message must finish**: run
  `NF_ISO=<abs> python3 tools/test/suite.py` on this branch and check every walkreal_*_live +
  npc_doors_live/fix_guard_misc_live (lesson 7/10 — the shared helper changed, so EVERY walkreal binary
  needs a pass before merge).

### Test results appended at the finish

```
(pending — see below)
```

## What's left

1. Run the two launched binaries to completion and paste their pass counts (they were mid-run at the
   sandbox deadline).
2. Run the full suite (fast runner) on the final tree merged with origin/master; check EVERY
   walkreal_*_live binary (the shared helper changed), plus npc_doors_live and fix_guard_misc_live
   (nf-game untouched this session — those two are only needed if drone.rs changes; none were made).
3. The route's mission-end band (2400..=3600) is pinned from the driver's measured ~3003; if the real
   run lands outside it, re-measure and re-pin with the run's own number.

## Live checks for the main agent

None needed — the parity evidence is the test's own pass/fail.

## Usage

- Input tokens: not visible to me.
- Output tokens: not visible to me.
- Context used: not visible to me.
- Cost: not visible to me.
