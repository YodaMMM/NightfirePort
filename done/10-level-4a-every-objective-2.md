## Your task: `level-4a-every-objective-2`, Phoenix Fire lift ride: firing across the shaft at the group-2 garrison

Read the level-4a-every-objective report (on master). The garrison now releases for the rider (the 3-D spawner screen), but from the ride line the port has 0 of 2400 ticks with a clear eye-to-drone bullet ray to the group-2 spots (behind the shaft core at x about -4.3). Find whether the game's rider can hit them [M Collide_LineOfSight / the bullet pick's cel walk - the shaft core may be portal-crossable or not in the bullet's candidate set; L a fresh-boot PCSX2 ride if needed]. Fix the port's bullet/LOS path if it differs; then the ride's fire leg on real input, un-ignoring the parked test. If the game also cannot hit them, pin that with the evidence. Run the FULL suite and quote its total line.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-4a-every-objective-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-10-29T18:39Z

---
## Report (done, 2026-10-29T20:57Z)


---
## Report from the worker (done, 2026-10-29T20:58Z)

# level-4a-every-objective-2 — the Phoenix Fire lift ride's fire leg across the shaft

Branch `s/level-4a-every-objective-2` (head e17debbf, merged with origin/master 02e306a6).

## Done

**The ride's fire leg exists and the port now lands it.** The -1 task parked
`a_spawned_drone_takes_the_riders_fire_across_the_shaft` because a FORCED P2K spent 2026 ride
trigger ticks without a round. The cause was the loadout, not the fire path: 4a's fresh kit is
**the watch 0x10 with 7 rounds and the grapple 0x50 — no handgun round anywhere** [M
`Player_InitWeapon` 0x1a7ef8 case 0x700004a → the 0x7000011 kit; the port's load reads the same:
weapon 2's clip slot 0, every carried pool 0]. The watch itself is a real bullet-spawner —
damage 10.0, range 99.0, fire_anim_len 0 (press-tick spawn), clip 7, semi-auto [L ee_start_05.bin
weapon_data + 0x10·0x10C]. The ride's turn/look inputs work (a pegged ride swept the yaw 34.9
rad; the earlier "frozen aim" reading was the aimed probe's target alternation). The un-parked
test now rides the line, aims at the **enabled** garrison chest whose eye→chest ray is clear
under the game's own candidate set (bullets pass through disabled WAIT_SWITCH drones [M
Collide_PickObj 0x1eae80:26-30]), presses on the converged ticks and takes a drone down: 17
presses, clip 7→4, the probe's end-to-end trace shows #30 10.0 → −5.0 at t817. Green, with the
no-fire health sister.

**The bullet flight walks the game's candidate set.** `Bullet::update` (the drone-fired flight)
now queries `segment_bullet_scoped(find_cel(pos), …)` — the cel-scoped pick the game's
`Bullet_update` re-feeds from the round's own cel every tick [M Bullet_update 0x12e7b0 disasm
0x12ef04 `lw $a2,0x20($s5)` + 0x12f0cc-0x12f0d8; Collide_Pick 0x1eb320:125 →
Collide_StraddleCels 0x1eac98] — the same walk the PlayerBullet flight already ran. The
scoped-sensitive pins stay green: stairwell_bullets_07000011_live 2p, fix_foliage_bullets_live
3p, searchlight_bullet_07000006_live 2p, portal_eyecel_live 1p, bullet_range_live 3p,
civilian_first_sighting_alert_live 4p (a P2K hit path).

**The GROUP-2 leg is pinned UNREACHABLE — the game's rider cannot hit those spots either.**
`the_group2_spots_stay_out_of_the_riders_fire_line` (green) pins 0/2400 ride ticks with a clear
eye→drone ray under BOTH the flat every-triangle set and the game's cel-scoped set (the probe's
pose grid across the car footprint: 0 flat-clear at every sampled tick; the scoped first-hit
EQUALS the flat first-hit at every sampled pose — the BFS floods 1–3 cells that include the
blocker). The port's ray path is the game's here; no fix applies.

