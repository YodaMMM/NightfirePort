# 0700000d: the Agent real-damage walk dies at the ladder top

The ignored Agent real-damage walk on 0700000d dies at tick 4275 at the ladder top; the no-wait #27 shot does not land on the guard that kills Bond (see docs/reports/batch-1109a-walks.md on s/batch-1109a-walks). Identify that guard, compare his sight/engage/accuracy on Agent with the game's, fix the port if it differs, otherwise take the ladder top as a player would (clear him from below first). Base on origin/s/batch-1109a-walks, push to s/level-0d-agent-ladder-top.

Reproduce on the real level first through real input, not a teleport. Compare with the decompile, fix, and test red on the old code. Run the walkreal_* binary of every level the change touches, one level per tool call. Branch from origin/master, push to `s/level-0d-agent-ladder-top`, report in docs/reports/level-0d-agent-ladder-top.md. No draw changes: name what is wrong if it is a draw issue.

---
claimed-by: sandbox3 at 2026-11-09T09:09Z

---
## Report (done, 2026-11-09T09:36Z)

# level-0d-agent-ladder-top — the killer identified (g33, the west-roof sniper) and audited: the port's guard law matches the game on every axis, so no port fix; the ladder-top tactic legs landed (the trio act after #17, the #17 re-sweep) and three measured laws recorded (the crouch boundary, the aim-mode view/cursor freeze candidate, the #17 wake lag); the walk stays red at the ladder top

## The task

The ignored Agent real-damage walk on 0700000d dies at tick 4275 at the ladder top; the
no-wait #27 shot does not land on the guard that kills Bond
(docs/reports/batch-1109a-walks.md). Identify that guard, compare his sight/engage/accuracy on
Agent with the game's, fix the port if it differs, otherwise take the ladder top as a player
would. Branch from origin/s/batch-1109a-walks (origin/master already contained). Real input
only; no teleports; no draw changes.

## The killer, identified

g33 — drones.list[33], skin 0x5000087, the west-roof sniper at (151.4, 22.7, −248.5), d 63.5
from the ladder top. Reproduced on the real level through the real input path
(`walk_arrival_to_the_c5_door_real_damage_agent`, the load → the walk → the climb → the ladder
top; no teleports): **death tick 4275, g33 dealt 94.0 in 4 hits (28+28+28+10) at the EXACT
180-tick weapon cadence (3735/3916/4095/4275)** — every shot landed. The batch's "#27 act
does not land" reproduced too: the act acquired g33 and never got the ticks (the 4th shot
killed ~95 ticks into it).

## The audit: the port does NOT differ from the game

Every axis of g33's behaviour checked against the decompile:

- **Accuracy** [M DroneWeap_DoBulletAccuracy 0x179740]: the port's `hit_chance`
  (crates/nf-game/src/drone.rs) is the decompile line-for-line — p = 100 − 5·(+0xB4); ×
  TooClose_Accuracy twice when close; × TargetFirstMoved; × the 0x0700000C/D anim-0x13
  stopped factor; × TargetMoving; the difficulty row on GameState+0x28 (1 → Easy, 2 →
  Normal [M :74-88]); `Rand_FRand(100) < p` — and a PASSING roll aims EXACT (the zero
  vector [M :116-121]). A sighted 0-accuracy sniper at d 63 on Agent with a still target
  rolls ~100% — 4/4 hits is the game's own output.
- **Sight/engage** [M NDrone2_DefaultInit 0x14b300:380-424, NDrone2_DSTATE_SniperFire
  0x15fd88:34-63]: the class-4 snipers carry sight 1000 and engage 75+frand(8) with NO
  distance gate in the mode-5 sniper states — the engage at d 63.5 is the game's law.
- **Cadence** [M DroneWeap_NextBulletTime 0x179d40]: the 180-tick spacing is the 0x1e
  weapon's own.
- **Damage** [M Player_HandlePain 0x1902c8]: 28.0 per hit = the raw × the Normal pain
  factor at Agent.
- **The bullet law** [M Bullet_init 0x12c5a0]: the game fires from the MUZZLE along the aim
  ray with the spread cone zeroed only by the 0x400000 aim bit — the muzzle-parallax dice
  Bond's own acts hit at d 54-63 is the game's own law, not a port deviation.

**Conclusion: no port fix on the guard side.** The tactic legs are the answer.

## What was done (all changes in `crates/nf-view/tests/walkreal_0700000d_live.rs`, the Agent block only — the plain and Operative timelines untouched)

1. **The west-roof trio act moved up** (the no-drain box act, cap 700, right after the #17
   leg): the base run's act only started at ~4180 and got ~95 ticks; the batch ledger shows
   84 of g33's 94 landed DURING the bay-duo legs. The v3/v4 runs: the act acquired g33 at
   (151.42, −248.65) and fired 5 on-target rounds at d 63 (the muzzle-parallax dice — none
   landed this roll), rode the reload, re-converged (+575, the cross ≈ 0, clip 5) — the
   sequencing fix is in; the dice is the game's.
