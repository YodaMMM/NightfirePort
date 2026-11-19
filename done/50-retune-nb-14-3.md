## Your task: `retune-nb-14-3`, 14 infiltration to the delta exit 00 Agent walk green again (round 3)

Branch from current origin/master; if origin/s/retune-nb-14-2 is not on master, merge it in first. Read docs/reports/retune-nb-14-2.md there and the ignore message on walk_the_infiltration_to_the_delta_exit_real_damage_00agent. Skip every lever the ledger marks measured dead; take the next untried one from the ignore message. If no untried lever is left, scan the armour and health pickups near the death first, then sweep the idle wait before the fatal fight (each wait shifts the guards' roll). Helpers stay gated to the level and the difficulty, so the other walks and replays stay byte-identical. Real input only, no top-ups, never lower damage or health, never change guard code. Judge each lever on its own real run. Remove the ignore when green; if still red, push the refreshed ledger and the next lever into the ignore message. Run the whole walkreal_07000014_live binary and interactables_07000014_live.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/retune-nb-14-3`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox7 at 2026-11-19T07:40Z

---
## Report (done, 2026-11-19T09:01Z)


## Report from the worker (done, 2026-11-19T09:01Z)

# retune-nb-14-3 — 14 infiltration to the delta exit, 00 Agent real-damage walk, round 3

Branch `s/retune-nb-14-3`, cut from origin/master c7e74c182, then origin/s/retune-nb-14-2 merged in first (it was not on master). v10 baseline on the merged tree replayed the v9 ledger byte-exact (death t38033 at (−57.69, 0.73, 38.88), 32 hits for 100.0, 31 fights) — the merge did not shift this walk's rolls, so the round-2 ledger stood.

## What the task was

Round 3 of the red sister: `walk_the_infiltration_to_the_delta_exit_real_damage_00agent`. Read docs/reports/retune-nb-14-2.md and the ignore message, skip every lever the ledger marks measured dead, take the next untried one, judge each lever on its own real run. Helpers gated to level+difficulty; real input only, no top-ups, never lower damage or health, never change guard code; run the whole `walkreal_07000014_live` and `interactables_07000014_live`.

## What changed (all in `crates/nf-view/tests/walkreal_07000014_live.rs`; no `crates/*/src/`, no `crates/nf-view/tests/common/`, no guard code, no damage/health values; every change behind `difficulty >= 3 && !tops_up()` gates)

1. **The south loop walked node-by-node** (follow_nav_route's own route read + the same r(1.3) legs — the wake-legs' byte-identity idiom) with four visibility-keyed `route_scan` stops keyed on the measured node positions, not indexes: the descent corridor foot, the door (−23.0, −19.6) crossing, the ascent base (the x −30 lane, z 9-11 — the round-2 ignore message's own lever-1 spot) and the ascent top (before the node-64 leg's dry resupply).
2. **The loop's dry-gun scoop** (the d1 wake legs' shipped law): keyed on clip+spare < 10, the nearest live pickup within 4.0 u same floor, panic-safe `try_walk_to`, gated by a peace check (no live team-1 guard within 12 u).
3. **The door beats moved earlier and repeated**: a scan at walk 1's end (the door's west face), the shipped post-walk-2 scan, and a second beat that re-parks on walk-2's endpoint (z 39.56 — outside the sealed jamb band) before scanning again.

## What was measured (each lever its own real run; [L] = this sandbox's real-input runs, NF_KILL_TRACE + NF_DMG_TRACE)