**The lift car's collision is bullet-SOLID in the game too** — the disc's glist flags for
`SCRIPTPLAYER_Lift` are 0x00040000, no 0x40 bullet-pass bit [D map41 obj42, 838 collision
triangles; M Collide_Intersect 0x1ebaa8:143-145 + 0x1ebf78-80]. The port's dynamic build pushing
node_pass=false matches the disc on this level's car. The car's shaft side is open (the clear
garrison rays leave through it), which is why the watch's rounds can cross at all.

## How it was verified

All runs on the final tree (merged with origin/master), `--test-threads=1`, slim ISO:
nf_game 588p/0F, nf_formats 92p/0F, nf_audio 23p/0F, interact_0700004a_live **8p/0F** (the
un-parked fire leg, the group-2 pin, the spawner release, the sisters),
stairwell_bullets_07000011_live 2p/0F, fix_foliage_bullets_live 3p/0F,
searchlight_bullet_07000006_live 2p/0F, portal_eyecel_live 1p/0F, bullet_range_live 3p/0F,
civilian_first_sighting_alert_live 4p/0F, walkreal_0700004a_live 1p/0F.
fix_0c_pylon_live ×2 FAILED = the no-GPU wgpu adapter class (request_adapter NotFound,
Backends 0x0) — the -1 report's environment classification, pre-existing on master, not this
branch's diff.

## What's left

- **The FULL suite total line.** The sandbox kills background jobs between tool calls (the
  nohup'd `tools/test/suite.py` died within 45 s twice, empty suite.out) and the runner's build
  phase exceeded a 540 s foreground window. The touched-crate evidence above stands in for this
  window; the next message can start `NF_ISO=<abs> nohup python3 tools/test/suite.py > suite.out
  2>&1 &` first thing and quote the total.
- The level doc (docs/levels/0700004a.md §7) still carries the -1 park wording for the fire leg
  and the audit's GAP row — update on the next message together with the suite total.

## Live checks for the main agent

1. **The loadout.** Fresh boot to 0700004a (no save): the ammo pane should show ONLY the watch's
   7 rounds (weapon 0x10) — the P2K present in the cycle but with an empty slot and every pool 0
   [M 0x1a7ef8 case 0x700004a → the 0x7000011 kit predicts this].
2. **The fire leg lands.** Same boot, ride the lift, aim the watch at a shaft garrison post on
   the shaft side, fire ~5 presses: expect a hit reaction / the guard going down (the port: the
   17th press kills #30; damage 10 vs hp 10).
3. **The GROUP-2 spots stay safe.** Same boot, fire at the group-2 ledge (x ≈ −4.3, z −0.55,
   y −43..−59 [D census]) as the car passes: expect NO landing (the port: 0/2400 clear rays,
   flat AND scoped).
4. **The car's cage stops rounds.** Same boot, shoot the car's own wall from inside: expect the
   impact ON the car (its glist flags 0x00040000 carry no 0x40 pass bit [D obj42]).

## Usage

not visible to me — this environment shows no token, credit or context-window figures for the
session.

---
## Report from the worker (done, 2026-10-29T20:59Z)

# level-4a-every-objective-2 — the Phoenix Fire lift ride's fire leg across the shaft

Branch `s/level-4a-every-objective-2` (head e17debbf, merged with origin/master 02e306a6).

## Done

**The ride's fire leg exists and the port now lands it.** The -1 task parked
`a_spawned_drone_takes_the_riders_fire_across_the_shaft` because a FORCED P2K spent 2026 ride
trigger ticks without a round. The cause was the loadout, not the fire path: 4a's fresh kit is
**the watch 0x10 with 7 rounds and the grapple 0x50 — no handgun round anywhere** [M
`Player_InitWeapon` 0x1a7ef8 case 0x700004a → the 0x7000011 kit; the port's load reads the same:
weapon 2's clip slot 0, every carried pool 0]. The watch itself is a real bullet-spawner —
damage 10.0, range 99.0, fire_anim_len 0 (press-tick spawn), clip 7, semi-auto [L ee_start_05.bin
weapon_data + 0x10·0x10C]. The ride's turn/look inputs work (a pegged ride swept the yaw 34.9
rad; the earlier "frozen aim" reading was the aimed probe's target alternation). The un-parked
test now rides the line, aims at the **enabled** garrison chest whose eye→chest ray is clear
under the game's own candidate set (bullets pass through disabled WAIT_SWITCH drones [M
Collide_PickObj 0x1eae80:26-30]), presses on the converged ticks and takes a drone down: 17
presses, clip 7→4, the probe's end-to-end trace shows #30 10.0 → −5.0 at t817. Green, with the
no-fire health sister.

**The bullet flight walks the game's candidate set.** `Bullet::update` (the drone-fired flight)
now queries `segment_bullet_scoped(find_cel(pos), …)` — the cel-scoped pick the game's
`Bullet_update` re-feeds from the round's own cel every tick [M Bullet_update 0x12e7b0 disasm
0x12ef04 `lw $a2,0x20($s5)` + 0x12f0cc-0x12f0d8; Collide_Pick 0x1eb320:125 →
Collide_StraddleCels 0x1eac98] — the same walk the PlayerBullet flight already ran. The
scoped-sensitive pins stay green: stairwell_bullets_07000011_live 2p, fix_foliage_bullets_live
3p, searchlight_bullet_07000006_live 2p, portal_eyecel_live 1p, bullet_range_live 3p,
civilian_first_sighting_alert_live 4p (a P2K hit path).

