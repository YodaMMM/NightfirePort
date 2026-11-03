# level-01-agent-hall-fight — finish the 01 Agent real-health walk

Branch from origin/s/level-01-agent-escort-fight (merge origin/master into it first) and read docs/reports/level-01-agent-escort-fight.md. That branch restores walkreal_07000001_live::walk_from_the_spawn_to_the_exit_with_the_contact_real_damage_agent and survives the contact escort's first three fights (all changes gated difficulty >= 2 in the level's own test file). The walk still dies in the hall fight: last death tick 3366, the east stage walk into #6; the door trade at 6.8 u is the closest loss.

Do: win the hall fight on real input the way a player would (the report's weapon/grenade lever, approach angle, kill order). No health top-ups, no forced guard state, shared helpers stay opt-in. Done = the Agent walk green, every other walkreal_07000001_live test still green, every walkreal_*_live green, then the full suite total line.

---
claimed-by: sandbox3 at 2026-11-03T13:30Z

---
## Report (blocked, 2026-11-03T14:43Z)

# level-01-agent-hall-fight — the hall fight is won by the door-line sound lure; the walk's last 5% (the stairwell bleed) remains

Branch `s/level-01-agent-hall-fight` (cut from `origin/s/level-01-agent-escort-fight`,
`origin/master` merged clean at 05075e4d). The escort branch's three fights were already green
(Bond reaches the meet scene at 76.5 [L the runs]); this session attacked the hall fight.

## Status: blocked (the walk is not green yet) — the mechanism is found, landed, and proven

`walkreal_07000001_live::walk_from_the_spawn_to_the_exit_with_the_contact_real_damage_agent`
still fails, but the failure moved from "dies at the hall entrance at tick 3366" to "dies at
1.6 health to the last route guards at the stairwell" — the hall fight itself is WON (the hall
AND, ch 0x99, fires at tick 4950 with the door phase costing 0 health in the best runs), the
escort runs the contact's FULL route to (−28.2,14.3), 5 u from the plate, and the pre-clears
kill the route guards 3/14/5. The remaining bleed: the door-phase settle gap (76.5 → 42.9 across
the show-then-shoot gap in the worst runs) and the route guards' attrition (12/17).

## The hall fight, as the player plays it (all changes gated `difficulty >= 2` in the level's own file)

The decode of the room (probe01_hall_research.rs, a scratch file, not committed):

- The meet hand-off gives Bond the SMG 0x12 at clip 32 spare 0 [L the probe dump]; the level's
  deepest pool is the level-start pistol 0x2 (clip 7 + the type-1 pool 47) [L the same dump].
