# agent-glass-2: the impact at a glass sheet

Branch `s/agent-glass-2`, built on `s/glass-pierce-2`, which carries `s/glass-pierce` (from master
10956badb). It squashes all three; their reports are folded in below.
**This branch moves the shared random stream**: the sheet's `Effect_Create` and ricochet roll draw
as any other surface's do. Merge it with the other stream-shifting branches and re-pin once.

## What a player gets

A round through an intact sheet now shows and sounds at the sheet: the glass impact sound, the
material's emitter, debris, gas and bullet hole, and the round flies on. A pane that dies to the
round loses the hole with it.

## The game [M]

`Bullet_CollisionHandler` 0x12ce80, raw words 0x12cfb8-0x12d020:

- The pass flag is `bVar7` ($s4): for records with +0x6c bit 0x800 clear, surface byte & 0x3f == 0x10.
  No random draw on that arm.
- 0x12cfc0 is `bnel`: its slot's `sh $v0,0xf4($s3)` runs only when the branch is taken, the pass.
  0x12cff0 is plain `bne`: its slot always runs. State 2 is stored on both arms; the loop's end
  writes state 1 back on a pass and walks to the next contact.
- A pass skips only the damage arm 0x12cff8-0x12d01c. Both arms reach the tail at 0x12d020, so
  `Effect_Create` runs on a pass contact. The explode arms are gated on no pass; the ricochet roll
  draws at mask 0 [0x12d6f8].
- The pane's health is not the handler's: `Collide_Update` mirrors the contact onto the object and
  `Break_Update` 0x12bfd8:36-37 drains it. The port's breaks arm already does that.

## The port

- The breaks arm queues the contact in `pending_sheet_impacts`. `flush_world_impacts`, the collide
  slot every other player impact uses, replays it ahead of that round's stop: `sheet_impact` plays
  the surface's impact sound and calls `bullet_impact_on` at ricochet mask 0.
- `Breaks::sheet_contact` finds the point, normal and surface byte on the pane's own collision
  triangles, under the pane's matrix.
- The hole is hosted on the pane (`PANE_PART`). `follow_hosts` reads a pane's matrix from the
  breaks list and frees its holes when the pane dies, as the game's go with the object
  [M 0x1ce8a8:170-186].
- The `PlayerImpact::World` entry stays, moved to the same point, so the first impact is the sheet.
- The doubled `#[test]` on `the_08_station_shot_crosses_to_the_ambush_guard` is gone: the binary
  lists 3 ignored again. The delay-slot comment is corrected.

## Carried from glass-pierce-2

- Guard, turret and space-laser rounds read the glass pass from the record (+0x6c bit 0x800 clear,
  `WeaponData::glass_pass`) at all five spawn sites [M 0x12f030-0x12f058, 0x12cf44-0x12cf64]; the
  melee records 99-102 carry the bit set and still stop [L weapon_data].
- The pass gate is the contact's material 0x10, not placement kind 2 [M 0x12cf54-0x12cf60]: 01's
  breach panes (material 0x02) now stop the round, and `the_01_breach_shot_breaks_the_entry_glass`
  is re-pinned as the stop.
- The 08 real-damage walks were repaired for the armed pass on the walk side only (the Hind pane
  shoot-out on every real-damage run, the d2 back-off, the d2/d3 east boarding legs, opt-in glass
  sight for the walker's rays, a retry for a pane that survives, the flight pin on path time).

## Still open

1. Two 08 walks are red on glass-pierce-2 alone, and fail identically with this branch on top
   (same tick, same message): `walk_the_whole_mission_on_real_input` (the Hind takes the 9th
   rocket, past the ledger's 8) and `..._real_damage_00agent` (Bond dies at tick 28105 to a
   -62.5 blast in the Hind fight). glass-pierce-2 also left 01's 00agent walk red (7/1).
2. Guard rounds through a 0x10 sheet do not drain it; the breakable boxes are only read in the
   player walk. In the game a guard round kills a health-0 sheet.
3. Non-glass breakables stop the round in the breaks arm with no `Effect_Create`; the game runs it
   on every contact. Not touched here.
4. The pane collision ride and the guard LOS masks: untouched; dead_pane's movement test stays ignored.

## Material census [D]

`examples/breakable_census.rs` prints each level's kind-2 pane tri materials. 08, 12, 0a and 06 are
0x10; 01's breach panes 0x02; 09's 108-123 0x0f.

## Unproven

- The look: no render was made (draw work is not this lane's). It needs a firing look at an 08
  sheet against the game: hole position and size on the glass, the emitter and debris.
- The order of the sheet's draws against the pane kill's draws: the port kills the pane in the
  update pass, the game in the next frame's `Break_Update`. Not traced live.
- `sheet_contact` uses the pane's shape under its matrix; that pairing is called unproven for the
  collision ride. On 08's front pane it lands on the sheet plane (z -121.988, normal +z) [L the
  test run]; other levels were not checked.
- The box fallback (no triangle met) faces the shot and is [S].

## Proof

`dead_pane_live::a_round_through_the_sheet_leaves_its_impact_there` is red on the old arm (no hole
at the sheet) and green with the fix: a hole at the sheet plane, the glass impact sound, the round
on past the sheet, and the holes gone once the pane is shot out. The run showed the round
ricochet off the shell and cross the sheet again from behind, leaving a second hole on the back
face, which is why the test asks for at least one.

Rebuilt on master d09f8b2b3, which has since landed the pane ride code (`PANE_RIDE`, off) and
`world::PANE_PART`; the branch uses that constant and keeps master's ignore on
`the_shot_out_pane_stops_blocking_a_round`. Run on the fix: `dead_pane_live` 1/0 (2 ignored),
`glass_pass_every_level_live` 3/0 (3 ignored, was 4), `breakables_live` 2/0, `skybridge_panes_live`
1/0, `ammo_pane_live` 3/0 (1 ignored), `dynobj_glass_live` 2/0, `fix_space_pane_live` 5/0,
`glass_breach_live` 1/0, `glass_dive_07000001_live` 1/0, `glass_guards_14_live` 2/0,
`rocket_glass08_live` 3/0, `urn_break_live` 4/0, `walkreal_07000008_live` 4/2 (the two reds above,
the same tick and message as on glass-pierce-2 alone).
