# level-0b-corridor-world-damage — report

Branch `s/level-0b-corridor-world-damage` (cut from origin/master 84e64088).

## What's done

**1. The objects that hurt in the exec-floor corridor are named — and the task's hurt-volume
premise is ruled out.** 0700000b has ZERO type-0xE4 hurt placements (`sim.hurts` empty at load
[D probe_0b_hurts dump]), zero fuseboxes, and no explosion records accompany the hits [L the
`NF_DMG_TRACE=1` run]. The −2.40 drops are **GT auto-turret bullets**: every hit carries owner −1
(the world bucket) and weapon 0x60 = the turret weapon record 96 (`0x2c55d0` = weapon_data +
96·0x10c [L ee_start_05.bin]; +0x0c = 4.0 damage, +0x24 = 4.0 spread [L]). The port's damage site
is the bullet-hit pain path (`Drone_ModPlayerHitDamage` 0x13a940 → `Player_HandlePain`, drone.rs;
4.0 raw × 0.75 player mod = 3.0 × the 0b Normal factor 0.8 [M `Player_HandlePain` 0x190404-190470,
`damage_factor` 0x09..=0x0b → 0.8] = **2.40 applied**).

The shooters [D placement dump of 0700000b.bin]: the **terminal-room wall guns #174/#175**
(kind-0x27 `GT_Create` 0x17ffa0 placements, p0 = 1 = facing-1, p4 = 0xe = enable channel 14,
p5 = 0). Objective 9's channel ch 0x4e (the files) does NOT gate them; ch 0x0e pulses at frames
12591/12615/12653 as Bond returns through the corridor [L the `NF_LEVELLOG=1` run] and the pair
fires down the terminal-room → ring corridor the whole crossing (the port's 0x72f-mask LOS [M
`GT_Track` 0x17f9c8:258] holds from both turrets to every spot z −5.5..+5.2 — the same mask the
game passes per candidate [M 0x17f9c8:271-278]). The exec-floor trio #291/#292/#293 (p4 = 0x1c,
p5 = 0x4e) never fires: ch 0x1c never rises in the whole walk [L the same run] and p5 = 0x4e is
the files-objective channel, which is UP from frame ~7566 — the game's DORMANT gate [M
0x180498 case 0: enable off → return; disable on → return] keeps them parked.

**2. The game's contact-event law is ported and the port's deviation fixed.** The game's
pain-per-contact law for BOTH hurt objects and bullets is one `Player_HandlePain` per hit
([M `Player_DealWithObjHit` 0x194458:94-103 — the class-0x27 arm, `Hurt_GetDamage` 0x17ea18;
the bullet arm through `Bullet_init` 0x12c5a0's flight]; the port already applied one pain per
bullet hit — that half matched). The real deviation was **where the turret rounds flew**: the
port's `GT_Update` case-4 fire block pushed the DEAD-ON direction to the target. The game eases
the barrel first and fires along it:
- case 4, per tracked tick: `Mat_LookAt(out, ext+0x30, target+0x30, up, 0x2d6220)` — ext+0x30 IS
  the turret's position vector (the ext holds its pose matrix at +0x00, position at +0x30;
  disasm 0x180a04-0x180a2c) — slerped by `Quat_Slerp_Acc(REC_FRAME_RATE·4.0, …)` [M 0x180498:295-330;
  the slerp's alpha = param_1 = 4/60 per tick [M 0x1e4188] — a 6.67%/tick first-order aim lag],
  then the aimed matrix is copied into obj+0x90 and the barrel objects ext+0xc0/c4/c8;
- the 0.1 s fire clock then reads `Mat_GetDir(barrel obj+0x90)` — the JUST-EASED matrix — for the
  muzzle offset AND `Bullet_init(−1, ext+0xb8, barrel, 0x2c55d0, muzzle, dir, 0xffff)`'s direction
  [M 0x180498:344-352; disasm 0x180b1c-0x180bac].

The port now eases the pose BEFORE the fire block and fires along the pose forward (gt.rs COMBAT;
the (yaw, pitch) ease stands in for the quaternion slerp [S], the same 4/60 factor). Pin test
`gt::combat_fire_rides_the_eased_barrel_not_the_dead_on_line`: a runner's barrel lags behind him
(dir z below the dead-on z), the fired dir equals the eased pose forward, and a STILL target —
the ease's fixed point — converges onto the dead-on line (dot > 0.999). Cadence/hp/re-holster
pins untouched and green (nf-game `gt::` 19/19; full nf-game 100 binaries, 1035 passed, 0 failed,
9 ignored).

**3. Verified end-to-end on the walk.** With the fix, `walkreal_0700000b_live::
walk_fresh_spawn_to_the_mission_end_real_damage` (un-ignored, real input, real health) no longer
dies in the corridor: the crossing cost dropped from **42 × −2.40 = 100.0 (death at tick 12765 in
the corridor) to 23 × −2.40 = 55.2** [L the two runs' damage logs]. The run now survives the
corridor and dies at tick 16652 on the roof (−14.71, 58.13, −2.06): the 55.2 leaves 44.8 for the
exec-floor/roof fights and the walker's plain return-fire stands break off ("killed false", the
targets at d 20.9/hp 10) — so the sister is re-ignored with that measured reason (an un-ignored
red test would break the suite).

Checked and ruled out along the way (all cited in the WORKLOG): turrets die to blasts only —
`GT_ApplyDamage` 0x1816d8's only caller is `Explode_Propagate` [M 0x17c518:190-193]; there is no
bullet-vs-turret damage in the game, so the walker cannot shoot back and no port gap exists
there. The ring-corridor door's auto-close (360 ticks [M door.rs, `FRAME_RATE_INT` × 6]) covers
the 2.5 s crossing — not a factor. The exec-floor turrets' enable channel 0x1c has no writer on
0b's data [D the level docs' writer census; L the NF_LEVELLOG run]. A `lane_fights` opt-in (the
07000013 open-floor precedent) replayed byte-identically (the fights converge to the same picks)
and was reverted.

