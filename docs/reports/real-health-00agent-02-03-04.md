# real-health-00agent-02-03-04 — report (2026-11-03)

**PART DONE.** The three 00 Agent (difficulty 3) sisters of the 02/03/04 Agent real-health walks
exist on `s/real-health-00agent-02-03-04`: **04's is green** (the level passes as-loaded, like
07 in the 05-07 task), the restored Agent (d2) sisters and every Operative master are green, and
**02's and 03's d3 sisters are red with the death mechanics measured to the hit** — the next
session continues them from the branch.

Branch `s/real-health-00agent-02-03-04`, cut from origin/master 01917076 (the previous session),
then merged origin/master 408b5012 (which brings the merged 00agent-05-07 work;
`s/real-health-agent-difficulty-01-07` was already an ancestor). master's 2879088f "Revert the
Agent-difficulty walks" had removed the 02/03/04 Agent sisters; they are restored **byte-exact**
from e356a0ca — the three files differ from that commit only by the revert (verified by diff
before restoring) — and the three `_00agent` sisters were added on top.

## The difficulty-3 rows the game reads (checked, already in the port)

- Bond's pain on the 01-04 group: Plr_DMod_Hard = 1.2 (0x3f99999a; Operative 0.5, Agent 0.7) [M
  Player_HandlePain 0x1902c8; tuning.rs ← ReadTuningVars 0x1d3a80].
- The guards' hit chance: DroneFiring_Accuracy_Hard = 1.0 (0x3f800000; Easy 0.5, Normal 0.7) and
  their rounds DroneDamage_Hard = 0.8 [M DroneWeap_DoBulletAccuracy 0x179740:74-88; tuning.rs].
- The autoaim bit closes at 00 Agent: Autoaim_HardMul = 0 [M tuning.rs:36].
- The min-difficulty placements pass Drone_Create's gate at d3 [M 0x136be0:49]; the 02 captain
  criterion reads prop 8 at difficulty 3 (prop 5 at Agent) [M 0x14b300:504-535].
- Every `_00agent` test asserts `Drones::difficulty == 3` at load [M GameState+0x28 →
  Drones::difficulty] and rides the front end's launch path [M C_SBNFDFCTY 0x216ca8:15 →
  load_with_carry_difficulty]. Real input only — the load path is the only difference.

## What passed (all runs `--test-threads=1`, slim ISO)

- `walkreal_07000004_live`: **10 passed / 0 failed** — the Operative masters, the platinum/stats
  tests, the restored Agent sister, and **the 00 Agent sister green as-loaded** (the mission
  route's own cover holds at d3; no tactics needed, like 07).
- `walkreal_07000003_live`: **6 passed / 1 failed** — the masters, the Agent sister (d2, byte
  -exact), the key-gate and servant tests green; the d3 sister red (below).
- `walkreal_07000002_live`: **3 passed / 1 failed** — the real-health master (d1), the exit
  walk, the Agent sister (d2, byte-exact) green; the d3 sister red (below).
- nf-game + nf-formats + nf-audio (`--test-threads=1`): **136 passed, 3 failed** — exactly the 3
  allowed slim-ISO `disc_golden` (bigf header/tables/directory).

## The 02 d3 sister (red, measured to the hit)

The geisha-1 room is a point-blank ambush: the captor approach met shotgun drone #7 at 1.3 u
(weapon 0x1c), its first volley landed 8 pellets × 7.2 = 57.6 at 2.7 u and the follow-ups
finished Bond at tick 6398 (100.0 in 56 hits) [L the run ledgers]. The kill trace showed the
fight's only two taps hit part 4 (an arm, 2.8 each) — the doorway angle never exposes a head
part — and 23 rounds burned to 5.6 damage. At d2 the same room costs ~33.6 a volley at 0.7
accuracy and the sister passes.

Three tactic iterations (all gated `difficulty >= 3` inside the 02 file; the d1/d2 runs keep the
exact restored RNG): the guards' dropped shotguns as the resupply (`scoop_drops`, `arm_weapon
0x1c`, the per-round captor scoops, `take_room`'s d3 arm), a per-take arm (the captors close
during the scoop walk), and an `arm_fullest` verify re-cycle (the draw ate a press and armed the
dry 0x47 cuffs with a 32-round 0x12 on the wheel — Bond then took the corridor's rifle chips on
an empty hand). The best run got past both geisha rooms to the bunker door at 27.6 health, then
died to rifle chips from drones #8/#10 (weapon 0x12, 13.4/26.1 u, −1.8 each) whose return-fire
fights never opened. The pre-room scoop walk is retired (it passed the captors' posts and cost
61.2 before the room); the open question for the next session is the round-0 fight itself — the
shotgun must be in hand BEFORE the 1.3-u doorway fight, and the only drops lie behind the
captor's fire.

## The 03 d3 sister (red, measured to the hit)

Died at tick 25850 on the west lawn: drone #8 (weapon 0x12) sat at **1 hp** through FOUR
return-fire fights ("killed false" ×4, 900-tick caps each, no flank prints — the fights held
blocked, likely an ally on the line: the 03 servants stand among the guards) while its −1.8
chips bled Bond from 31.6 to 0.0; the walk then wedged on the west-lane nav (STUCK d 1.77) [L
the run ledger]. The mid-courtyard and west-lawn fights before it killed 4 guards for 68.4.
Next session: unblock the fight (move off the ally line, or re-pick), then re-run.

## Rule 7 (shared helpers)

**No shared-helper changes**: the diff touches only the three level test files and the docs.
The other levels' walkreal binaries are untouched code (their last green sweep stands from the
merged batch); the 02/03/04 binaries were rebuilt and run above.

## What's left

1. The 02 d3 sister: get a shotgun (or a survivable first volley) before the geisha-1 round-0
   fight; then re-measure the bunker-door rifles.
2. The 03 d3 sister: unblock the 1-hp-guard fight; re-run.
3. `tools/test/suite.py` was not run this session (the 2-hour window went to the d3 fight
   forensics; the sandbox background jobs die between tool calls, so the fast runner cannot
   survive the report-writing gap). The per-binary runs above are the evidence; the next
   session should run the suite on the final tree.

## Live checks for the main agent

1. The 02 geisha-1 doorway: fresh boot at 00 Agent, walk the south hall to the ch-0x17 plate —
   the captor guard beside the door opens with an 8-pellet volley at ~2.7 u. gdbwatch on
   DroneWeap_DoBulletAccuracy (0x179740) would pin whether the game's d3 hit chance at under
   3.0 u is really 1.0 for all 8 pellets ([M 0x179740:113-115] says the under-3.0-u rule).
2. The 03 west-lawn ally block: whether the game's own guards shoot THROUGH a standing servant
   on the line at 00 Agent (the port's fights hold fire — the shoot_blockers law). A PCSX2
   watch on NDrone2_DealWithObjHit's team gate during the west lawn would settle it.

## Usage

not visible to me (no token/credit/context figures are exposed to this session; nothing in the
sandbox reports them).
