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
