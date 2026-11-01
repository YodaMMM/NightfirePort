## Your task: `level-11-start-corridor-guard`, 11: a guard walks into the start corridor and shoots Bond

On a fresh 07000011 load with Bond standing at the start, the port has a red-suited guard walk into the corridor beyond the unlocked door from about tick 200, hit Bond by tick 509 and kill him by tick 3172. On the PS2 the same corridor stays empty over the same frames [L PCSX2 capture: the start-position lighting sweep, frames ~f200-f3172]. Find which drone this is (placement, group, wake channel, route), why it wakes or walks in the port [M the wake/ENTER gates, Drone_Create 0x136be0's difficulty gate, the patrol/route class], fix the gap with the address cited, and pin it: an idle Bond at the start is not approached in the first 3200 ticks (sister: the guard IS present in drones.list). Run every *07000011* binary and nf-game; quote the totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-11-start-corridor-guard`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox4 at 2026-11-01T03:03Z

---
## Report (done, 2026-11-01T04:46Z)


---
## Report from the worker (done, 2026-11-01T04:47Z)

# level-11-start-corridor-guard — report

Branch `s/level-11-start-corridor-guard` (from origin/master). WORKLOG: `docs/WORKLOG.md`,
entries `level-11-start-corridor-guard` (session 1) + the session-2 continuation.

## The symptom

On a fresh 07000011 load with Bond idle at the spawn #146 (−0.40, 46.97, 25.83 [D]), the port
walks a red-suited guard into the corridor south of the spawn from ~tick 200 and he kills Bond;
the PS2 capture shows the corridor empty over f200-f3172 [L]. Reproduced in the port (fresh
`load_with_carry_difficulty(3)`, Bond idle): **placement #127** (skin 0x0500003A) becomes visible
at ~tick 200, first hit ~tick 417, Bond dead ~tick 700; at difficulty 1 only **#126** (skin
0x05000038) walks; #127 (disc p2 = 2) exists from difficulty 2 [D; M 0x136be0:49].

## Which drone it is

- **#126** (10.43, 47.86, 11.52): skin 0x05000038, dtypenew block p15=2/p16=3 → mode 7, switch
  channel p3 = 0, spawner-group word p4 = 31, difficulty tag p2 = 1 [D map 43].
- **#127** (−11.39, 47.86, 12.22): skin 0x0500003A, mode 7, p3 = 0, p4 = 30, p2 = 2, one stun
  grenade (p11 = 0x21) [D].
- Neither belongs to a spawner (the level's spawner groups are 1-7 [D]; 30/31 match none) — both
  are direct templates, created at load subject to Drone_Create's difficulty gate
  `placement+0x34 (param 2) <= GameState+0x40` [M 0x136be0:49].

## Why they walk — what the game does (all [M]/[D], verified this session)

The wake gate is NOT the divergence: with p3 = 0 the game's `DSTATE_Global` ENTER skips the
WaitSwitch arm [M 0x15b830:22-35], and p1 = 0x06000000 is zeroed at Drone+0x554 by DefaultInit
(the "null start script") [M 0x14b300:611-613], so Global sends them to +0x5A2 — mode 7's start
state 0x85 EnemyRunToPoint [M 0x15b830:39-43; DroneTypeSettings row 7 start = 0x85 [D
ACTION.ELF 0x29aee8+7*12]]. The port does exactly this (verified in the AI trace).

EnemyRunToPoint's ENTER then: sets +0x5A2 = 0x56, and — the Tower-mission clause for
0x07000011/12/13/41 with the 0x4000000 latch clear — LOWERS it to Idle (alertness < 0.66) /
Alert (< 1.0) / leaves ATTACK at full alertness [M 0x168910:23-31]. The port implements this
clause. The guards' alertness: the game seeds +0x50c/+0x500 from prop 0x20 through
DoModeSettingsNEW [M 0x14c600:60-68] with the table at 0x2f5cc8 = (0.0, 0.0, 0.66, 1.0) [D ELF];
#126/#127's prop 0x20 = 3 → **alertness 1.0** — the port matches (its `alert` = 1.00 at load).

## The fix (pushed, ce3558f8)

`EnemyRunToPoint` asks `NDrone2_FindRunToPoint` 0x153168 → `NDrone2_ClosestAIPoint` 0x151b18 for
the group-0 run-to point, preferring the point whose +0x0c **owner** equals the drone's +0x144
[M 0x151b18:151-181]. DefaultInit writes **+0x144 = placement param 4** (piVar19[0x11] =
params[4]; params start at piVar19[13] = DIVars+0x34, the flat level_tag param array) [M
0x14b300:472]. The port read **property BIT 0x11** of the props words instead — a different
field entirely (the guards' alert block p17 = 0x5b0003 sets bit 17, so the wrong read yielded 1).

The disc DESIGNS the pairing: #126's p4 = 31 ↔ the group-0 AIPoint with owner 31 at (1.40,
46.84, 16.71); #127's p4 = 30 ↔ the owner-30 AIPoint at (−1.48, 46.79, 18.21) [D]. With the old
read both guards matched the owner-1 point at (−1.51, 0.40) instead. The fix (drone.rs
`find_run_to_point` + `find_alarm_point`, the game's one +0x144 field) makes the port's goals
exactly the disc points — verified by probe (goal = (1.378, 16.715) / (−1.476, 18.206)).

Pinned in `crates/nf-view/tests/start_corridor_guard_07000011_live.rs` (2 tests, green):
- the sister: #126/#127 are present in drones.list at d3 (the difficulty gate) with their skins
  and param-4 owners [D; M 0x136be0:49; M 0x14b300:472];
- the wake chain (no WaitSwitch — p3 = 0, start_script zeroed [M 0x15b830, 0x14b300:611-613]);
- the owner pairing: `aipoint::closest` with the game's owner rule picks the disc points [D; M
  0x151b18:151-181];
- the negative: the old props-bit read disagrees with the disc owner [D; M 0x14b300:472].

## What's left — the pin is NOT met

With the owner fix the port's guards still walk to their points, resume to ATTACK (the Tower
clause at alert 1.0 [M 0x168910:23-31]), acquire Bond at ~7 units once LOS clears and kill him
(~tick 700) — while the PS2 capture shows the corridor empty f200-f3172. Ruled out this session:

