# level-11-start-fight-real-health-2 — report (2026-11-02)

**Status: BLOCKED by the session window; the branch is at its best verified state and the last
blocker is fully characterized with measured alternatives.** The prior session's fix (the
flight-B blocker stand, `DamageLog::shoot_blockers`, opt-in) still stands; this session branched
from `origin/s/level-11-start-fight-real-health`, merged origin/master, reproduced the remaining
death exactly, and measured every tactic the prior report named — all die en route to the fix.
The walk stays `#[ignore]`d with the full measured story in its reason string.

## The remaining blocker, measured again (the repro)

`walkreal_07000011_live::walk_spawn_to_the_lower_floor_load_point_real_damage` dies at **tick
2789 at (19.86, 1.13, −10.87) — 07000012's south corridor** — drone #10 (weapon 0x1c,
stationary at (19.5, −11.3)) lands 8 pellets × 6.0 TooClose in one tick at 0.86 u [L the run's
death log; L weapon_data +0x18 = 8 pellets; M 0x178f68:233-241 the per-shot loop; M
0x139f50:24-26 the ×2 under 3.0 u]. Bond is DRY (the d3 front-end load's own 7 rounds were spent
on the spawn-lobby fights [L the report of level-11-start-fight-real-health]). The damage chain
is the game's (the prior session's audit stands: nothing in the per-shot chain is a port bug).

## The tactics this session tried (all measured, all reverted)

1. **The corridor swing to z −13.8** (the pre-batch line, the report's idea 2): the swing's
   straight legs cut the corner to 0.6-2.2 u of the post and died at the same tick 2789. The
   funnel geometry caps EVERY path at ~2 u: the proven bend 2 (20.75, −9.75) is already 2.0 u
   from the post, the east side is walled (the walk pinned at (21.26, −9.09) against the bulge),
   and the corridor itself runs 0.7-0.9 u past the post's z. No crossing path holds the ~2.5 u
   the spread cone needs [L repro5-8, four waypoint shapes, all dead at 2789].
2. **The BodyArmour #734 detour** (the report's idea 1, as opt-in tower2 legs
   `(14,6) → (17.5, 3.2) → (17.48, 1.51 [D p2 0x64 = 100])` with rejoin): died 1.8 u short of
   the plate twice, for two different reasons:
   - with the walker's fights on: the detour legs re-shuffle the spawn-lobby fights (#4
     "killed false"), a DRY return-fire fight vs the east volleyer #5 stood 900 ticks in his
     field and cost 82.5 health [L repro3/reproA];
   - with fights suppressed in the detour zone (an opt-in `no_fights_near` zone, walked not
     fought): a single **−72.0** blast landed at 12.2 u with **no `Grenade` in flight anywhere
     in the walk** [L repro9 — the NF_DBGLOB probe printed zero lines] — the blast rides the
     bullet/prop explosion path, not the port's grenade list, so a walk-level grenade jink
     (also tried, opt-in) never fired. The lobby guards carry p11 0x41 [D] (1 frag each) — the
     throw itself may be the game's, but the port's blast attribution needs a look.
3. All common/ churn (the detour legs, the two opt-in flags, the walk-level jink, the probe)
   was **reverted** — the branch carries no unverified helper changes. The test file sets no
   flags; `walkreal_07000012_live` and every other walkreal binary are untouched.

## What was run (the chunked suite, the 10-min tool cap)

- `walkreal_07000011_live`: 1 passed (the topped-up walk), 1 ignored (the real-damage walk).
- `start_corridor_guard_07000011_live`: **8/8**.
- **EVERY `walkreal_*_live` binary** (rule 7): run in batches — all ok except
  `walkreal_07000014_live` and `walkreal_07000015_live`, both SIGKILLed at ~3.5 G free even at
  `-j1` — the sandbox's known memory kill (documented in the earlier reports' `exit -9` lines),
  not a test failure.
- nf-game + nf-formats + nf-audio: 136 passed, 3 failed — exactly the 3 allowed `disc_golden`
  (bigf header/tables/directory tree).
- nf-view `fix_*` set (141 binaries): 418 passed, 2 failed — both the
  `fix_0c_pylon_live` pair, panicking at `wgpu request_adapter` — the sandbox has no GPU (the
  same class as `probe_render_pylon_only`, below); visual work is the main agent's.
- nf-view non-live set (208 binaries): 330 passed, 1 failed (`probe_render_pylon_only` — GPU).
- nf-view `_live` set: 90 of 478 binaries run (212 tests, all green) before the window closed;
  the remaining ~388 did not fit.

## What's left (the next session's first items, in order)

1. **The Deagle #777 detour** (the report's idea 3 revisited): #777 (1.74, 0.24, 13.99 [D],
   7 rounds) — if the office-north detour is fight-shaped safely, Bond kills #10 from the bend
   (5.9 u) instead of crossing the funnel; the audit said "reachable only AFTER the corridor" —
   verify against the route first.
2. **The −72.0 blast attribution**: find the port path that deals a lobby-guard-attributed 72.0
   with an empty `grenades` list (the prop/barrel explosion chain or a bullet-path explosive);
   cite it and check it against the game [M the blast push sites in drone.rs].
3. **A live check for the main agent**: in the real game (07000012, hard), walk Bond dry through
   the south corridor at z −12 — does guard #10's loop really land 8×6.0 at 0.86 u, and does the
   corridor's real geometry allow a ≥2.5 u crossing? gdbwatch `Drone_ModBulletDamage`
   (0x139f50) returns during the crossing.
4. Un-ignore the walk once a tactic survives; run walkreal_07000011_live,
   start_corridor_guard_07000011_live, every walkreal_*_live, then the full suite.

## Usage

not visible to me (no token/credit/context figures are exposed to this session).

## ntfy failed

(previous session's three fallbacks all failed; this session's attempts are logged below if
they fail too.)
