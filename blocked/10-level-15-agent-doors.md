# level-15-agent-doors — the 15 Agent walk with the guards using their door routes

On master (after level-14-cascade-servers-4) guards take their door routes, and walkreal_07000015_live::walk_spawn_to_the_omega_sector_exit_real_damage_agent is ignored: Bond takes a -41.6 world hit (live wire / blast / hurt volume) at frame 5400, -22.5 from #11 (skin 0x50000b2) at 6191, and dies to #5 (skin 0x500008c) at 7885 near (-0.02, 41.32, 46.87). The Operative walk stays green.

Find what the -41.6 world hit is (which blast or volume, and why the door change moved Bond or a guard into it) and re-route the walk the way a player would. Real input only; no top-ups. Remove the ignore when green; run every walkreal_*_live; full suite total line.

---
claimed-by: sandbox3 at 2026-11-03T22:28Z

---
## Report (blocked, 2026-11-03T22:59Z)


---
## Report from the worker (blocked, 2026-11-03T22:59Z)

# level-15-agent-doors — the 15 Agent walk with the guards using their door routes

Branch `s/level-15-agent-doors`, cut from origin/master 38052bf6 (2026-11-03 22:35 UTC).
**Status: not finished — the Agent walk is still red.** The driver's first ask (identify the
-41.6 world hit) is done and evidenced; the re-route is part-way (two verified improvements
landed, ~30 health still to find). The next session continues from this branch.

## What the -41.6 world hit is (the driver's question — answered)

It is **mine #526's own blast** — the walk's own rim shot on the door[29] ankle beam, not a
door-route effect:

- The hurt law is pure distance: `Explode_Propagate`'s sweep computes
  `damage * (1 − (int)dist / radius)` with the (int) truncation toward zero, no LOS and no wall
  check [M crates/nf-game/src/explode.rs `propagate`; disasm 0x17c61c-0x17c628; site 0x17c518
  case 3 → `Player_Hurt`; the port pushes the hit with owner usize::MAX → the log's "world"].
- #526's record: damage 80, radius 15 [L ee_start_05.bin the mine records; the walk's own
  #487-chain comment pins radius 15].
- Bond's pain on Agent (difficulty 2) scales by Plr_DMod_Normal 0.6 [M Player_HandlePain
  0x1902c8]. At the measured boom distance 2.44 u: 80·(1−2/15)·0.6 = **41.6** — the exact hit.
  The Operative walk (×0.4) pays the same blast as −27.7 and stays green: the blast was always
  there; the door change did not move Bond or a guard into it.
- Second confirmation of the law: the −6.4 world hit at tick 1224 is #462's own blast — the
  walk's lane shot from 13.57 u: 80·(1−13/15)·0.6 = 6.4, exact [the NF_DMG_TRACE run quoted in
  WORKLOG].

## The measured death ledger (the ignored Agent test, master route)

`walkreal_07000015_live::walk_spawn_to_the_omega_sector_exit_real_damage_agent` reproduced the
driver's death exactly (tick 7885 at (−0.02, 41.32, 46.87)):

| frames | damage | source |
|---|---|---|
| 611 | −2.1 | #15 (0x500008c) — the gantry-east grenadier's round |
| 1224 | −6.4 | #462's own blast (the lane shot from 13.57 u) |
| 3282-3445 | −5.4 | #17 (0x5000087) — the plank burst-rifle guard, 3 rounds |
| 4315 | −0.0 | a far blast |
| 4446/4515 | −4.2 | #2 (0x500008c) — the desk-corner grenadier, 2 rounds |
| 5400 | **−41.6** | #526's own blast (the rim shot from 2.44 u) |
| 6191 | −22.5 | #11's (0x500008c) grenade — in flight when the fight killed it |
| 7885 | −17.8 | #5's (0x500008c) grenade — same, at door[50] |

Sum: exactly 100.0. The door routes' true delta is the chip guards (#15/#17/#2, −11.7
together); the killers are the two in-flight grenades and the walk's own mine blasts.

