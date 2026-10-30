# level-02-captors-real-approach — report

Task: replace the `clear_captors` teleport in
`walkreal_07000002_live::walk_the_castle_all_objectives_to_the_exit` — the helper stood Bond
2.5 units beside each captor guard by writing `bond.pos` before the shot — with a real-input
approach, keeping the walk green with NO `bond.pos` writes after the spawn. Branch
`s/level-02-captors-real-approach` = origin/master (f9d1971d).

## What's done

**The teleport is gone and the walk is green.** `clear_captors`
(crates/nf-view/tests/walkreal_07000002_live.rs:64-79) now walks the nav web into the captor
room through the player's own input and fires through the real trigger path:

- round 0 walks to the geisha's spot (`around` — the natural player route into the room),
  later rounds walk to the guard's CURRENT position (the AI moves between rounds), each via
  `walk_nav_to(dest, …, 3.5)` — the walker's nav-graph route (per-path A* then the cross-path
  roadmap, walker.rs:781-795, 1173-1215) driven by `step_bond`/`Input` only;
- the shot itself is `w.shoot(di, …)` (walker.rs:587-717) — the player's own aim/reposition
  loop, including its nav-graph flank arm when LOS stays blocked 9 s (walker.rs:640-700),
  unchanged this task;
- zero `bond.pos` writes remain reachable from the walk: the file's only occurrences are the
  doc comment and a read-assert (line 87); an independent audit (subagent, 5/5 PASS) grepped
  the whole reachable path (`walk_nav_to` → `walk_to` → `tick` → `step_bond`, `shoot`) and
  found all 30 `bond.pos` uses in walker.rs are reads — the only sim mutation is
  `step_bond(input, trigger, reload, false)` (walker.rs:666).

**Measured results** (`cargo test --offline --release -p nf-view --test
walkreal_07000002_live`, two runs, both green):

- **walkreal_07000002_live: 1 passed; 0 failed** (39.6 s and 39.3 s; the full objective route
  in 27124 ticks, falls: []).
- geisha 1's captors: approached through 3+1 nav legs ((−41.55, 6.45, 7.82) → (−43.17, 6.45,
  7.90) → (−45.04, 5.88, 10.97), then (−44.13, 5.85, 14.38)) and killed through the real aim
  loop at dist 7.1 and 1.2 — no flank, no repositioning, no survivor.
- geisha 2's three captors: approached through 9+2+1 nav legs across the east courtyard
  ((−28.03, 6.47, −20.34) … (−13.05, 6.47, −38.33) → (−11.75, 6.47, −39.87) → (−15.26, 6.47,
  −41.29) → (−17.83, 6.47, −41.52)) and killed at dist 1.8 / 1.2 / 0.9.
- the rescue/death/AND chain asserts are untouched and pass: ch 0x49 (rescue [M
  DroneFunc_HostageSaved 0x1441c0]), the three captor deaths 0x29/0x2A/0x88 [D param 12],
  AND #397 → 0x3B, AND(0x49,0x4A) → 0x4B, AND(0x48,0x4B) → 0x4C [D tr[001]/tr[030]], then the
  Load Point #206 E → 07000003 through the real reload path [D param 0].

**No port gap was exposed**: the nav web carries the player into both captor rooms and the
shoot loop kills every guard first try — the guards were alive (health 20/10/10/20 read at
shot time in the logs) and died through the real damage path, no teleport needed.

## What's left

Nothing on the task's core. The full suite was still running at report time — its partial is
below; the next session may want the final total line.

## How it was verified

- The touched binary ran twice end to end (39.6 s and 39.3 s, both `test result: ok. 1 passed;
  0 failed`), full log captured (scripts/walk02_real.log + the run's stderr).
- The independent audit subagent checked: the diff touches only
  crates/nf-view/tests/walkreal_07000002_live.rs (+ the WORKLOG/docs); no credentials, no game
  bytes; docs/WORKLOG.md append-only; every ch assert unchanged from the parent commit; no
  `bond.pos` write reachable from the walk.
- **Suite** (the fast runner, started 15:12 UTC on this final tree — master + this change;
  the only code delta is the one test file): build ok — 936 test binaries in 1124 s; 436 of
  936 binaries done by 16:37 UTC with **0 unexpected failures**: FAILED
  probe_pylon_render::probe_render_pylon_only and
  fix_0c_pylon_live::the_0c_pylon_fills_the_view_from_the_closeup_spot /
  ::the_0c_pylon_is_on_screen_at_the_user_spot — the render-dependent pylon trio this GPU-less
  sandbox cannot satisfy (the branch changes no render code; the main agent's render check
  covers them) — and CRASH mp_stress_explosives_live (exit −9, the runner's wall-clock kill).
  **walkreal_07000002_live ran INSIDE the suite too: ok (39 s)** — the third green run. The
  remaining ~500 binaries will not finish inside this sandbox's window; the runner's log
  (../suite.log) carries each binary's line for the next session.

## Usage

not visible to me (this CLI session exposes no token/credit/context-window readout).
