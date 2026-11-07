# level-14-door-pair — report

Branch `s/level-14-door-pair` (from origin/master 22d5836b7). Task: both 07000014 door guards
reach the locked door as they do in the game — the ignored
`the_door_guards_hold_combat_no_route_like_the_game` names the gap: only #783 routes to the door,
the released group-9 pair #778/#780 wake into no-route combat at their own spawns.

## The finding: the [L] door pair is the mission-walker pair, and the group-9 pair's wake is game-faithful

**The port's group-9 chain is faithful — the divergence the task framed does not exist in the
game's own chain.** Proven end to end:

1. The pair spawn at resting alertness **1.0**: prop 0x20 reads word 1 bits 0-1 ([M] bitDescs
   0x2c7250, prop 0x20 = word 1, shift 0, mask 0x3 — dumped from `work/elf/ACTION.ELF`), and
   #778/#780's placement parameter 19 is 0xee738fe7 → 3 → the static alertness table
   {0, 0, 0.66, 1.0} [M 0x2f5cc8, read from the ELF] gives +0x500/+0x50c = 1.0 [M
   NDrone2_DoModeSettingsNEW 0x14c600:61-66]. #783's parameter 19 is 0xee738fe6 → 2 → 0.66.
   [D docs/levels/07000014-audit.md :81/:83/:85]
2. `GetDroneTypeAttackTypeFriend`'s fold `1.0 <= +0x50c` maps start state 4 → ATTACK 0x56 [M
   0x14cbd8:122-155], so the game's own wake sends #778/#780 to 0x56 → COMBAT 0x58 → 0x5a [M
   0x148338 case 0]. The port does exactly this (probe trace: 0x00 → 0x56 → 0x58 → 0x5a).
3. Their combat route to Bond (held in the generator hall, west) cannot be built: the goal-node
   set is stamped around Bond's feet [M NDrone2_NavNodeCache 0x155cd8 → AINetwork_BuildAITarget
   0x15ac78 → AINetwork_NodesForPosition 0x1586a8 → NodeSearchCel 0x156e18:59-62 — radius
   default 15.0, so every candidate is hall-side, x −61.8…−91.8], and the only link west is the
   locked door pair's link, which the A* rejects [M AINetwork_DoAStarPath 0x1571b0:262-271]. A*
   exhausts → 6 → `DroneFunc_CombatState` case 2 default → COMBAT_NO_ROUTE 0x5f at the spawn [M
   0x148338 case 2]. The port's 0x5a → 0x5f within one dispatch (probe-verified) is the game's
   chain too. The shuffle (0x73/0x74 picks) is the known NB_SCAN_LIVE hold, not this task's.
