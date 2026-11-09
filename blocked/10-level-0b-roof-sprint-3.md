# 0700000b: the roof-deck sprint to the parachute (third pass)

Continue on branch s/level-0b-roof-sprint-2 (a4e3541ae; read docs/reports/level-0b-roof-sprint-2.md): the crossing exists and the take fires; the failure is margin - Bond arrives with 5.2 hp because the dome-floor turret pair #174/#175 costs 19.2 and guard #4 parks on the sprint line at (2.6, 58.0, 32.7) costing 36.4 in the last 8 u. Finish its "What's left": a lane past the turret pair that costs less, and dropping or avoiding #4 before the final dash (as a player would), then make the Agent and 00 Agent real-damage walks pass, then the d1 walk and the whole walkreal_0700000b_live binary.

Push to `s/level-0b-roof-sprint-3` (from s/level-0b-roof-sprint-2), report in docs/reports/level-0b-roof-sprint-3.md. Never lower damage or health, no loosened pins, no new ignores. No draw changes.

---
claimed-by: sandbox1 at 2026-11-09T20:49Z

---
## Report (blocked, 2026-11-09T21:12Z)


---
## Report from the worker (blocked, 2026-11-09T21:12Z)

# level-0b-roof-sprint-3 — report

Branch `s/level-0b-roof-sprint-3` = `origin/s/level-0b-roof-sprint-2` (a4e3541ae) + this task's
commits. **Status: NOT finished — the Agent real-damage walk still dies on the deck leg.** The
stop line (22:31 UTC) arrived mid margin-tuning; the next window continues from this branch.
No damage/health lowered, no pins loosened, no new ignores, no draw changes, no crates/ changes.

## What's done

1. **The deck geometry is pinned [L the ray probe on this tree].** The NW quadrant is walled by
   a 45° diagonal face (normal (−0.71,−0.70), top 58.2 — above the standing capsule) running
   (−8.2, 23) → (−14, 28.8), plus a flat south wall at z 22.28 spanning x −7..−3 with a second
   piece at x 2. **The only north opening is the GAP x −2..1** (north rays clear, 57.0 floor at
   z 22-26; the floor grid also shows the 58.2 wall top, the 55.9 lower pockets behind it and
   the -- holes). Sprint-2's "structure x −11..−14, z 24-28" was the diagonal's NW half only.
2. **The chained deck lane is built and the take FIRES on the pass** (two runs):
   P1 the west band (−11.8, 18) — the measured 87.2-through-the-band hug; P2 the east run south
   of the z 22.28 wall to (−1.5, 20.3); P3 north through the gap; P4 the dash to the slot
   (2.23, 41.34). Phase advances on arrival (dxz < 1.6), never re-aims backward. The turn-
   tracking fire variant was measured and REJECTED: it yanked Bond off the lane twice (he drifted
   point-blank into #4 and died — the sprint-2 re-steer pathology), so the fire-on-move is
   opportunistic only (trigger when a target is near the crosshair, the turn never leaves the
   lane).
3. **The margin failure's root cause is found: Bond's gun is DRY on the deck.** The stall trace
   shows `gun w0x58 clip 0 spare 0` through the whole deck phase, while the wheel owns
   **0x43 (shotgun, clip 5 spare 10), 0x2 (clip 7), 0x1 (clip 1), 0x4e (500), 0x4a (clip 100)**
   [L the inventory trace]. Bond was running the deck holding an empty weapon — every shot this
   route ever needed was a blank. A second [L]: **0x4a (clip 100) spawned no rounds** through a
   converged, trigger-held stand — a non-bullet gadget; 0x43 (the shotgun) DOES fire through the
   same path (the clip ticked 5 → 4 on the on-heading #7 during P1).
4. **The fire plumbing is fixed:** `Walker::tick`'s second argument is the USE key, not the
   trigger (`tick_inner(input, use_key, fire, reload)`) — the sprint's fire was going into
   use_key. The sprint now taps `tick_fire` (the shoot_point discipline: fire only when the aim
   is converged) for both the opportunistic fire-on-move and the bounded stand-and-drop of #4 at
   the funnel (stop past z 29, turn, drop him, cap 240 ticks, then dash).
5. **The level's legal margin objects are identified [L the pickup probe]:** the ROCKET LAUNCHER
   #349 (8 rounds) at (−22.50, 47.96, 29.30) and the heavy armour #302 (50) at (17.53, 47.68,
   28.46) — both on the lower floor the return lane crosses, ~23-30 u off the proven n14→n16
   lane. One rocket at #4 from the gap mouth is 13 u with LOS (the z 22.28 wall is clear at
   x 0.6 [L]) — a one-shot answer to the 36.4 on-line pass.
6. Where the walk stands now: armed first (the real switch path to 0x43), the lane runs, the
   take fired at 14.4 hp in the best unarmed run — the arm-first stand at the re-place costs
   ~20 (87.2 → 67.6) and P2's east run bleeds ~25-30 (from #5/#7/#6/#9), so the drop at the
   funnel never completes inside the budget. The test `walk_fresh_spawn_to_the_mission_end_
   real_damage_agent` still dies in P2 (hp 3.2 at t15675 on the last run).

## How it was verified

Every number above is from the real walk through the real input path
(`walkreal_0700000b_live walk_fresh_spawn_to_the_mission_end_real_damage_agent --nocapture`,
the `[0b-stall]` per-30-ticks trace now with the gun/inventory/d4hp columns) or the fresh-load
ray/pickup probes (scratch examples, labelled as such, not committed). Each rerun is
deterministic on the fixed 60 Hz sim. The full suite did not run this window (the stop line
arrived first); the only changed files are the level's own test file and docs.

## What's left (the next window's plan)

1. **Kill the two standing costs.** The arm-first block currently STANDS at the re-place
   (~20 hp, 87.2 → 67.6 [L]) — a player switches while running: ride the switch during P1
   (`switch_to` while `forward` is on; Player_WeaponChange does not require a stand).
2. **Give the drop a one-shot answer.** The shotgun's 5+10 rounds may not finish #4 (20 hp)
   inside the 240-tick cap while four shooters work the funnel. The rocket launcher
   (−22.5, 47.96, 29.30) is the clean answer: walk it into the return lane (a real-input detour
   west of n16), `switch_to 0x2e`, one rocket at #4 from the gap mouth (13 u, LOS [L]), switch
   back, run the lane. The blast at 13 u does not touch Bond.
3. If the deck still bleeds: the heavy armour #302 (17.53, 47.68, 28.46) on the east side is
   the second object — it is a walk-over take on the real path, not a health write.
4. Then the Agent walk green → the 00 Agent sister → the d1 walk → the whole
   `walkreal_0700000b_live` binary → the full suite before any merge.
5. Watch: the fire-on-move gate (0.18) only ever engaged #7 (clip 5→4 [L]); #5 (err ~0.4 from
   the lane headings) never engages — if his P2 cost stays ~10+, widen the P2 lane south or
   drop him from the gap mouth with the shotgun before P3.

## Live checks for the main agent

None this window — every open question is answered from the port's own collision/inventory with
the probes. If the next window wants game confirmation of the pickup coordinates:
a PCSX2 walk from the terminal room to the west lower floor should show the rocket launcher at
(−22.5, ~48, 29.3) and the armour east at (17.5, ~47.7, 28.5).

## Usage

not visible to me — this environment shows no token, credit or context-window figures for the
session (no usage readout is exposed to the worker).
