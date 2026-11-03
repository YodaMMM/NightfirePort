# WORKLOG (branch running copy)

## level-06-agent-guardroom-descent

Branch `s/level-06-agent-guardroom-descent` created from `origin/master` (554952ac). The
difficulty branch s/real-health-agent-difficulty-01-07 (737515ff) is already an ancestor of
master, so no merge was needed.

Task: walkreal_07000006_live `walk_the_castle_interior_to_the_party_real_damage_agent` wedges,
not dies — Bond floats on the guardroom's upper threshold at (28.55, ~41.5, -19.16) with
floor_below 40.0 and grounded false, and the door #59 legs hit the 6000-tick cap. At Agent the
extra spawns move the fight and Bond lands on the upper level, where the Operative run never
goes. Find what holds him up there [D coll-scan of the threshold, M the player's floor and fall
path, FeetOnPoint 0x194188]; if the game would drop him, fix the port. Then find the ramp or
step a player walks down [D], add the descent leg for the sister, make it pass. Where Bond dies
or wedges, read the damage log against the per-difficulty pipeline. Real input only.

Progress:
- [08:49 UTC] Setup done (repo, ISO, work/ all present from the prior session); branch pushed;
  full suite started in the background (suite.out).

- [09:10 UTC] The sister restored from e356a0ca and the wedge root-caused by coll-scan (scratch
  probe, deleted): the guardroom is ONE room at floor 40.0 (the floor grid x 26..32, z −24..−16
  reads 40.0 throughout) — there is NO upper level and nothing standable at the wedge spot. The
  "floating" stuck samples (41.56/41.83 ungrounded) are the walker's own stuck-ladder jump
  pulses [M walker.rs stuck 3/4/5]; the grounded sample reads 41.13 = the 40.0 floor + 1.1. The
  real blocker: the west wall's pilaster (east face x 28.61 over z −19.71..−21.36, top ≈42.2
  [D tris_near + the floor grid's 42.2 hits at x 28.5]) corner-catches the direct southwest
  steer at corner + radius = (28.55, −19.16), d 1.95, through the 6000-tick cap; at Agent the
  extra spawn drone #3 fights from (29.9, 41.0, −19.5), 1.4 from the catch point, and delivers
  Bond into it. Fix: Agent-only legs (difficulty >= 2, Operative byte-identical) round the
  pilaster through the east lane — (29.7, 40.6, −20.9) then (29.4, 40.6, −22.3). Second find:
  the ch-55 alarm assert was a phase/path coincidence — head6 (5.50, 55.67, −50.15) reaches
  Bond's sphere only where its pitch edge meets body height (z −38..−35 × x 2.4..8.6; the
  Operative alarm measured (4.55, 42.18, −36.30) exactly there); the Agent descent's west lane
  misses the band by ~0.6 u and the game's own sweep misses it too. Alarm not a fail channel
  [L record 1] — the pin stays Operative-only, the sister pins the lit heads + the route.
  walkreal_07000006_live: ALL 6 green (36 s), the Agent sister completes in 15216 ticks at
  46.4 health, 21 hits for 53.6 — the per-difficulty pipeline reads clean (Agent hits >= the
  Operative hits for the same drones, Normal 0.7 vs Easy 0.6 [M Player_HandlePain 0x1902c8]).