4. The walkers: #783 AND #784 spawn at alertness 0.66 (parameter 19 = 0xee738fe6 both), take
   start 4, and IDLE's prop-0x1d arm (prop 0x1d = word 0 bit 0x20000000 [M bitDescs]; set in both
   words 0xf1dec9c1/0xf1dec8e1) assigns the kind-8 MissionPath and SetStates 0x1b EnemyMission
   [M 0x15bf18:32-56], whose walk ends at the east door node (−51.4, 39.6) [D node dump: path 0
   node 124, flags 0x2]. **At Operative #784 does not exist** (param 2 = 0x2 > 1 [M
   Drone_Create 0x136be0; D audit :85-87]) — the port's single walker is CORRECT at that
   difficulty. At Agent (the game's boot default +0x28 = 2 [M bootup 0x1c7e78:25 @0x1c7f88])
   #784 exists, and the port reproduces the [L] end state: #783 holds ≈ (−50.6, 39.5) vs [L]
   (−51.2, 38.8), #784 ≈ (−51.15, 40.73) vs [L] (−51.6, 41.2) — two guards at the [L] door
   positions, ~1.3–2.4 apart depending on the step-aim shuffle phase.

## What landed

- `crates/nf-view/tests/door_guards_locked_door_07000014_live.rs`:
  - NEW `at_agent_the_two_mission_walkers_hold_at_the_locked_door_like_the_game` (passing,
    game-evidence-cited): at Agent both #783 and #784 spawn, walk >10 u on the kind-8 assign
    [M 0x15bf18:32-56], hold within 3.0 of the door node (−51.4, 39.6) [D], and never enter
    OPEN_DOOR 0x81 [M 0x1571b0:262-271]. This is the [L] end state pinned on the port.
  - The ignored hold test's note REWRITTEN with the difficulty finding (the assert untouched and
    still game-true). The test stays `#[ignore]`d: at Operative the game's own chain cannot
    produce a two-guard 0x5f door hold (only #783 walks), so the 0x5f-at-every-sample assert as
    written can only pass with NB_SCAN_LIVE live AND the right difficulty framing — see the live
    checks.
- No `crates/*/src` changes at all — the divergence the task described is not a port bug.

## How it was verified

- `door_guards_locked_door_07000014_live`: **5 passed, 0 failed, 1 ignored** (16.2 s, release,
  NF_ISO set) — the ignored one is the hold pin.
- `walkreal_07000014_live`: **9 passed, 0 failed, 1 ignored** (146.2 s).
- `interactables_07000014_live`: **10 passed, 0 failed** (11.5 s).
- `ai_traces_live` + `npc_doors_live`: **6 passed, 0 failed, 1 ignored** (25.5 s).
- `nf-game --lib`: **686 passed, 0 failed, 5 ignored** (0.9 s).
- `walkreal_07000001_live` NOT re-run: the diff touches one test file only (no `src` change), so
  it cannot regress; the previous session's 5/0-with-flip evidence stands.
- Full suite (`tools/test/suite.py`): started detached twice in the first minutes; the sandbox
  killed the background session both times (suite.out empty, no total line). The touched surface
  is one test file; the binaries quoted above cover it.

## What's left

1. The live checks below (the [L] boot's difficulty and the door guards' placement ids).
2. If the [L] run was Agent: the hold test should be re-framed at Agent (with NB_SCAN_LIVE) and
   un-ignored there; if it was Operative, the second door guard's identity needs the live read
   (the decompile chain gives the game no Operative second walker).
3. The NB_SCAN_LIVE flip (not this task's goal; the 014-walk re-pin blocker stands).

## Live checks for the main agent

1. **The [L] boot's difficulty.** On the 2026-10-30 fresh boot that measured the door hold:
   read `GameState+0x28` (the difficulty) — expect 2 if the boot used the game's default. This
   decides between "the [L] pair = #783+#784 at Agent" (the port already matches) and a real
   second-walker gap at Operative.
2. **The door guards' identities.** During the hold window, dump the two door guards'
   `Drone+0xE0` (skin) or their placement indices, at the hold positions (−51.6,41.2) /
   (−51.2,38.8). Expect #783 + #784 (skins per the audit rows' param 0: 0x5000090/0x5000087) if
   the boot was Agent.
3. **The group-9 pair's states in the game.** For the two guards at the group-9 spawns
   (−30.58,13.36)/(−23.52,36.03) after ch 0x87+0x88 with Bond in the hall: state CUR/PREV over
   5 s. The decompile chain predicts 0x56 → 0x58 → 0x5a → 0x5f at the spawn (no route, the door
   link locked). If the game instead shows them at 0x5a walking west, the divergence is in the
   A* goal-test model (the game pops ANY node stamped with the target's found/fallback flags [M
   0x1571b0 disasm 0x1571cc/0x157294/0x1573f8; the port requires the ONE found/fallback node,
   crates/nf-game/src/nav.rs `astar`]) — that model difference is real but its behavioural
   consequence on THIS chain could not be proven from the decompile alone.

## ntfy failed

All three fallbacks were tried in order and all failed:

1. `curl -sS -m 20 -H "Title: NightfirePort" -d "level-14-door-pair: done — …" https://ntfy.sh/nightfire-yoda-8k3x`
   → `curl: (28) Connection timed out after 20001 milliseconds`
2. `python3 -c "import urllib.request as u; u.urlopen(u.Request('https://ntfy.sh/nightfire-yoda-8k3x', data=b'level-14-door-pair: done …', headers={'Title':'NightfirePort'}), timeout=20)"`
   → `urllib.error.URLError: <urlopen error [Errno 101] Network is unreachable>` (the first attempt with a
   non-ASCII body failed with `SyntaxError: bytes can only contain ASCII literal characters`; the retry
   with an ASCII body produced the URLError above)
3. `wget -q -O- --post-data "level-14-door-pair: done" https://ntfy.sh/nightfire-yoda-8k3x`
   → no output within 30 s; the command was killed by the tool timeout (`Command timed out after 30.0
   seconds and was killed`).

## Usage

not visible to me (this environment shows no token, credit, context-window or cost figures for
the session; nothing to copy).
