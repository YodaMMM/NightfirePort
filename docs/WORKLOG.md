# WORKLOG

## level-04-interactables-2 (branch s/level-04-interactables-2)

- start (master ec9dfc3f, batch-1031j). Task: `interactables_07000004_live.rs` covers only the
  lamps (2 tests); using `interactables_0700000b_live.rs` as the model (one live test per KIND
  of player-touchable placement), run the census of 07000004 [D every placement kind], and for
  every kind a player can touch add the live test that drives it through the real path
  (walk/E/shoot) and asserts the game's result [M the class's Update/Activate; D its params],
  with a sister that fails on the wrong result; fix any port gap the tests expose, with
  evidence. Run interactables_07000004_live, walkreal_07000004_live and nf-game; quote totals.

- 19:05 UTC: census re-run over the disc parse [D: nf-formats Level::parse on 07000004.bin, main map
  "010000c5", 70 placements] — kinds 0x0F ×1, 0x19 ×40, 0x21 ×4, 0x24 ×1, 0x2A ×3, 0x2B ×2, 0x32 ×3,
  0x47 ×2, 0xD8 ×2, 0xD9 ×2, 0xDF ×1, 0xEE ×1, 0xF0 ×4, 0xF3 ×2, 0xFC ×2; no doors/switches/lifts/
  locks/monitors/fuseboxes/SimpleScripts — matches the 2026-10-17 table. Two unpinned pickup-kind
  behaviours added to interactables_07000004_live.rs: the state-2 respawn re-arm (60·2·10 = 1200
  ticks [M 0x18e148:182-189; D uzi param 5 = 2]) and the armour-cap refusal [M 0x18e650:220-225].

- 19:57 UTC: the armour-refusal test caught test-design exposure, not a port gap — the take log
  [NF_PICKUPLOG=1] shows Some(Armour): the assassination target (3.5 units from the post-settle
  stand) drops the poked +0x8B0 below 50 with his rounds during the walk, so the take legitimately
  grants [M 0x18e650:220 reads the live armour]. Fix: the kill through the real fire path first
  (the mission's own step), then the refusal holds. interactables_07000004_live: 4/4 green
  (2 lamps + uzi respawn re-arm + armour-cap refusal).
