# agent-lvl-14: 07000014 object audit

**This branch moves the shared random stream.** Blast damage, spawn screening and guard ranges all
change who lives and when.

## Fixed

| What a player sees | Cause | Evidence | Test |
|---|---|---|---|
| The wrong wire at a generator box kills Bond where he stands (500 over 10 units); it cost about 3 | `Destroy_Smash`'s radius and damage were crossed: radius = p5 x 0.01, damage = type byte x 10 | [M 0x12bcdc-0x12bd0c, 0x17cbd0] | `the_wrong_wire_...` |
| A barrel beside Bond deals up to 30, not 2.5, on every level | Same | same | `a_barrel_going_up_...` |
| Blasts fall off smoothly and reach Bond's and guards' sphere edge | `Explode_Propagate` has no integer step: the decompile's `(int)x * (0 < x)` is `max.s` | [M 0x17c5e8-0x17c614; L obj+0x8C 1.0576 Bond, 1.02-1.06 guards] | `a_blast_falls_off_...` |
| Rook takes three times the shots (30 health, was 10) | Kind 0x10 on this level was not ported | [M 0x14b300:708-713, :776] | `rook_stands_...` |
| Guards here close to 10 units (7 for anim set 9); 07000011 to 6, combat sight 9 (5 and 8) | The per-level arm after the sight class was not ported | [M 0x14b300:423-460] | `the_guards_close_...` |
| A guard on his mission route here no longer breaks off for a noise | `AlertSound`'s level gate | [M 0x177800:35-37] | `a_guard_on_his_mission_route_...` |
| Spawners keep their distance from Bond on every level (10 units on the Delta wave) | p8 was read as float bits, so every distance was about 0 | [M 0x1379c0-0x137a1c] | `the_spawners_keep_...` |

All tests are in `destroy_blast_07000014_live`. Each was red with its fix reverted.

## Timed leaves

Rows are the audit's numbering; pl = row - 326. All writers and readers are present and ran on the
real level with the touch that starts them; docs/levels/07000014.md has the table.

- pl 294, 296, 922 (0xE1, 0xE0, 0x70): 30 ticks, then ch 0x4F, then the alarm 0x96.
- pl 668, 669: 0x20 smashes 668, 90 ticks to 0x14, which smashes 669. Pl 669 has no output channel.
- pl 672, 673: 0x23 smashes 673, 90 ticks to 0x04, which smashes 672. Pl 672 has no output channel.
- pl 923: Rook's wake 0x0D, 300 ticks to 0x75, which smashes #1251 and its barrels.
- Each output also arms an `Entity_Apocalypse`.

## Not changed: draw

- Camera shake is not drawn at all. `Drones::shake` has no reader in nf-view, so no explosion shakes
  the view. `ApocalypseFx::fade` is `glb_viewer+0x200`, the same shake word (the code called it a
  whiteout, now renamed), so the level-wide shake of every stage above is missing too.

## Test state

Passing: `destroy_blast_07000014_live`, `walk_07000014_live`, `panel_sequence`, `destroy_countdown`,
`destroy_wreck`, `fix_fusebox_cut_live`, `explode_live`, and every walk that tops up health.

**Red on this branch:** real-damage walks in `walkreal_07000008` (agent, 00agent), `0700000c`,
`07000011` (all four), `07000013`, `07000014`. `walkreal_07000014` was green at the first commit; the
others have no baseline from this branch. Most ledgers read as a 100-health budget running out to
gunfire plus one blast. One `07000011` run instead stalls at a door that stays
in its opening state. No cause is proven for any of them; none was retuned.

`walk_spawn_to_the_omega_sector_exit_real_damage_agent` (07000015) is now ignored, with the numbers
in its reason. Test edits: both fusebox walks cut group 0 only; `destroy_wreck`'s sister accepts a
door worker, because the guards now kill each other with frags at the pair; the 14 real-damage walk
asserts its landing before the fight and walks back after.

## Unproven

- That the wrong wire is lethal in play is read from code and disc only. Live question: cut a group-1
  wire at #709 and read Bond's health.
- The guards' 1.05 sphere comes from a few animated guards; undrawn ones read 0.0 in the snapshots.
- `DroneSpawner_SpawnDrone` draws over the whole group and can fail a tick; the port draws over the
  usable set. Left as is.
- An early-alarm probe did not get a guard to the switch, so the 0xDE and 0xB8 to 0x61 mission fail is
  unexercised on the real path.
