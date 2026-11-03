# real-health-00agent-02-03-2 — report (2026-11-03)

**PART DONE.** The **03 d3 sister is GREEN and un-ignored** —
`walkreal_07000003_live::walk_the_full_route_and_complete_every_objective_real_damage_00agent`
runs in the suite and the binary is 7 passed / 0 failed. **The 02 d3 sister is parked red**
(`#[ignore]` stays): the geisha-1 rooms are fixed (60.0 health past them vs -04's 27.6) but the
courtyard kill zone still decides the run. Everything else in both binaries is green, and the
walkreal sweep ran green on every binary that fit the window.

Branch `s/real-health-00agent-02-03-2`, cut from `origin/s/real-health-00agent-02-03-04` with
origin/master merged (the -04 tip is an ancestor of master, so the merge landed on master
68298698; the two `#[ignore]`s from f3b24398 were the starting state). Only test files changed
between -04 and master — the sim is byte-identical, so -04's measured deaths reproduced
exactly (03 died at t25850 again, 02 at t6398 again — the first-run signatures).

## The 03 fix (green)

The west-lawn death: drone #8 (weapon 0x12) bled Bond out through FIVE 900-tick fights at
"killed false" while the walk stood at the west edge [L the -04 ledger; reproduced here]. The
NF_RF_TRACE run showed `aim Some, blocked true` on every sample — the stand COULD see a part
but held its fire. The blocker was never a live hostile (the `shoot_blockers` retarget never
fired) and never a servant (the servants stand at (30.8,−5.7)/(15.4,−27.0)/(32.9,1.6) [L the
walk's own identity snapshot] — all off that gun line): it was a **not-yet-enabled guard** on
the line. The game's own collide drops Enable(0) objects before the hit test [M Collide_PickObj
0x1eae80 — the +0xfd&1 arm returns the obj before the hit; NDrone2_DSTATE_WaitSwitch 0x15b960
enter → NDrone2_Enable(0) 0x149dd0; the pin the smg-close-range-hits report cites for the
dormant ch-0x5 guard taking nothing], and the bullets and the auto-aim skip it [M Check_AutoAim
0x1a73f8 skips +0xF4 == 1] — so a disabled drone never stops a round and the hold was not
game-shaped there.

- **`DamageLog::fire_through_disabled`** (new shared opt-in, set by the 03 d3 walk only): a
  disabled drone is dropped from the stand's blocker test. Default off = master behaviour.
  With it, the west-lawn fights resolved (drone #8 killed at t21426 at 43.6 health instead of
  bleeding Bond out at 8.2), and `shoot_blockers` (also set) answered the one live hostile
  that did cross a line ("fight#4 t53: line blocked by live hostile #3 — retarget"; the
  retarget reads team == 1, so the servants keep their hold and are never shot through).
- **In-file d3 resupplies** before the servants' rescue legs (`resupply`'s 9-u covered-path
  corridor — on the route): the first fixed run died at t42315 to shotgun captain #5 (weapon
  0x1c, 24.0 a hit) at 1.6 u with the P99 at clip 1 spare 0. The hook's resupply is the lane
  walks' arm, not this walk's.
- **`DamageLog::switch_dry`** (new shared opt-in, set by both d3 walks): the in-hand mag ran
  dry mid-fight and the master fight gives up while the wheel still carries rounds (the
  courtyard-lane run: the 0x12 at clip 0 vs the P99's 15 in reserve, the hp-5 rifle chaser
  finished a clip-0 Bond). The flag cycles the game's own weapon wheel mid-fight (one press +
  the draw wait). Default off = master behaviour.

Result: the walk runs to the exit — **t59079, health 28.8, exit loads 07000004** — and the
binary is 7/7 with the ignore removed.

## The 02 state (parked, the funnel works, the kill zone remains)

The -04 first-run death reproduced at t6398: the approach met the shotgun captain #7 (0x1c)
coming OUT of the geisha-1 room — the diag run shows the captors patrol out on their own
routes (state 0x67 at 11.5-14.7 u) — and the corridor meeting at 1.3 u took the opening
8-pellet volley at 57.6 [M DroneWeap_DoBulletAccuracy 0x179740:113-115: under 3.0 u every
round hits].