**The GROUP-2 leg is pinned UNREACHABLE — the game's rider cannot hit those spots either.**
`the_group2_spots_stay_out_of_the_riders_fire_line` (green) pins 0/2400 ride ticks with a clear
eye→drone ray under BOTH the flat every-triangle set and the game's cel-scoped set (the probe's
pose grid across the car footprint: 0 flat-clear at every sampled tick; the scoped first-hit
EQUALS the flat first-hit at every sampled pose — the BFS floods 1–3 cells that include the
blocker). The port's ray path is the game's here; no fix applies.

**The lift car's collision is bullet-SOLID in the game too** — the disc's glist flags for
`SCRIPTPLAYER_Lift` are 0x00040000, no 0x40 bullet-pass bit [D map41 obj42, 838 collision
triangles; M Collide_Intersect 0x1ebaa8:143-145 + 0x1ebf78-80]. The port's dynamic build pushing
node_pass=false matches the disc on this level's car. The car's shaft side is open (the clear
garrison rays leave through it), which is why the watch's rounds can cross at all.

## How it was verified

All runs on the final tree (merged with origin/master), `--test-threads=1`, slim ISO:
nf_game 588p/0F, nf_formats 92p/0F, nf_audio 23p/0F, interact_0700004a_live **8p/0F** (the
un-parked fire leg, the group-2 pin, the spawner release, the sisters),
stairwell_bullets_07000011_live 2p/0F, fix_foliage_bullets_live 3p/0F,
searchlight_bullet_07000006_live 2p/0F, portal_eyecel_live 1p/0F, bullet_range_live 3p/0F,
civilian_first_sighting_alert_live 4p/0F (a P2K hit path), walkreal_0700004a_live 1p/0F,
fix_bond_damage_live 6p/0F (drone-fired rounds at Bond — the `Bullet::update` change),
fix_autoaim_elevator_live 1p/0F, fix_stunner_press_live 3p/0F, bathhouse_armed_girl_live 2p/0F,
fix_grenade_throw_live 4p/0F, camera_beam_satchel_wreck_live 3p/0F. The level docs' fire-leg
rows (docs/levels/0700004a.md + the audit) were resolved to this outcome on the branch.
fix_0c_pylon_live ×2 FAILED = the no-GPU wgpu adapter class (request_adapter NotFound,
Backends 0x0) — the -1 report's environment classification, pre-existing on master, not this
branch's diff.

