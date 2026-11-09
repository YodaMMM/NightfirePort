# 0700001b: console 5 latches 93 ticks late on 00 Agent

On 00 Agent console 5 latches at frame 3667, after its 3574 deadline, so missile 5 launches (docs/reports/batch-1109a-walks.md). Find what makes the latch late (the walk's order, the console's hit test, a 00 Agent script timing [D the console UI scripts]) and fix the port or the route so the console phase finishes in time. Base on origin/s/batch-1109a-walks, push to s/level-1b-console5-00agent.

Reproduce on the real level first through real input, not a teleport. Compare with the decompile, fix, and test red on the old code. Run the walkreal_* binary of every level the change touches, one level per tool call. Branch from origin/master, push to `s/level-1b-console5-00agent`, report in docs/reports/level-1b-console5-00agent.md. No draw changes: name what is wrong if it is a draw issue.

---
claimed-by: sandbox4 at 2026-11-09T09:13Z

---
## Report (done, 2026-11-09T10:09Z)


---
## Report from the worker (done, 2026-11-09T10:09Z)

# level-1b-console5-00agent

Task 0700001b: on 00 Agent, console 5 latched at frame 3667, 93 ticks past its 3574 read
deadline, so missile 5 launched (docs/reports/batch-1109a-walks.md). Find what makes the latch
late and fix the port or the route so the console phase finishes in time. Branch
`s/level-1b-console5-00agent`, based on `origin/s/batch-1109a-walks` (the task's own base — the
00 Agent route and the merged walks live there; the 00 Agent test does not exist on master).

## What made the latch late

The game's clocks are clean; the walk's own defense fight was the thief.