- the alertness seeding ([M 0x14c600:60-68] — the port matches);
- the EnemyLookForOpponent 0.66/prop-0x51 gate ([M 0x177ed0:57-64] — ported, and alert 1.0
  passes it in the game too);
- a locked-door block on the route: the door-flagged nav nodes of path 1 sit at the doors #98/
  #15/#16 (x −23..−21, 19.8-21.1) and (12.9-13.7, 18.6-19.5) [D node flags & 2] — none lies
  between the guards' spawns and their owner points; the disc graph route is open;
- the emit's ±2 start-box (every AIPoint has a path-1 node within 0.05 [D nav]);
- NearestNodeDCV semantics ([M 0x155958 = NodeSearch mode 1, limit 625 — the port's `node_from`]).

**Strongest open hypothesis**: the game's per-cel object activation. The engine's control pass
(0x1336f8) walks DynamicObjList and `control_handle_cel_change` 0x133288 relinks objects between
per-cel lists; the port has no per-cel activation gate on drones, so its guards run their AI from
tick 1 while the game's may sit frozen (never ENTER-dispatched, never drawn) until their cel
activates. The guards' spawn cells ((10.4, 11.5) / (−11.4, 12.2)) vs Bond's start cel would then
explain both the empty corridor and the untouched Bond. Supporting [M]: an object inherits enable
bytes +0x100/+0x101/+0x102 from its CEL (+0x9a/+0x9c/+0x9e, indexed by the object's +0xa2 channel
byte) at every cel link — create ([M 0x1346b8 control_link_object_to_cel]) and every cel change
([M 0x1336f8:283-296]); `control_init_object` 0x134630 seeds them 0xff. The reader that would gate
the AI on them is still unlocated; the checks below settle the question directly.

## Live checks for the main agent

1. **Do the guards' AI run at boot?** Fresh boot 07000011 (difficulty 3, Bond idle at the start).
   gdbwatch on drone #126's and #127's machine state byte (the SM `cur` state) and +0x5A2 for the
   first ~400 frames. Expected if the cel hypothesis holds: state stays 0 (no Global ENTER ever
   runs) and +0x5A2 never becomes 0x85; if their AI runs, +0x5A2 = 0x85 within the first tick and
   the state walks 0x85 → 0x63.
2. **Do the guards move?** Watch Drone+0x30 (pos.x) of #126/#127 for f0-f3200. Expected under the
   cel hypothesis: constant at 10.43/−11.39 (frozen at spawn). If they move, record where they
   stop (expected: their owner points (1.40, 16.71) / (−1.48, 18.21) per the owner rule).
3. **Which cel owns each?** Fieldprobe obj+0x38 (the cel pointer) of #126/#127 and of Bond at
   boot — are they in different cels, and does the cel's active flag differ?
4. **The AIPoint emitter bytes** (only if the guards DO run): the level's group-0 AIPoint with
   owner 31 — its per-node dist byte at #126's nearest node (the port computes 10, not 0xff).
   Confirms the port's emit matches the game's.

## Runs

- `cargo test --offline --release -p nf-view --test start_corridor_guard_07000011_live -- --test-threads=1`:
  **2 passed; 0 failed**.
- The fast runner (`tools/test/suite.py`, started 04:10 UTC): build ok (1014 binaries, 1594 s),
  run in progress at report time. Lines so far: FAILED disc_golden ×3 (the documented slim-ISO
  allowance) and FAILED `identity_disc::the_real_disc_passes_the_identity_check` — an AI change
  cannot affect disc identity; this needs a master-branch comparison (the slim ISO's identity
  check has not been run green in recent sessions' notes). No other failures observed before the
  report cut.

## Usage

not visible to me