## What landed on the branch (commit 243a6aa8)

1. **The great-hall grenade guard (placement 733) is answered from the door[50] threshold,
   before the corridor's last legs** (was: after route_d, a no-op). VERIFIED: the guard dies
   before its throw — #5's grenade never flew; the −17.8 is gone from the run.
2. **The west-room grenadiers (#10/#11) are answered from the rim mark, before the door[29]
   doorway is walked** (was: from inside the west room, after #11 had already thrown). PARTLY
   effective: #11 now costs 4.2 (was 22.5); #10's fight still drags (21.0 in 5 GL rounds — the
   first rim-mark fight gives up on its cap, the door[52]-leg return fire re-fights it at 7.7 u
   and eats the spray).
3. The #526 comment block now carries the full falloff law and the probe evidence.
4. `walkreal_probe15e.rs` — the shot-line probe (assertion-free, `--ignored`): the bullet trace
   to #526 from every stand x ≥ 14 (east AND west of the rim, direct and off-axis, raw and
   bullet-family) dies on the rim's face; only x ≤ ~13 clears. **−41.6 is the geometric floor
   for the #526 shot** — "the level's own toll", already paid on the Operative walk.

Tried and REVERTED (the evidence is in the WORKLOG): the #462 shot from 16.2 u east (zeroed
the −6.4 but the ~100-tick route shift reshuffled the door-route fights — the east-plank
grenadier's drag became −21.0, net worse), and the #526 shot from west of door[32] through the
walked-through doorway (the rim blocks the line; bullets die at the rim face).

## Current state of the Agent run (v7, NF_DMG_TRACE)

Died at tick 7502 at (0.27, 41.41, 26.36) — past the #447 fence shot (9.50 u, cause 2 ✓),
during the door[42]-through leg. Ledger: #526 41.6 + #462 6.4 + #17 5.4 + #2 4.2 + #15 2.1 +
#11 4.2 + #10 21.0 + #4 1.8 + #19 4.2 + the #447-fence chain pops 9.1 (−4.8/−4.3 at 7501/7502,
owner world — the fence's own blasts at 9.5 u; the radius is evidently NOT the #383 fence's 8,
the pops' law is not yet pinned) = 100.0. ~30 health to find.

## What's left (the next session, from this branch)

1. **Kill #10 in the first rim-mark fight** (hp 20 — one head hit; the fight gave up on its
   cap instead). Options: raise that fight's cap, or pre-shoot #10 by proximity rather than the
   placement spot (the door routes move the guards off their spots; `agent_drone_near`
   tolerance 8.0 missed the moment).
2. **Pin the #447-fence pop law** (the −4.8/−4.3 at 9.5 u don't fit radius 8/damage 100
   truncation — check the #447 record's params [L ee_start_05.bin @0x2bf150 + the #447 entry])
   and move the fence shot out of its chain's reach if the geometry allows.
3. Optionally re-try the #462 east mark AFTER 1-2 land (its −6.4 saving may then outweigh the
   fight reshuffle).
4. Remove the `#[ignore]`, run EVERY `walkreal_*_live` binary, then the full suite total line.

## Verification done on this branch

- The ignored Agent test: RED (documented above) — the repro runs are deterministic per route.
- `walk_spawn_to_the_omega_sector_exit_real_damage` (the Operative real-damage sister):
  **re-run GREEN** on the branch route.
- NOT yet run on this branch: the rest of the walkreal_07000015_live binary (the top-up walk,
  the mine sister tests), every other walkreal_*_live, the full suite. The next session must
  run them before reporting done (merge lesson 10).

## Live checks for the main agent

None needed so far — the port's blast law matched the game's addresses and the measured runs
at three distances (2.44 u, 13.57 u, 9.50 u). If the fence-pop law stays unpinned, a live check
would be: stand Bond ~9.5 u from the #447 fence on the real game, shoot it, and record Bond's
health drop (expected −4.8/−4.3 if the port's law is right; the record params would settle the
radius).

## Usage

not visible to me (this environment shows no token, cost or context-window figures).
