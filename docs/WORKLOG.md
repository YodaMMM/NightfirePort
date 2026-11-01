
## level-11-start-corridor-guard — 2026-11-01

Task: on a fresh 07000011 load with Bond standing at the start, the port has a red-suited guard
walk into the corridor beyond the unlocked door from about tick 200, hit Bond by tick 509 and kill
him by tick 3172. On the PS2 the same corridor stays empty over the same frames [L PCSX2 capture].
Find which drone this is (placement, group, wake channel, route), why it wakes or walks in the port
[M the wake/ENTER gates, Drone_Create 0x136be0's difficulty gate, the patrol/route class], fix the
gap with the address cited, and pin it: an idle Bond at the start is not approached in the first
3200 ticks (sister: the guard IS present in drones.list). Run every *07000011* binary and nf-game;
quote the totals.

Session 2 (2026-11-01, continued): sandbox reset, setup redone. Progress:
- Reproduced the symptom: on a fresh d3 load p127 (skin 0x0500003A, swch 0, mode 7) walks to Bond,
  drawn at ~tick 200, first hit ~417, dead ~700 (d1: p126 only; p127 exists from d2 — [D p2]).
- Ruled out the wake gate: [M 0x15b830:22-43] swch 0 → not WaitSwitch; [M 0x14b300:611-613] p1
  0x06000000 zeroes +0x554 — the game activates them into 0x85 exactly like the port.
- THE FIX (pushed ce3558f8): the ClosestAIPoint owner is Drone+0x144 = placement param 4 [M
  0x14b300:472]; the port read props bit 0x11. The disc pairs p126 (p4=31) ↔ AIPoint owner 31 at
  (1.40, 16.71) and p127 (p4=30) ↔ owner 30 at (−1.48, 18.21) [D]; after the fix the port's goals
  are exactly those points.
- New [M]: the game seeds +0x50c/+0x500 from prop 0x20 via DoModeSettingsNEW 0x14c600:60-68 (the
  table [D ACTION.ELF 0x2f5cc8] = 0, 0, 0.66, 1.0) — the port matches (p126/p127 alert 1.0);
  the Tower resume at alert 1.0 is ATTACK [M 0x168910:23-31] — the port matches; EnemyLookForOpponent's
  0.66/prop-0x51 gate is ported [M 0x177ed0:57-64]. No locked-door node pair sits between the
  guards' spawns and their points [D nav node flags] — the route is open on the disc graph.
- Remaining gap: with all of the above the port's guards still reach combat at ~7 units and kill
  Bond; the PS2 capture shows the corridor empty f200-f3172. Strongest open hypothesis: the game's
  per-cel object activation (control_handle_cel_change 0x133288 / the DynamicObjList membership)
  holds these guards' control until their cel activates — needs live evidence. Live checks listed
  in the report.

## level-11-start-corridor-guard-2 — 2026-11-01

Task (continuing s/level-11-start-corridor-guard, merged origin/master): the owner-read fix
(Drone+0x144 = placement param 4 [M DefaultInit 0x14b300:472]) is in but the pin is not met — on a
fresh 07000011 load with Bond idle at spawn #146, #127 (d>=2) / #126 still walk into the corridor,
resume ATTACK at alertness 1.0 (the Tower clause [M 0x168910:23-31]) and kill Bond by ~tick 700,
while the PS2 corridor stays empty over f200-f3172 [L]. Find the remaining divergence (the run-to
point's own state/latch, the 0x4000000 latch, LOS from those points, the difficulty the capture ran
at / GameState+0x40 assumptions), fix it with the address cited, replace the branch's "EXPERIMENT"
comments with plain cited comments, and pin: idle Bond at the start is not approached in 3200 ticks
(sister: both guards present). Run every *07000011* binary and nf-game; quote the totals.

Session 1 (2026-11-01): sandbox reset — full setup redone; branch created from
origin/s/level-11-start-corridor-guard + merge of origin/master (2249e9f5). WORKLOG started.

Session 3 (2026-11-01, the -2 task, branch s/level-11-start-corridor-guard-2 from the -1 branch +
merge of origin/master): the four suggested leads resolved with [M]/[D] — no port fix was needed,
the divergence question narrows to two [L]-only facts:
- Prop 0x56 is CLEAR on both guards: the guards' p20 = 0x40280000 (bit 26 = 0x04000000 clear) and
  p25 = 0x40291660 (clear) [D 07000011 map 43 params]; prop 0x56 = behaviour word 2 bit 26
  [M 0x1c7d98 + bitDescs 0x2c7250[172/173] = (2|26<<3, mask 26) / bitMasks[26] = 0x4000000 [D
  ACTION.ELF]]. So EnemyRunToPoint's RunToAlarm redirect [M 0x168910:17-27] cannot fire — the
  prior session's mode-7 analysis stands; a probe run confirmed the port's prop read and words.
- AIPoint+4, the ClosestAIPoint eligibility byte [M 0x151b18:128], is written 1 for EVERY point
  at creation [M 0x137550 Drone_AIPoint] — the points never block the selection at boot.
- The difficulty gate is `placement p2 <= GameState+0x40` [M 0x136be0:49] and +0x40 is the
  difficulty (the 0..5 bands in HitDamage 0x145558:135-144, DefaultInit :505/:514/:526). At d0
  NEITHER guard is created (p2 = 1, 2 > 0) — the game's own empty corridor.
- The 0x4000000 latch is clear at boot (only the HandleImpact sighting paths set it [M 0x1469b8]);
  the Tower clause at alertness 1.0 leaves the resume at ATTACK [M 0x168910:23-31] — the port
  matches.
- The port probe (scratch, removed): at d3 both guards walk mode 7 → 0x85 → 0x63 to (0.43, 17.65)
  / (−2.14, 18.86) by t240 and hold attack states there — the data-faithful walk.
- Pin reworked in start_corridor_guard_07000011_live.rs (5 tests): the d0 empty corridor (no guard
  satisfies the gate; idle Bond unapproached over 3200 ticks), the d3 owner-point walk + ATTACK
  resume, prop-0x56 clear + no kind-8 AIPoint [D the 12 0xE9 placements], and the -1 task's two.
  The f200-f3172 capture's difficulty and the per-cel enable gate (+0x100/+0x101/+0x102 from the
  cel's +0x9a/+0x9c/+0x9e [M 0x1336f8:292-296, 0x1346b8]; the AI reader still unlocated) remain
  the live checks.