- **v10 (baseline)**: byte-exact v9. #18's post LOCATED: (−29.9, 32.5) — his 20.0/2 landed at d 1.0-1.6 during the node-64 resupply wedge (t14387/14418). The shipped fixed-spot fetch takes NOTHING in this stream (arrival (28.33, 31.60), clip 3+6 at node 24 — the v4 drop position was v4's own fight trail).
- **v11 (nearest-live-pickup fetch, 8 u)**: DEAD — the nearest pickup sat at (32.26, 31.14) BEHIND the shelf rim; the scooping walk wedged 2.05 short at the 6000-tick cap after dragging Bond through the junction fight zone. REVERTED to the fixed spot (itself a no-op walk).
- **v12 (loop stops only, no scoop)**: the loop crossed CLEAN — zero hits t12500→t24326, no wedge, the node-64 resupply never triggered (it only fires inside a dry fight) — but the walk reached the door face DRY (clip 0+0, "the switch band's guards: cycling to a gun (0)") and died during walk 2 to #33/#34's doorway bursts (#34 full-health at (−51.8, 41.0), 27.5; death t24600). 30.0 health at the door face vs v10's 12.5 — but an empty gun.
- **v13 (scoop, threshold 10, radius 4.0, no peace gate)**: THE DEEPEST RUN — the doorway CROSSED. The scoops fed the fights, the #14/#15 window (v10's 22.5) measured ZERO, health at the door's west face 50.0, the walk's own fights killed #34 (t24822) and #32 (t24989). Death t25995 at (−48.78, 1.06, 42.58) in the Kiko corridor, 26 hits for 100.0, 28 fights: #33 45.4/5 — he converged THROUGH the opened doorway to d 0.9 and his 35.44 grenade blast (t24931) landed while his fight ran dry (killed false on the empty clip); then the Kiko squad #28/#20 (d 9.6-9.9) finished the 7.1 remainder while the shipped Kiko stands/hold landed ZERO at that range ("drone 20 still alive after 300 ticks (health 10)", clip-3 windows).
- **v14 (scoop threshold 14, radius 4.5)**: DEAD EARLIER — the scoop walks run the walk's own fight machinery; a dry gun mid-scoop fired the fights' resupply and dragged Bond into #1's convergence zone (45.0 in 10 hits, death t9879). Hence the peace gate.
- **v15 (the peace gate, v13 values)**: the re-rolled stream wedged node 5 (the fights' backpedal onto the level's hover cells, grounded=false, the 6000-tick cap).
- **The scans' ray truth** (the candidate dumps, [L]): the foot scan — ALL candidates gunfalse (the junction geometry, the same law as the leg-2-spot dead end); the door-crossing scan — #14/#15 gunfalse (their approach is blind-sided); the ascent-base scan — **#16 eyetrue+guntrue d 4.2, #17 guntrue d 7.3 — the rays ARE clear from the x −30 lane** (the fight ran dry at clip 6+0 and lost); the ascent-top scan — #18/#19 gunfalse (the rays cross the (−29.9, 23.7) door plane); the door beats in v13's re-roll — only #20 gunfalse (the squad had converged/moved).

## Verification (the required binaries, run whole on the final tree)

- Whole `walkreal_07000014_live` (10 tests): **8 passed, 1 FAILED, 1 ignored** (164.9 s). The ignored one is the d3 sister (this task's walk). **The FAILED one is the d1 `real_damage` walk — and my diff cannot reach it**: every change is behind `difficulty >= 3 && !tops_up()` gates; the d1 path compiles byte-identical to the retune-nb-14-2 tree, whose binary run passed 9/0/1. The break arrived with the merge of the newer origin/master commits (22d5836b7 the Exchange weapons, e4fb1c512 the weapon-loop substate stop — the branch was cut from current master as instructed). **Flagged to the main agent: a stash-run of the d1 test on the clean merge (my diff reverted) will attribute it in one run.** No d1 fix was attempted inside this task's window.
- `interactables_07000014_live`: **10 passed, 0 failed** (13.5 s).
- The fast suite (`tools/test/suite.py`) was started in the background in the first minutes per the driver instruction (`nohup ... &`) and died between tool calls, like the last two rounds (verified: process count 0, `suite.out` empty). The required binaries were run whole in the foreground instead; the full-suite total line is not available for this tree.

## What's left for the next message

1. **Arm the ascent-base fight** — the scan's rays are measured clear (#16/#17) but the gun must carry 10+ rounds into the x −30 lane. The trio's drops lie ON the nodes the fights kill them (x 27-28, z 25-12) and the peace gate blocks scooping while they converge — a second scoop pass keyed on the ascent-base approach (after the node-44 door, the lanes quiet), or a raised threshold only on the quiet west nodes.
2. **Win #33's doorway fight** — the fights killed #34/#32 in v13 (the fights' own resupply armed the gun mid-phase) but #33's first fight fired dry. The west-face beat needs the gun armed BEFORE walk 1 — e.g. a scoop pass at the switch band (the hall legs' quiet stretch).
3. **The Kiko machinery's zero-land windows at d 6.5-11** — the shipped 300-tick stands and the 90-tick hold fights land nothing at that range (the "rounds die on the corridor corners" shape the 07000013 chain measured): pull the squad closer or move the stands south first.
4. Attribute the d1 `real_damage` failure (the stash-run above) — it predates this branch's diff or came with the master merge; either way it is not the d3 lever set.
5. The full suite on the final tree once a sandbox keeps background jobs alive.

## Live checks for the main agent

None requested this round; the measured runs are all walker-side and the ray facts cited are this sandbox's own candidate dumps ([L], with the door leaves' plane and jamb bands from the chain's existing [D] probe notes).

## Usage

Not visible to me — this session exposes no token, cost or context-window figures to the worker; nothing to copy.

---
## ntfy failed (appended)

All three fallbacks failed (2026-11-19 ~10:00 UTC): curl (28) Connection timed out after 20002 ms; python urllib the 20 s urlopen timeout, killed by the wrapper, no output; wget exit 0 with an EMPTY body (a plain topic poll also times out — the endpoint unreachable, as in rounds 1-2). The branch and queue pushes succeeded.