The committed **funnel defence** (d3-only, in the 02 file): hold the ch-0x17 plate, duel each
captor as he funnels out (`stand_and_fire` — Bond does not move; `hold_for_sight` then the
stand), scoop the drop and arm the dropped shotgun after each kill, then a 16-u
`clear_pursuers` pass for the corridor chaser that enabled mid-duel and chewed 43.2 into the
fight's back. The rooms now pass at **60.0 health (vs -04's best 27.6)**, drone #7's duel
costing 28.8.

What kills the run now is the **stair/lane kill zone**: the courtyard sniper (weapon 0x1e,
skin 0x5000036, 12 hp) covers the west-wing stair descent — his one hit landed **48.0** at
19.6-25.7 u in three separate runs — plus the 0x12 rifle guards' chips. Attempts measured and
in the WORKLOG: the stair-top duel (the sniper is NOT exposed from the stair top — the stand
never fired, clip 32 at the status line, hip or aimed), the lane-mouth duel, the 6.0 funnel
retreat (worse — died in the rooms), the stair-bottom/lane scoops + rearms (the rifle from
#8's drop killed the sniper in one run but the RNG reshuffles decide the sniper's shot). The
run is a coin flip at the crossing with 56-60 health against a 48.0 hit.

## Verification (all runs `--test-threads=1`, slim ISO)

- `walkreal_07000003_live`: **7 passed / 0 failed** (the d3 sister in-suite, 183.9 s).
- `walkreal_07000002_live`: **3 passed / 0 failed / 1 ignored** (the parked d3 sister).
- The walkreal sweep (the shared opt-ins default off, so these verify the shared file compiles
  and nothing moved): 01 **7/0**, 04 **10/0**, 05 **14/0**, 06 **6/0**, 07 **5/0**, 08
  **5/0**, 09 **7/0**, 0a **11/0** (3 pre-existing ignores), 0b **6/0**, 0c **2/0** (1
  pre-existing ignore), 14 **9/0**, 15 **6/0**, `npc_doors_live` **6/0** (1 pre-existing
  ignore), `fix_guard_misc_live` **6/0**.
- **Not run this window**: 0d, 11, 12, 13, 16, 1b, 41, 4a and the 0c-band/probe binaries —
  built clean (the whole nf-view test set compiles) but the sandbox window closed before
  their runs. The new flags are opt-in and none of those walks set them; their fight code
  paths are the master ones.
- `nf-game` / `nf-formats` / `nf-audio`: **untouched by this task** (the diff touches only
  `crates/nf-view/tests/` — common/damage_log.rs + the two level files — and the docs), so
  their last green suites stand; the full `nf-view` suite did not fit the window.

## What's left

1. The 02 d3 sister: the stair/lane kill zone needs either the sniper's exposure law (where
   the game lets you answer him from) or a survivable crossing (health > ~55 after the rooms,
   i.e. the funnel duel's 28.8 shrunk). The funnel defence, the scoops/rearms and the two
   shared opt-ins are committed and are the base to iterate from.
2. The walkreal binaries listed above that did not fit the window.

## Live checks for the main agent

1. The 02 courtyard sniper (the guard with weapon 0x1e at ≈(−52.0, 6.3, −14.9)): from the
   west-wing stair top (−36.7, 5.9, 1.0) a player cannot see him (the port's stand never
   fired from there — all 32 rounds still in the clip), but from the lane mouth (−43.2, 5.5,
   −1.8) the port's hook fight killed him. A gdbwatch on his Drone record during a real
   descent — when does his vision/attack state pick the descending Bond, and from which of
   the two vantages does the game let Bond's aim find a part? That settles whether the stair
   top is geometry-blind (his ledge hides him) or a port ray gap.
2. The 02 geisha-1 doorway (carried over from the -04 report, still open): the captors patrol
   OUT of the room on their own routes (state 0x67, the diag run) — a PCSX2 watch on their
   route/goal state during the ch-0x17 approach would confirm the game's own captains leave
   the room before Bond arrives (the port's funnel defence leans on it).

## Usage

not visible to me (no token/credit/context figures are exposed to this session; nothing in the
sandbox reports them).