## Test totals (the final tree)

- **nf-game**: 100 binaries — **1035 passed, 0 failed**, 9 ignored (includes `gt::` 19/19 with
  the new pin, ai_live, grenade_live 2/2, explode/stun/bullet suites).
- **nf-formats + nf-audio**: 10 binaries — 136 passed, **3 failed = exactly the 3 allowed
  slim-ISO `disc_golden`** (bigf_header_fields_behave_as_documented,
  bigf_tables_match_the_golden_manifest, directory_tree_matches_the_golden_manifest).
- **nf-view named groups** (the full ~950-binary build cannot fit this sandbox — 3.5G free vs
  ~8G needed, the standing limitation): the mission walks walkreal_0700000{1,2,3,4,5,6,9,a,b,c,d}_live
  and walkreal_070000{11,11_route,12,13,14,15,16,1b,41,4a,4a_satchel}_live — **all green**
  (0a = the six sky-bridge turrets: unaffected, the eased-barrel change only alters turret
  shots and 0a has no corridor runner); hurt_02_live, hurt_07000008_live; the fire/blast group
  (blast_debris_draw, blast_room_shelter_16, concurrent_blasts_draw, fire_modes3, fix_06_fire,
  fix_blast_scripts, fix_samurai_fire, fix_weapon_fire_modes, g049_blast_light, qworm_fire_mode,
  rapid_fire, weapon_fire_sound_ids, 0700000d_gunimp_fire_sound, grenade_live) — **all green**.
- walkreal_0700000b_live: 4 passed, 1 ignored (the real-health sister, the measured reason above).

## What's left

1. **The [L] measurement of the real machine's corridor cost** (the live check below). If the
   game's crossing costs ≲ 40 (the port shows 55.2 from the [M]-exact law), the un-ignored walk
   should pass as-is; if it measures ≈ 55, the roof attrition needs a walk-side answer (the
   route's open-floor fights) or the law has one more input the port flattens (the deploy-anim
   end pose of the wall guns — the port's pose starts at the create facing, the game's at the
   anim's final pose [S]).
2. The 0b real-health sister un-ignored to green (it is ignored with the measured reason).
3. The other real-health sisters (03/0c/0d) keep their own pinned blockers (the servant-carry
   assertion, the 0c weapon-0x1e drones + yard climb, the 0d drone #31) — untouched by this
   task's change by construction (no turrets on those routes' fight paths).

## Live checks for the main agent

1. **The corridor cost, tick by tick.** PCSX2, 0700000b fresh run, after taking the NightFire
   files (obj 9), walk Bond back through the terminal-room → ring corridor (x −1.0..−1.4,
   z −5.8..+5.3, floor y 47.18) at run speed. With `tools/trace/gdbwatch.py` watching Bond's
   health float: record every write during the crossing. Expected if the port over-damages: a
   handful of drops; if the port matches: ~20-25 drops of −2.4-class (the wall guns' 0.1 s
   cadence [M 0x180498 case 4] with the eased-barrel misses). Also note WHEN the first drop lands
   relative to the corridor mouth (the port's first hit is at the mouth, frame 12794, ~3.4 s
   after the first ch 0x0e pulse at 12591 — the deploy anim).
2. **How many wall guns fire.** During the same crossing, the muzzle flashes/sounds of #174 vs
   #175 (positions (2.26, 49.10, −21.60) / (−4.60, 49.10, −21.60) [D]): one or both?
3. **The exec-floor turrets stay parked.** ch 0x1c (enable) stays 0 and ch 0x4e reads 1 after the
   files take; #291/#292/#293 never unfold or fire during the crossing (expected [M case 0
   gates]).

## Usage

not visible to me — this environment exposes no token, credit, context-window or cost figures
for the session (no usage panel or API counters are reachable from the sandbox shell).
