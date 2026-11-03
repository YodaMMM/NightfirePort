# trigger-hittest-segment — port the game's player-vs-trigger contact so low laser beams trip standing

Read the `player_touchers` comment in crates/nf-game/src/drone.rs and docs/reports/level-0a-low-beam-standing-toucher.md on origin/s/level-0a-low-beam-standing-toucher. The game's shape is pinned [M]: the collide-pass HITTEST segment (Player_Collision 0x194df0, A = pos+up·0.275, B = pos+up·(0.55−cc), r 0.55, mode 0x800), Collide_PickObj 0x1eae80:96-119, then Collide_Intersect 0x1ebaa8:226-247 → Intersect_CylGeom 0x109da0 against the trigger's OWN glist triangles (obj+0xD8), credited by Trigger_Hit 0x1b1fc8.

The port still uses a measured box, so Bond walks through 0a's low beam #621 standing without tripping it (the game trips it [L]). Swapping in the bare segment broke three pins because the rest of the chain is missing. Port the whole chain:
1. The narrow phase against the trigger's own glist triangles, including the ref-gate/extent that lets 07000011 plate #235 and the 0c ch-27 bay volume fire (interactables_07000011_live::sound_triggers_created_and_played_per_the_disc_gate).
2. The ride's class-5 truck contact (trigger-port-check #11) so fix_truck_live stays right.
3. walkreal_0700000a_live::walk_office_floor_qworm_keyboard_hatch (+ real-damage sisters): if the walker's path brushes a low band, move the walk's waypoints off it like a player would; never weaken the trigger.

Done = low_beam_621_fires_standing un-ignored and green, those three binaries green, every trigger/beam/laser test and every walkreal_*_live green, then the full suite total line. List each binary you ran.

---
claimed-by: sandbox7 at 2026-11-03T23:53Z