2. **The #17 leg runs twice**: the v3 run's killing 10.0 at 4000 came from #17 — woken by the
   acts' own shots (his fire spot is d 39, INSIDE the P2K shot's hearing radius 40 [L]) AFTER
   his leg's pick ran pre-wake (the box caught the deck guard g20 instead). The second pass's
   live check measured the wake LAG: #17 is not yet present in the box when the check fires
   (the v4 run is byte-identical to v3 — the second pass never ran). Recorded; the next
   session needs a delay or a wider wake window.

## Three measured laws (the session's findings)

1. **The crouch-dodge does NOT generalize to the ladder top** (measured, the v2 run): with
   Bond crouched at the ladder top (pos y 16.95, the eye 17.2) g33 landed the SAME 4 × 28 at
   the same cadence. The rim-corner law's clip geometry is a SHALLOW line (the 0.45 target
   drop [M Player_GetHeadPos 0x195aa0:39-51] moves the clip point ~0.4 u there); the
   ladder→west-roof line is STEEP UP (5.7 u over 62 u) and the mid-band parapet sits at ray
   fraction ~0.97, where the 0.45 drop moves the line ~0.04 u — the head line stays clear and
   the snipers keep firing. The crouch was removed from the route.
2. **THE AIM-MODE VIEW/CURSOR FREEZE — the port deviation candidate** (measured 3-for-3):
   with the aim bit held, the stick moves NEITHER the view (the pitch −0.022 frozen for 400
   ticks under a −0.089 rad pitch error) NOR the cursor (pinned at the ±0.296 clamp edge —
   the [0d:dbg] cross [−0.296, 0.193] stable through a whole drained act); both only move
   after the aim bit DROPS (the clip-empty reload path), and every landed kill this session
   came after that drop + re-converge. scene.rs's free_aim block routes the stick into the
   cursor (`want[0] = cursor[0] − turn/0.76966` — a sign the convergence loop fights) and
   only the clamp overflow into the view, against Player_GetAimingPoint 0x1a4a98's pad law
   (the stick moves the cursor in the box and the view past the edge — a player tracks while
   aiming). **The next session's work: the 0x1a4a98 disassembly + the pad cursor update.**
   This affects every act-based test in every level if real.
3. **The #17 wake lag** (above).

## How it was verified

- The Agent sister run 4 times on the real input path (the base 4275 / v2 4169 / v3+v4 4000 —
  all at the ladder top), each with the NF_DMG_TRACE ledger; every change above cites its run.
- `walkreal_0700000d_live` whole on the final tree: **1 passed (the plain master replay), 0
  failed, 2 ignored** (`--test-threads=1`). The Agent block changes are gated
  `if difficulty >= 2` / `if real_damage` inside it — the plain and Operative timelines keep
  their measured legs byte-identical.
- The full four-crate suite was not started (the build phase exceeds the window on this box —
  the same measurement the batch and the operative-walk-3 reports recorded); the diff touches
  only the 0d test file (no `src/`, nothing under `tests/common/`), so no other binary's
  behaviour can change.

## What's left

1. The aim-mode view/cursor freeze audit (0x1a4a98 + the pad cursor update) — if real, the
   fix lands in `scene.rs`'s free_aim block and EVERY walkreal_*_live binary must re-run.
2. The #17 wake lag: delay the second #17 pass ~150-300 ticks (or widen the wake window) so
   the post-wake pass catches him.
3. The ladder-top endgame once the dice align: the trio act's re-converged shot (+575 state)
   lands on the next roll; the bay duo and the sweep follow.

## Live checks for the main agent

1. **The pad-aim cursor law.** In PCSX2 on 0700000d, aim the P2K at a far target and HOLD the
   stick left/right while the aim button is held: gdbwatch BL+0x118/+0x11C (the cursor) and
   BL+0x120/+0x124 (the smoothed copy) — expected: the cursor moves WITH the stick inside the
   box and the VIEW turns only past the ±0.296/±0.4 edges (Player_GetAimingPoint 0x1a4a98).
   If the game's cursor tracks the stick, the port's freeze (the stick moves nothing while
   the aim bit is held) is the deviation.
2. **The west-roof head line from the ladder top.** Segment-probe the eye (213.1, 18.1,
   −234.5) to g33's head box centre (151.4, 23.0, −248.5): expected CLEAR (the port's law);
   then the same from the CROUCHED eye (17.65): expected still clear (the v2 measurement) —
   confirming the crouch law's steep-up boundary.

## Usage

not visible to me — this environment shows no token, credit or context-window figures for
this session.