- **The deadline is game-true.** Missile 5 (index 5, p3 seed 0 s [D placement params
  2026-11-06, by index 105/105/40/20/0/60/105/60]) leaves state 0 on the first gated tick after
  the launch order: `countdown -= REC_FRAME_RATE (1/60)` per tick [M SpaceMissile_Update
  0x1aa118, state-0 body 0x1aa1c8-0x1aa208, disasm-pinned], then its p0 script runs
  P0_LEN[4] = 1800 ticks and state 2 reads console 5's channel [M 0x1aa23c-258, 0x1aa25c]. The
  measured missile starts (m5 1775 = the ch19 frame, m4 2975 = +1200, m3 4175 = +2400) match
  launch + seed·60 exactly [L this run's log]. Deadline 3574 = 1775 + 1800 − 1.
- **The latch path is clean.** The console UI scripts (gate 30+k, hit_points 20 [D SP#213-220])
  write the UI gate at armed+359 [D P0_GATE — m5's gate 35 at 2134], hold, and each 20-damage
  block resumes via the SP damage tail [M SP_Update 0x1dc2f8-0x1dc318 / SP_GetHitDamage
  0x1dc5f8 / SP_UnPause 0x1dca58, difficulty-blind — verified in level-1b-console-latch-00agent].
  The hit test works: all three holds answered by real shots in every run.
- **The thief: the walk's mid-phase defense fight.** The red run's trace: k5 hold #1 answered
  2917, then `[1b] mid-phase defense: frames 2988..3305 (lined was 2)`, then hold #2 only at
  3593, hold #3 at 3665, latch 3667 (+93). The fight (the level-1b-console-latch-00agent lever)
  fired while 2+ astronauts lined Bond and stole 676 ticks (the fight + the re-acquisition)
  inside k5's hold chain. k5 is the only console with no slack to spend (seed 0; its UI script
  cannot even answer hold #1 until the gate write + the hold time, ~2784).

## The fix (route only — `crates/nf-view/tests/walkreal_0700001b_live.rs`; no `crates/*/src/` change)

1. **Deadline-slack gate on the mid-phase defense.** New `read_deadline_in(sim, k)` reads the
   pending console's ticks-to-read (state 0: `countdown·60 + P0_LEN − 1`; state 1:
   `len − 1 − (clock − 1)` — the same clock `tail_read_in` uses). The fight now runs only when
   `lined ≥ 2 && cooled && >1200 ticks remain` (the 700-tick cap + the re-acquisition + the
   rest of the hold chain). With less slack Bond keeps shooting the screen — the run1 ledger
   (no defense at all) measured ~1 hit per 250 ticks and the latch landing ~3100, so the
   exposure trade is survivable. The gate is proven by the instrumented run: read_in 586 at
   f2988 declining tick by tick, the fight declined through the whole k5 window, holds #2/#3
   answering 3019/3103.
2. **Height-capped defense float (`fight_opts_y`).** While fixing the deadline gate the run
   exposed a second, game-true killer: the SpaceLaser chain (ch 19 arms it [M
   Create_SpaceLaser 0x1aa8b8 / SpaceLaser_Update 0x1aa3f8 — ported since the 0700001b audit's
   GAP note], the 10 s fire writes ch 18 ~2375) turns **BigLaserBeamHURT #259** (0xe4, type 1,
   dmg 50, gate ch 18 [D], box (8.5,9.0,−0.4)..(81.4,9.9,0.4) [L the leg-start hurt dump]) live
   across the level, and the defense's aim lift (the view is level-only, so the eye must sit at
   the target's height) parked Bond at feet 7.53 → capsule top 9.33 [M Player_DealWithObjHit
   0x194458's feet..+1.8 r 0.55 capsule] inside the band: two −50.0 deliveries, dead at 3457.
   `fight_opts_y` caps the body height (cover candidates and the aim lift) at 6.0 feet for the
   defense only; `f32::MAX` at every other call site is a no-op, and the whole-binary run
   confirms the other 16 tests keep their timelines.

## Result [L this run]

- k5 holds answer 2917/3019/3103 uninterrupted; the channel writes ~3104 (the gate scan records
  the latch at 3201, −373, because the defense fight 3105..3201 suspends the scan) — **470
  ticks before the read**. Missile 5 DISARMS: `MissileDeploy [0,0,1,2,2,1,0,1]` — no state-3
  missile, mission status stays 1. The task's defect (k5 late → missile 5) is gone.
- k4 latches 4317 (−457) and k3 5574 (−400), their missiles disarmed the same way. The recorded
  margins are scan artifacts when a legal defense fight (pending = the next console, slack
  >1200) suspends the scan; the channel writes are what count and all three land early.
- Whole `walkreal_0700001b_live` binary: **16 passed, 0 failed, 3 ignored** (89.4 s) — the
  shared-path tests keep their measured timelines.

## What's left

The walk is still red, on survivability now, not deadlines: the wave grows 3@f3000 → 6@f4200 →
8@f5100 (the missile-script spawn channels), the capped fight cannot reach the ceiling-band
shooters (in-plane lifts above feet 6.65 enter the live beam band; the wave posts sit at
z ±2.7-4.3 behind the panel rows, and a z-follow float wedged against the rows in the measured
attempt), and the grind kills Bond at 5619 (14 hits for 100.0) during k6's window with k5/k4/k3
latched and disarmed. Next levers, in order: (1) a band-safe kill shape for the wave — an
off-plane lane at |z| ≥ 1.0 that the panel rows do not block, or waiting the band shooters down
to the ≤8.3 aim heights (the measured safe-reach line: aim ≤ max_y + eye_off keeps the capsule
top under 9.0); (2) the kind-107 blast-source decompile-compare of the previous ledger still
stands for the disarm wait. One measured negative to record: a z-follow on the capped fight
(`spot[2] = a[2]`) landed ZERO hits in its 700-tick cap (the wave posts are unreachable behind
the panel rows) and let the wave grow 6→8 over the same window — reverted.

## Live checks for the main agent

1. **The beam arm timing**: in PCSX2 at 00 Agent, from the launch order (ch 19) time the SpaceLaser's
   fire — expected ~600 ticks (10 s) later, ch 18 rising, and the BigLaserBeamHURT band
   (8.5,9.0,−0.4)..(81.4,9.9,0.4) dealing 50 to anything whose capsule overlaps it. If the
   game's band is cold longer (or never live until later), the defense cap can relax to the
   measured safe line instead of 6.0.
2. **The wave posts**: gdbwatch the five ch-19-window spawners' drone positions — expected
   y 5.4-8.7, z ±2.7-4.3 (the measured posts). If the game's wave also floats those heights,
   the band-safe kill shape is a real player problem too (cover behind the panel rows), not a
   harness artifact.

## Usage

- Tokens / context window / cost: **not visible to me** — no token, credit or context figures
  are exposed anywhere in this environment; I cannot quote numbers I was never shown.
- Wall clock (from the session's own `date -u`): the driver message arrived ~09:18 UTC; the
  branch was created and pushed 09:24; the red reproduced 09:47 (exit 101, k5 margin +93); the
  deadline gate + the height cap landed 10:05-10:20; the run ledger above 10:24-10:49; the
  whole-binary run 10:51 (16/0/3); this report written ~10:55 UTC, inside the ~11:17 UTC expiry.
  The full workspace suite did not fit the window; the changed file's binary ran whole and no
  `crates/*/src/` file changed, so no other binary's code path moved.