- The two dormant SMG guards (10 at the meet plate, 16 south, both on switch ch 0x5) never fire
  on this route — ch 0x5 never rises [L every probe run; the escort report's own NF_CHWATCH
  note] — but they cannot be killed at 0.5-2.1 u: 32 SMG rounds with a settled aim registered
  ZERO health change while `last_shot` advanced (the rounds spawn) — the SMG's spread is tiny
  (16 + 5·bloom [L the record dump]) and its damage 1.5 [L]. **A player-bullet-vs-guard
  registration bug at close range; filed below for the main agent.** The walker kills them with
  the pistol at 3+ u (the initial clear's 3.5-dmg registrations [L the health trace]).
- The hall's ONLY nav entry from the meet room is the door at (7.9,11.1) (the "east"/"south"
  labels are the same aperture [D hall_ammo_ladder's own waypoint]). From the south the hunters'
  parts NEVER expose — plain AND scoped probes, door open or shut — while their volleys cross
  the plane, and every aperture crossing cost 30-70 health (the funnel concentrates both 8-pellet
  shotguns where Bond walks; the entry event alone took 62.5 → 9.3 once [L]).
- **The win is the sound lure**: Bond holds SOUTH of the pre-opened door (the hold line
  (7.2,8.8), 2.5-4 u, outside the 3.0 forced band [M DroneWeap_DoBulletAccuracy 0x179740:100-110])
  and fires into the hall; the guards hunt the gunshot hearing and cross the open aperture ONE
  AT A TIME, dying in the doorway (the probe run's corpses all sat at z 10.2-12.7 — all four
  named guards 5/6/7/11 AND the west-corridor pair 14/15; the AND fired with the door phase at
  **76.5 — zero damage** [L]). The hunters do cross: guard 11 was killed mid-crossing by the
  walk's own return fire in an earlier run [L].
- The fight-space: keep_off 3.2 for the door phase, not the escort's 6.5 — the wide space backs
  Bond out through the door where the south structure re-occludes every part (the probe: exposed
  false at 6.7 u with the door open) [L]. The dry ladder is gated to resupply-only in the door
  phase (its stage leg crosses the funnel under fire — one run died there [L]).
- After the AND: Bond steps just inside the doorway (2.6 u) — the contact's route runs back
  THROUGH the hall (his stall (5.9,14.1), route 18/37 [L]) — inside the lead's 5.0 release band
  [M NDrone2_DSTATE_AllyLeadWait 0x15eda8 case 3]. The direct steer crosses the already-open
  aperture; the nav's funnel hop wedges (the closes capped at 3000 ticks four runs running) —
  the walker nav wedge at this door is the level's known one, now confirmed from both sides.
- The route: the contact walks west through the west corridor where guards 12/17 (SMG) and 3
  (24 hp) sit on the ch-0x26/0xf waits — the route pre-clear (the player walked this corridor
  minutes ago) and the own-radius sweeps (the follow legs bled 76.5 → 42.9 crossing their
  volleys [L]) kill them as encountered.

## What ran (the final tree 4895e0a7)

- `cargo test --offline --release -p nf-view --test walkreal_07000001_live -- --test-threads=1`:
  **6 passed / 1 failed** — every Operative master green (the real-damage walk included), the
  plate/sisters green; only the Agent sister red at the stairwell bleed.
- `cargo test --offline --release -p nf-game --lib`: 656 passed / 0 failed / 5 ignored (the
  merged tree, this session).
- The fast runner (`tools/test/suite.py`, started in the background at 13:34 on the merged
  tree): 430 binaries reported when this report was written; its `walkreal_07000001_live`
  FAILED line predates the fix (the suite's build phase compiled the pre-fix binary). No TIMEOUTs;
  one environment CRASH (`walk_07000013_live (exit -9)`, the sandbox's SIGKILL, the same class as
  the previous session's mp CRASHes). The full total line did not land before the sandbox
  deadline; the per-binary totals the task names are the two quoted above.

## Remaining / open — the next session's move

1. **The door-phase settle gap**: the bounded 300-tick lure stops at the show; the crossing
   guard's volley lands in the shoot's settle gap (76.5 → 42.9 across the phase in the worst
   run). The 2400-tick tripwire variant killed Bond at the hold line instead (the exchange).
   The middle: keep the continuous stream ON the crossing guard (the aim tracks HIS live
   position, not a fixed point) — the probe's fixed-point stream worked as a tripwire; the
   live-tracking stream should kill each hunter inside his own volley budget.
2. **The route attrition**: 12/17's volleys during the follow legs. The pre-clear + the
   own-radius sweeps cut it to the 1.6-health miss; one more health lever (the door phase's
   fix above is worth ~30) closes it.
3. **The SMG close-range registration bug** (the port): 32 rounds at 0.5-2.1 u, aim settled,
   zero damage, spawns confirmed (`last_shot` advances) — player bullets vs guard boxes fail
   where the pistol (weapon 0x2) succeeds at 3.5-9 u. Candidate areas: the player-bullet's
   first-segment spawn origin for full-auto records (`fire_player`'s `shot_from`, scene.rs
   ~5150 — the rocket-origin midpoint path) or the drone hit-box test's near-field gate. With
   the SMG working, the hall fight and the route get a real magazine.

## Live checks for the main agent

- `gdbwatch` on the SMG's bullet spawn during a hip-fired burst at a guard 2 u away: confirm
  the game's BU spawn position/direction and the hit — the port spawns (last_shot advances) but
  never damages at 0.5-2.1 u while the pistol damages at 3.5+ u.
- `fieldprobe` poke: Bond 2 u from a guard, weapon 0x12, fire one round — expected: the guard's
  health drops 1.5×(the difficulty scale); observed in the port: no change.

## Usage

not visible to me — this environment shows no token, credit, or context-window figures for the
session (nothing to copy).
