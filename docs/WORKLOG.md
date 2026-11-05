# WORKLOG

## level-1b-platform-touch
Started 2026-11-05 08:12 UTC. Branch s/level-1b-platform-touch (cut from origin/master 27c191518).
Task: on 0700001b the Drake-fight platforms break at the slightest touch. Find the game's break
rule for them (damage source, threshold, what counts as a hit) in the decompile, compare it to
the port, and fix the difference. Prove with a test that touches a platform by walking on it
(no break) and one that does what breaks it in the game. Cite [M] addresses.

Findings (probes, crates/nf-view/tests/probe1b_platforms*.rs, opt-in NF_ISO + --ignored):
- The "platforms" are the 16 solar-panel placements #41-56 (flags 0xd9, param0 script
  0x06000832, param7 0x14) around the Drake arena (z=±4.62, x 10.14..56.12, y -1.72..-9.83).
- Script 0x06000832 spawns OBJECT_SolarPanelWhole 02000a42 (intact panel, type 0x31), 10
  fragment streams (OBJECT_SolarPanel1..10, each stream one fragment at Wait 5,15,...,595) and
  Event kind 3 param 0 — i.e. the break sequence is: whole panel out, fragments in over time.

Break-rule comparison (decompile vs port, 2026-11-05 ~10:20-11:35 UTC):
- The game's rule, all three elements: damage source = class-5 bullet contacts in the panel
  objects' obj+0xD0 chains [M Collide_Update 0x1ea398], summed mask-0 over the current
  script's stream objects [M SP_GetHitDamage 0x1dc5f8; Collide_FilterBullets 0x1ece30 mask 0
  drops nothing; Collide_GetDamageNObjects 0x1ee220 sums every record's +8 damage];
  threshold = SP+0x14 (placement param 7 = 20 [D]), accumulated in SP+0x18 while the current
  script is held and unheld at hp <= damage (SP_UnPause, remainder kept) [M SP_Update
  0x1dc1c0:236-267]; a hit = Collide_PickObj 0x1eae80's gates + the glist bounding-sphere
  test, fast bullets on the 0x201 closest-point-on-segment arm with the object's radius
  [M Bullet_update 0x12e7b0:432-458 arms +0x94 = 0x101/0x201; Control_BuildWorldSph 0x133070
  builds obj+0x80/0x8C from the glist centre/radius].
- The port (nis.rs Player::update damage tail, held_hit_targets, drone.rs
  update_player_bullets) implements the same rule on every point checked; no touch break
  path exists in either [M Destroy_CollisionHandler 0x12bd48 subtracts class-0x05 only].
- Proof tests (crates/nf-view/tests/platform_touch_1b_live.rs, real input from level start):
  walking_on_a_platform_breaks_nothing (16 held, 0 damage, no debris after 900 ticks on the
  walkway) and a_bullet_contact_breaks_a_panel (sustained fire unholds a panel + debris
  spawns) — both green.
- Repro probe probe1b_touch_repro.rs: no repro of touch-break found in the port; the
  reported symptom needs the live checks in the report.
