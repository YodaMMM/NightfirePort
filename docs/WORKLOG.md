
## trigger-glist-extents
Started 2026-11-04 16:31 UTC. Branch s/trigger-glist-extents cut from origin/master
b21edd082. Task: parse trigger cel extents the way the game places them (find the game's
extent read + ref gate [M], re-pin 0a #614, 07000011 #235, 0c ch-27 bay [D]); then make the
standing toucher's deep reach unconditional and retire the pos+1.26 box top if the evidence
allows. Evidence base: docs/reports/level-05-truck-class5-contact.md and the player_touchers
notes in crates/nf-game/src/drone.rs (~:745).
- (16:31) branch created, WORKLOG started, suite runner started in background.
- (18:1x) the parse pinned faithful ([L] ee_start_0700000a obj+0x90 = the placement for every static trigger; the only deltas = the key sweeps), the #614 "under the floor" premise disproven ([D] floor y ≈ 0), the game discriminator = flags bit 0x40 [M 0x1ebaa8:93-99] (the census probe), the deep-reach/box levers implemented+measured: green 6 binaries, red 3 levels (the route-timing family) — REVERTED as not-yet-allowed, the retune named in the report; the two probes committed.