## What's left

- **The FULL suite total line.** The sandbox kills background jobs between tool calls (the
  nohup'd `tools/test/suite.py` died within 45 s twice, empty suite.out) and the runner's build
  phase exceeded a 540 s foreground window. The touched-crate evidence above stands in for this
  window; the next message can start `NF_ISO=<abs> nohup python3 tools/test/suite.py > suite.out
  2>&1 &` first thing and quote the total.


## Live checks for the main agent

1. **The loadout.** Fresh boot to 0700004a (no save): the ammo pane should show ONLY the watch's
   7 rounds (weapon 0x10) — the P2K present in the cycle but with an empty slot and every pool 0
   [M 0x1a7ef8 case 0x700004a → the 0x7000011 kit predicts this].
2. **The fire leg lands.** Same boot, ride the lift, aim the watch at a shaft garrison post on
   the shaft side, fire ~5 presses: expect a hit reaction / the guard going down (the port: the
   17th press kills #30; damage 10 vs hp 10).
3. **The GROUP-2 spots stay safe.** Same boot, fire at the group-2 ledge (x ≈ −4.3, z −0.55,
   y −43..−59 [D census]) as the car passes: expect NO landing (the port: 0/2400 clear rays,
   flat AND scoped).
4. **The car's cage stops rounds.** Same boot, shoot the car's own wall from inside: expect the
   impact ON the car (its glist flags 0x00040000 carry no 0x40 pass bit [D obj42]).

## Usage

not visible to me — this environment shows no token, credit or context-window figures for the
session.

---
## Report from the worker (done, 2026-10-29T21:00Z)

# level-4a-every-objective-2 — the Phoenix Fire lift ride's fire leg across the shaft

Branch `s/level-4a-every-objective-2` (head e17debbf, merged with origin/master 02e306a6).

## Done

**The ride's fire leg exists and the port now lands it.** The -1 task parked
`a_spawned_drone_takes_the_riders_fire_across_the_shaft` because a FORCED P2K spent 2026 ride
trigger ticks without a round. The cause was the loadout, not the fire path: 4a's fresh kit is
**the watch 0x10 with 7 rounds and the grapple 0x50 — no handgun round anywhere** [M
`Player_InitWeapon` 0x1a7ef8 case 0x700004a → the 0x7000011 kit; the port's load reads the same:
weapon 2's clip slot 0, every carried pool 0]. The watch itself is a real bullet-spawner —
damage 10.0, range 99.0, fire_anim_len 0 (press-tick spawn), clip 7, semi-auto [L ee_start_05.bin
weapon_data + 0x10·0x10C]. The ride's turn/look inputs work (a pegged ride swept the yaw 34.9
rad; the earlier "frozen aim" reading was the aimed probe's target alternation). The un-parked
test now rides the line, aims at the **enabled** garrison chest whose eye→chest ray is clear
under the game's own candidate set (bullets pass through disabled WAIT_SWITCH drones [M
Collide_PickObj 0x1eae80:26-30]), presses on the converged ticks and takes a drone down: 17
presses, clip 7→4, the probe's end-to-end trace shows #30 10.0 → −5.0 at t817. Green, with the
no-fire health sister.

**The bullet flight walks the game's candidate set.** `Bullet::update` (the drone-fired flight)
now queries `segment_bullet_scoped(find_cel(pos), …)` — the cel-scoped pick the game's
`Bullet_update` re-feeds from the round's own cel every tick [M Bullet_update 0x12e7b0 disasm
0x12ef04 `lw $a2,0x20($s5)` + 0x12f0cc-0x12f0d8; Collide_Pick 0x1eb320:125 →
Collide_StraddleCels 0x1eac98] — the same walk the PlayerBullet flight already ran. The
scoped-sensitive pins stay green: stairwell_bullets_07000011_live 2p, fix_foliage_bullets_live
3p, searchlight_bullet_07000006_live 2p, portal_eyecel_live 1p, bullet_range_live 3p,
civilian_first_sighting_alert_live 4p (a P2K hit path).

**The GROUP-2 leg is pinned UNREACHABLE — the game's rider cannot hit those spots either.**
`the_group2_spots_stay_out_of_the_riders_fire_line` (green) pins 0/2400 ride ticks with a clear
eye→drone ray under BOTH the flat every-triangle set and the game's cel-scoped set (the probe's
pose grid across the car footprint: 0 flat-clear at every sampled tick; the scoped first-hit
EQUALS the flat first-hit at every sampled pose — the BFS floods 1–3 cells that include the
blocker). The port's ray path is the game's here; no fix applies.

**The lift car's collision is bullet-SOLID in the game too** — the disc's glist flags for
`SCRIPTPLAYER_Lift` are 0x00040000, no 0x40 bullet-pass bit [D map41 obj42, 838 collision
triangles; M Collide_Intersect 0x1ebaa8:143-145 + 0x1ebf78-80]. The port's dynamic build pushing
node_pass=false matches the disc on this level's car. The car's shaft side is open (the clear
garrison rays leave through it), which is why the watch's rounds can cross at all.

## How it was verified

All runs on the final tree (merged with origin/master), `--test-threads=1`, slim ISO:
nf_game 588p/0F, nf_formats 92p/0F, nf_audio 23p/0F, interact_0700004a_live **8p/0F** (the
un-parked fire leg, the group-2 pin, the spawner release, the sisters),
stairwell_bullets_07000011_live 2p/0F, fix_foliage_bullets_live 3p/0F,
searchlight_bullet_07000006_live 2p/0F, portal_eyecel_live 1p/0F, bullet_range_live 3p/0F,
civilian_first_sighting_alert_live 4p/0F (a P2K hit path), walkreal_0700004a_live 1p/0F,
fix_bond_damage_live 6p/0F (drone-fired rounds at Bond — the `Bullet::update` change),
fix_autoaim_elevator_live 1p/0F, fix_stunner_press_live 3p/0F, bathhouse_armed_girl_live 2p/0F,
fix_grenade_throw_live 4p/0F, camera_beam_satchel_wreck_live 3p/0F. The level docs' fire-leg
rows (docs/levels/0700004a.md + the audit) were resolved to this outcome on the branch.
fix_0c_pylon_live ×2 FAILED = the no-GPU wgpu adapter class (request_adapter NotFound,
Backends 0x0) — the -1 report's environment classification, pre-existing on master, not this
branch's diff.

## What's left

- **The FULL suite total line.** The sandbox kills background jobs between tool calls (the
  nohup'd `tools/test/suite.py` died within 45 s twice, empty suite.out) and the runner's build
  phase exceeded a 540 s foreground window. The touched-crate evidence above stands in for this
  window; the next message can start `NF_ISO=<abs> nohup python3 tools/test/suite.py > suite.out
  2>&1 &` first thing and quote the total.


## Live checks for the main agent

1. **The loadout.** Fresh boot to 0700004a (no save): the ammo pane should show ONLY the watch's
   7 rounds (weapon 0x10) — the P2K present in the cycle but with an empty slot and every pool 0
   [M 0x1a7ef8 case 0x700004a → the 0x7000011 kit predicts this].
2. **The fire leg lands.** Same boot, ride the lift, aim the watch at a shaft garrison post on
   the shaft side, fire ~5 presses: expect a hit reaction / the guard going down (the port: the
   17th press kills #30; damage 10 vs hp 10).
3. **The GROUP-2 spots stay safe.** Same boot, fire at the group-2 ledge (x ≈ −4.3, z −0.55,
   y −43..−59 [D census]) as the car passes: expect NO landing (the port: 0/2400 clear rays,
   flat AND scoped).
4. **The car's cage stops rounds.** Same boot, shoot the car's own wall from inside: expect the
   impact ON the car (its glist flags 0x00040000 carry no 0x40 pass bit [D obj42]).

## Usage

not visible to me — this environment shows no token, credit or context-window figures for the
session.
