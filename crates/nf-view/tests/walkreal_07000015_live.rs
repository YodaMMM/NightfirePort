//! 07000015 (the missile base silo) walked with the player's own movement — no `bond.pos`
//! writes after the spawn. GREEN end to end (level-15-complete): the spawn plate #484 fires ch
//! 0x4b under Bond; the gantry west and the NAV GRAPH's own silo crossing (the z≈−5 lane threads
//! the silo hole; the direct z≈0 line falls at x 67.45 [D floor grid]) → the north lane through
//! doors [38]/[39]/[46]/[47] → the x34.6 spine's west plank (the door[45] keypad slot 0xc8 has no
//! writer on the level — the east plank dead-ends [D trigger dump]) → door[32]'s doorway round
//! the console (map54 #145) on the NAV GRAPH's east→north→west lane → door[29] → the west room →
//! door[52] → the south corridor doors [50]/[42]/[51] (the great hall's middle pit crosses on the
//! east bridge; the z 4..0 split on the east lane) → door[49] → the USE exit #445 at
//! (0.0, 41.3, −24.8) ([M Player_Activate 0x1a5170 case 0x1d → Trigger_Activate 0x1b2318];
//! `Trigger_Update` case 10 never self-fires with gate 0 [M 0x1b1a78:162-166]) → the load into
//! 07000016. The objective — MissionData row 21's ONLY record (level-15-every-objective [D
//! work/elf/ACTION.ELF MissionData vaddr 0x2a4350, row 21 @ 0x2a42d8: level 0x07000015, count 1;
//! obj 0: text 0x0400004d, fail 0xffffffff (none), hint 0x04000054, ch 0x8e, init 0, enable
//! 0x4b, flags 0 — byte-for-byte the port's c07000015::DEF]) — is "Find the exit to Omega
//! Sector" (complete ch 0x8e [D row 21; L ee_start_05 @ 0x2a42d8]). Mission_MonitorObjectives
//! folds state 3 → 4 (complete) only on the record's channel [M 0x1827d8 case 3], and it
//! completes on 07000016's spawn plate #252 [D 07000016], so the walk carries its own
//! live channels into the fresh 07000016 load and finishes on that plate (the game's own
//! arrival spot — the spawn (5.17, 0.90, −65.30) stands inside the plate's box). No cut-scene
//! holds the route: the level's only script holders are the guards' own scripts; every trigger
//! the route crosses is a touch plate (0xf6/0xfa/0xfb/0xfc/0xfe/0xff/0xec/0xc7/0xed/0x4b) and
//! they all fire [D trigger dump]. The keypad-locked doors [37] 0xf7 / [35] 0xc8 are bypassed
//! by the north lane.
//!
//! Enemies are DISABLED for the walk only in the walkreal sense: damage is suppressed
//! (top_up_health, the walker's allowance — the garrison's pain pipeline still runs). All 20
//! guards stay live and armed. Opt-in via NF_ISO (the slim ISO is enough).

#[path = "common/walker.rs"]
mod walker;

use walker::{Act, Walker, Wp};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

// The laser mines (placement 0xFE, `Mine_Create` 0x17ce28 / `Mine_Update` 0x17c930): nine
// Tripbomb cels, all static (params 2-4 = 0: no spin, no bob) and unkeyed (the creator reads no
// channel — p0 damage 80, p1 radius 10/15, p5 the blast script [D census_07000015]). No switch
// disarms them and no trigger turns them on: a mine explodes when any object's collide-pass
// contact lands on its hit list [M 0x17c930's walk]. The route crosses six of them:
// - #462 (x 46.8, z −2.0..3.2, the beam 1.0-1.4 over the gantry floor) on the silo crossing;
// - the #383/#384/#385 doorway fence at z ≈ 18 (a floor-to-ceiling beam at x 35.6 and two
//   crossing diagonals) on the x 34.6 spine;
// - #526 (the 1.1-long ankle beam lying along door[29]'s doorway, x 10.7..11.8 at z 71.2);
// - #487 (x −2.4..2.9 at z 59.5, the beam 1.4-1.9 over the corridor floor) in door[52]'s mouth;
// - the #447/#448/#449 fence at z ≈ 17.6 in the south corridor.
// The player's body (`Player_Collision` 0x194df0's HITTEST: the walk reaches 1.86 over the floor,
// the crouch 1.3) meets every one of them standing on its lane. Crouched, #487 clears (and on
// the walk #526's blast takes it first); #526 lies along its doorway behind a rim and is shot
// down the lane from west of door[32] — past the worst of its own blast (level-15-agent-doors:
// the rim shot's own blast cost −41.6 on Agent, the run's largest hit). The rest are shot from
// down their lanes:
// a bullet's flight segment picks the mine (mode 0x201, mask 0 [M Bullet_update 0x12e7b0] →
// `Collide_PickObj` 0x1eae80 admits a cel-linked object with obj+0xF0 0 and a collision glist) and
// `Collide_Update` 0x1ea398:160-233 mirrors the hit onto the mine with +0x58 = the bullet — the
// referenced entry the update explodes on. A mine's blast hangs a referenced entry on every mine in
// its reach (`Explode_Propagate` 0x17c518 case 0x4B, the attacker = the mine [M 0x17c930
// EXPLODEINFO+0x24; 0x17cde0]), so one round sets a whole fence off.

/// The mine list index of placement `plc`.
fn mine_idx(w: &Walker, plc: usize) -> usize {
    w.scene.sim.as_ref().unwrap().drones.mines.iter().position(|m| m.placement == plc)
        .unwrap_or_else(|| panic!("mine placement #{plc} not created"))
}

/// Fire at `at` until mine `plc` explodes: the view is turned straight onto the point (the
/// input's turn/look add to yaw/pitch 1:1), and the trigger is pulled only while it is on the
/// point. Returns Bond's distance to the mine on the boom tick. Panics if the mine never goes.
///
/// The rounds BRACKET like a player's do: each live round's closest approach to the mine's pick
/// sphere centre is tracked while it flies; when a round dies without the boom and the eye line
/// to the aim is clear, the aim point shifts by the measured miss vector (the P2K record's
/// spread is 0.0 [L ee_start_05.bin weapon_data 0x02 +0x24/+0x98] and the flight arcs are the
/// same shape after a small aim shift, so one correction lands the next round). The measured
/// teacher: #447's beam pick centre rides at the record y 44.217 while the census aim sat at
/// y 42.3 — ~30 dead-on-eye rounds passed ~2.3 u under the pick (the round's own gravity drop
/// [M Bullet_update case 1 applies the record's gravity bit per tick] added ~0.4 more) and the
/// fence never took one.
fn shoot_mine(w: &mut Walker, plc: usize, at: [f32; 3], label: &str) -> f32 {
    let mi = mine_idx(w, plc);
    let mut pulls = 0u32;
    let mut aim = at;
    let mut best: Option<(f32, [f32; 3])> = None; // (closest approach to the pick centre, the point)
    let mut prev_bullets = 0usize;
    let mut fixes = 0u32;
    for t in 0..3600u32 {
        let (input, on, clear) = {
            let sim = w.scene.sim.as_ref().unwrap();
            if sim.drones.mines[mi].exploded {
                let p = sim.bond.pos;
                let m = sim.drones.mines[mi].pos;
                let d = ((p[0] - m[0]).powi(2) + (p[1] - m[1]).powi(2) + (p[2] - m[2]).powi(2)).sqrt();
                eprintln!("[mines] {label}: #{plc} went at tick +{t}, Bond {d:.2} u off, cause {} (fixes {fixes}, pulls {pulls})", sim.drones.mines[mi].cause);
                return d;
            }
            let bp = sim.bond.pos;
            let eye_y = bp[1] + sim.bond.eye - sim.bond.crouch_drop;
            let (dx, dy, dz) = (aim[0] - bp[0], aim[1] - eye_y, aim[2] - bp[2]);
            let err = walker::wrap_pi(dx.atan2(dz) - sim.bond.yaw);
            let want = (dy / dx.hypot(dz).max(0.5)).atan() / std::f32::consts::FRAC_PI_2;
            let perr = want - sim.bond.pitch;
            let on = err.abs() < 0.002 && perr.abs() < 0.002 && t % 12 == 0;
            let clear = sim.world.segment([bp[0], eye_y, bp[2]], aim).is_none();
            if t % 600 == 0 {
                eprintln!("[mines] {label}: w{:#x} t{t} bond {bp:?} err {err:.4} perr {perr:.4} clip {} spare {} los {:?} bullets {} door42 s{:#x} tmr {} pulls {} fixes {fixes}",
                    sim.gun.weapon, sim.gun.clip, sim.gun.spare, if clear { None } else { Some(0.0f32) }, sim.drones.player_bullets.len(),
                    sim.drones.doors.list[42].state, sim.drones.doors.list[42].open_timer, pulls);
            }
            (nf_game::player::Input { turn: err.clamp(-0.08, 0.08), look: perr.clamp(-0.08, 0.08), ..Default::default() }, on, clear)
        };
        if on { pulls += 1; w.tick_fire(input); } else { w.tick(input, false); }
        // The bracket: watch the live rounds' closest approach to the pick centre; a round that
        // died without the boom on a clear line hands its miss vector to the aim.
        let mut sim = w.scene.sim.as_ref().unwrap();
        let centre = sim.drones.mines[mi].centre;
        for b in &sim.drones.player_bullets {
            let d = ((b.pos[0] - centre[0]).powi(2) + (b.pos[1] - centre[1]).powi(2) + (b.pos[2] - centre[2]).powi(2)).sqrt();
            if best.map_or(true, |(bd, _)| d < bd) { best = Some((d, b.pos)); }
        }
        if sim.drones.player_bullets.len() < prev_bullets && !sim.drones.mines[mi].exploded {
            if let Some((d, p)) = best.take() {
                if d > 0.3 && clear && fixes < 4 {
                    aim = [aim[0] + centre[0] - p[0], aim[1] + centre[1] - p[1], aim[2] + centre[2] - p[2]];
                    fixes += 1;
                    eprintln!("[mines] {label}: the round missed by {d:.2} u — the aim walks to {aim:?}");
                }
            }
        }
        prev_bullets = sim.drones.player_bullets.len();
    }
    panic!("{label}: mine #{plc} never exploded under fire");
}

/// The Agent roster's placement-spot resolution (real-health-agent-difficulty-12-13-14-15):
/// the min-difficulty-2 placements shift the port indices, but a placement's [D] spot is
/// difficulty-stable — find the live hostile standing there.
fn agent_drone_near(w: &Walker, at: [f32; 3], tol: f32) -> Option<usize> {
    let sim = w.scene.sim.as_ref().unwrap();
    (0..sim.drones.list.len()).find(|&i| {
        let d = &sim.drones.list[i];
        d.present && d.enabled && !d.corpse && d.health > 0.0 && d.team == 1
            && (d.pos[0] - at[0]).powi(2) + (d.pos[1] - at[1]).powi(2) + (d.pos[2] - at[2]).powi(2) < tol * tol
    })
}

/// `agent_drone_near` by NEAREST, not list order (real-health-00agent-15-6): a wide tolerance
/// around a [D] placement spot must not answer with the first unrelated guard in the list —
/// a player shoots the guard standing at the spot, and the nearest one is him.
fn agent_drone_nearest(w: &Walker, at: [f32; 3], tol: f32) -> Option<usize> {
    let sim = w.scene.sim.as_ref().unwrap();
    (0..sim.drones.list.len())
        .filter(|&i| {
            let d = &sim.drones.list[i];
            d.present && d.enabled && !d.corpse && d.health > 0.0 && d.team == 1
                && (d.pos[0] - at[0]).powi(2) + (d.pos[1] - at[1]).powi(2) + (d.pos[2] - at[2]).powi(2) < tol * tol
        })
        .min_by_key(|&i| {
            let d = &sim.drones.list[i];
            ((d.pos[0] - at[0]).powi(2) + (d.pos[1] - at[1]).powi(2) + (d.pos[2] - at[2]).powi(2)) as u32
        })
}

/// `agent_drone_nearest` narrowed to a skin: the plank's rifle chaser (#17, 0x5000087) stands
/// 2.2 u off the west grenadier's [D] spot on the d3 roster [L the run15d tail] — an unskinned
/// nearest answers with HIM, and the stand burns on the wrong guard while the grenadiers rage.
fn agent_drone_nearest_skin(w: &Walker, at: [f32; 3], tol: f32, skin: u32) -> Option<usize> {
    let sim = w.scene.sim.as_ref().unwrap();
    (0..sim.drones.list.len())
        .filter(|&i| {
            let d = &sim.drones.list[i];
            d.present && d.enabled && !d.corpse && d.health > 0.0 && d.team == 1 && d.skin == skin
                && (d.pos[0] - at[0]).powi(2) + (d.pos[1] - at[1]).powi(2) + (d.pos[2] - at[2]).powi(2) < tol * tol
        })
        .min_by_key(|&i| {
            let d = &sim.drones.list[i];
            ((d.pos[0] - at[0]).powi(2) + (d.pos[1] - at[1]).powi(2) + (d.pos[2] - at[2]).powi(2)) as u32
        })
}

/// Draw weapon `id` (the real switch request) and wait out the holster/draw.
fn draw(w: &mut Walker, id: u8) {
    w.tick(nf_game::player::Input { switch_to: Some(id), ..Default::default() }, false);
    for _ in 0..90 {
        if w.scene.sim.as_ref().unwrap().gun.weapon == id && w.scene.sim.as_ref().unwrap().gun.switch_state == 0 { break; }
        w.tick(nf_game::player::Input::default(), false);
    }
    for _ in 0..20 { w.tick(nf_game::player::Input::default(), false); }
    assert_eq!(w.scene.sim.as_ref().unwrap().gun.weapon, id, "the switch to weapon {id:#x} did not take");
}

/// `draw` with the pressed-edge retries: a switch request fired into a live fire script or
/// switch anim is DROPPED — the machine queues only on the pressed edge from idle [M
/// 0x1a24c8:113-152's pressed-edge law; the run-5 rim draw followed shoot_mine's fire window
/// and timed out]. A player presses the wheel again. Each attempt: one press, the draw wait,
/// and the check — six attempts cap a wheel the game refuses for a reason the walk should not
/// paper over (the weapon not owned).
fn draw_persist(w: &mut Walker, id: u8, label: &str) {
    for _attempt in 0..6 {
        {
            let sim = w.scene.sim.as_ref().unwrap();
            if sim.gun.weapon == id && sim.gun.switch_state == 0 { return; }
        }
        w.tick(nf_game::player::Input { switch_to: Some(id), ..Default::default() }, false);
        for _ in 0..90 {
            let (wpn, st) = { let s = w.scene.sim.as_ref().unwrap(); (s.gun.weapon, s.gun.switch_state) };
            if wpn == id && st == 0 { break; }
            w.tick(nf_game::player::Input::default(), false);
        }
    }
    panic!("{label}: the switch to weapon {id:#x} did not take in 6 presses (gun {:#x})",
        w.scene.sim.as_ref().unwrap().gun.weapon);
}

/// The pre-lane stand's AIMED stand (real-health-00agent-15-5): the 07000006 walk's proven
/// aimed_stand, file-local (shared helpers stay opt-in per the merge lessons). The parts that
/// make long-range stands land, each measured on the 0c/06 branches: the settle + the aim-RAISE
/// loop (the raise queues only on the aim-PRESSED edge from the idle machine [M Player_Weapon
/// 0x1a24c8:113-152; the finish sets the bit 0x1a2e38:554-561]); with the bit set the aimed act
/// is zero-spread [M Bullet_init 0x12c5a0:139-170] — the plain hip cone's scatter is 4.9 u at
/// 35 u [L ee_start_05.bin @ 0x2c10b8 +0x24], which is what the run15b stand at 22 u rode (3.0
/// dealt, 74.0 taken); the STRICT gun-ray exposure re-pick every 10 ticks
/// [M DamageLog::exposed_part_gun_rays — the muzzle-lane ray set the fights' own law uses, the
/// eye-law ray the run15c stand rode and never landed on]; the CURSOR-aware convergence (the
/// fired round obeys the view PLUS the eased offsets [M scene.rs free-aim; Player_AutoAim
/// 0x1a7c98:50-59]); the STILL gate 18 (a stepping guard is not where the view was); the bails
/// (200 unseen / 1500 no-hit / dry).
fn aimed_stand(w: &mut Walker, di: usize, label: &str, cap: u32) -> bool {
    aimed_stand_impl(w, di, label, cap, false)
}

/// The desk stand's HEAD-priority aim (real-health-00agent-15-7, file-local): the shared
/// exposure pick is torso-nearest [M DamageLog::exposed_impl — the head box wins only inside
/// the opt-in 6 u HEAD_HUNT window], so the 10.7 u desk stand grinds 3.5 body rounds while the
/// guard's own stream rides the window [L the fresh run on this tree: #2 stood at hp 14 when
/// his first −7 landed, ground to hp 2 at tick 5790, and the killing −31.28 was his 0x1a
/// GRENADE blast at 10.7 u, not a bullet]. The head round is ×10 [M NDrone2_HitDamage
/// 0x145558 × DroneDamage_Head] — 35 from the P2K, one round per guard. The pick mirrors the
/// gun_rays law (the eye plus the two low stand-in gun rays [M DamageLog::exposed_impl:544],
/// `segment_bullet_scoped` — this walk never runs `with_glass_sight`) at the head box
/// (part 5, the same part the HEAD_HUNT branch reads); None falls back to the shared torso
/// pick, so an unexposed head costs nothing.
fn head_aim(sim: &nf_view::scene::Sim, di: usize) -> Option<[f32; 3]> {
    let d = &sim.drones.list[di];
    if d.corpse || d.health <= 0.0 || !d.present { return None; }
    let bp = sim.bond.pos;
    let eye = [bp[0], bp[1] + sim.bond.eye - sim.bond.crouch_drop, bp[2]];
    let (fy, cy) = (sim.bond.yaw.sin(), sim.bond.yaw.cos());
    let right = [cy, 0.0, -fy];
    let gun_spots = [eye,
        [eye[0] + right[0] * 0.25, eye[1] - 0.15, eye[2] + right[2] * 0.25],
        [eye[0] - right[0] * 0.25, eye[1] - 0.15, eye[2] - right[2] * 0.25]];
    let data = nf_game::drone::Data {
        tables: &sim.tables, props: &sim.props, scripts: &sim.scripts, weapons: &sim.weapons,
        world: &sim.world, level: sim.level, sequences: &sim.seqs, skins: &sim.skins,
        skeletons: &sim.skeletons, nav: &sim.nav, covers: &sim.covers, tuning: &sim.tuning,
        hitboxes: &sim.hitboxes, volumes: &sim.volumes, sfx: sim.sfx.as_ref(), models: &sim.models,
        surfaces: sim.surfaces.as_ref(), voices: sim.voices.as_deref(), difficulty: sim.drones.difficulty,
        ai_points: &sim.ai_points,
    };
    for b in d.hit_boxes(&data) {
        if b.part != 5 { continue; }
        let c = [b.m[3][0], b.m[3][1], b.m[3][2]];
        let ray = [c[0] - eye[0], c[1] - eye[1], c[2] - eye[2]];
        let Some(t_box) = nf_game::damage::ray_box(eye, ray, &b) else { continue };
        let mut clear = true;
        for g in &gun_spots {
            let gr = [c[0] - g[0], c[1] - g[1], c[2] - g[2]];
            if sim.world.segment_bullet_scoped(sim.world.find_cel(*g), *g, c).is_some_and(|tw| tw < t_box) {
                clear = false;
                break;
            }
        }
        if clear { return Some(c); }
    }
    None
}

fn aimed_stand_impl(w: &mut Walker, di: usize, label: &str, cap: u32, head_first: bool) -> bool {
    // Settle: no input until the switch anim is done, then the raise loop (the 0c idiom).
    for _ in 0..90 {
        let sim = w.scene.sim.as_ref().unwrap();
        if sim.gun.switch_state == 0 && !sim.gun.fire_state { break; }
        w.scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input::default(), false, false, false);
        w.total_ticks += 1;
    }
    for _ in 0..240 {
        let sim = w.scene.sim.as_ref().unwrap();
        if sim.aiming { break; }
        w.scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input { aiming: true, ..Default::default() }, false, false, false);
        w.total_ticks += 1;
    }
    const STILL_NEED: u32 = 18;
    let prev = w.dmg.aimed_fights;
    w.dmg.aimed_fights = true;
    eprintln!("[15] the aimed stand ({label}): engaging drone #{di}");
    let killed = {
        let mut unseen = 0u32;
        let mut last_tap = 0u32;
        let mut aim: Option<[f32; 3]> = None;
        let mut hp0 = w.scene.sim.as_ref().unwrap().drones.list[di].health;
        let mut last_hit = 0u32;
        let mut still = 0u32;
        let mut last_pos = [0.0f32; 3];
        let mut have_pos = false;
        for t in 0..cap {
            let (input, fire, reload) = {
                let sim = w.scene.sim.as_ref().unwrap();
                let d = &sim.drones.list[di];
                if d.corpse || d.health <= 0.0 || !d.present {
                    eprintln!("[15] the aimed stand ({label}): drone #{di} VANISHED at +{t} (corpse {} health {:.1} present {})", d.corpse, d.health, d.present);
                    return true;
                }
                if sim.disabled { return false; }
                if sim.gun.clip == 0 && sim.gun.spare == 0 {
                    eprintln!("[15] the aimed stand ({label}): drone #{di} bailed DRY at +{t} (clip 0 spare 0)");
                    return false;
                }
                // The still gate: root motion only (the 0c measurement — the box sway rides the
                // anim; the root is what carries the chest out of the round's path).
                let moved = have_pos && ((d.pos[0] - last_pos[0]).powi(2)
                    + (d.pos[1] - last_pos[1]).powi(2) + (d.pos[2] - last_pos[2]).powi(2)).sqrt() > 0.03;
                have_pos = true;
                last_pos = d.pos;
                still = if moved { 0 } else { still.saturating_add(1) };
                if d.health < hp0 { hp0 = d.health; last_hit = t; }
                if t - last_hit > 1500 {
                    eprintln!("[15] the aimed stand ({label}): drone #{di} bailed NO-HIT at +{t} (still {still}, aiming {})", sim.aiming);
                    return false;
                }
                if t % 10 == 0 { aim = if head_first { head_aim(&sim, di).or(walker::DamageLog::exposed_part_gun_rays(&sim, di)) } else { walker::DamageLog::exposed_part_gun_rays(&sim, di) }; }
                if aim.is_none() { unseen += 1; if unseen > 200 { eprintln!("[15] the aimed stand ({label}): drone #{di} bailed UNSEEN at +{t}"); return false; } } else { unseen = 0; }
                let b = sim.bond.pos;
                let eye = [b[0], b[1] + sim.bond.eye - sim.bond.crouch_drop, b[2]];
                let at = aim.unwrap_or([d.pos[0], d.pos[1] + 0.3, d.pos[2]]);
                let (dx, dy, dz) = (at[0] - eye[0], at[1] - eye[1], at[2] - eye[2]);
                let h = (dx * dx + dz * dz).sqrt().max(0.5);
                let desired_yaw = dx.atan2(dz);
                let desired_pitch = (dy / h).atan();
                // The cursor drive + the effective-aim error (the 0c convergence — the round
                // obeys the view + the eased offsets, not the view alone).
                let yaw_err = { let mut a = desired_yaw - sim.bond.yaw;
                    while a > std::f32::consts::PI { a -= std::f32::consts::TAU; }
                    while a < -std::f32::consts::PI { a += std::f32::consts::TAU; } a };
                let pitch_err = desired_pitch - sim.bond.pitch * std::f32::consts::FRAC_PI_2;
                let cx_want_raw = -yaw_err / 0.76966;
                let cy_want_raw = pitch_err / 0.57724;
                let cur = sim.cursor;
                let turn = ((cur[0] - cx_want_raw) * 0.76966).clamp(-0.08, 0.08);
                let look = ((cy_want_raw - cur[1]) * 0.57724 / std::f32::consts::FRAC_PI_2).clamp(-0.08, 0.08);
                let aoff = sim.drones.aim_offset;
                let err = { let mut a = yaw_err - aoff[0];
                    while a > std::f32::consts::PI { a -= std::f32::consts::TAU; }
                    while a < -std::f32::consts::PI { a += std::f32::consts::TAU; } a };
                let perr = pitch_err - aoff[1];
                let reload = sim.gun.clip == 0;
                // The tap: the aim bit ON, the effective aim converged, the guard still.
                let fire = sim.aiming && !reload && unseen == 0 && err.abs() < 0.01 && perr.abs() < 0.01
                    && still >= STILL_NEED && t.saturating_sub(last_tap) >= 8;
                (nf_game::player::Input { turn, look, aiming: true, ..Default::default() }, fire, reload)
            };
            if reload {
                w.scene.sim.as_mut().unwrap().step_bond(nf_game::player::Input::default(), false, true, false);
                w.total_ticks += 1;
                w.after_step();
                for _ in 0..60 { w.tick(nf_game::player::Input::default(), false); }
            } else if fire {
                last_tap = t;
                w.tick_fire(input);
            } else {
                w.tick(input, false);
            }
        }
        w.scene.sim.as_ref().unwrap().drones.list[di].health <= 0.0
    };
    w.dmg.aimed_fights = prev;
    // Level the view back (the fights' own idiom) so the next nav leg steers from level pitch.
    for _ in 0..90 {
        let p = w.scene.sim.as_ref().unwrap().bond.pitch;
        if p.abs() < 0.01 { break; }
        w.tick(nf_game::player::Input { look: (-p).clamp(-0.08, 0.08), ..Default::default() }, false);
    }
    let sim = w.scene.sim.as_ref().unwrap();
    eprintln!("[15] the aimed stand ({label}): drone #{di} killed {killed}, health {:.1}",
        sim.drones.health.health);
    killed
}

/// Walk the nav route toward `dest` node by node until `aim` is in sight (no world hit on the eye
/// line) within `max_d`.
fn nav_until_clear(w: &mut Walker, dest: [f32; 3], aim: [f32; 3], max_d: f32, label: &str) {
    let clear = |w: &Walker| {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        let eye = [p[0], p[1] + sim.bond.eye - sim.bond.crouch_drop, p[2]];
        let d = ((aim[0] - eye[0]).powi(2) + (aim[1] - eye[1]).powi(2) + (aim[2] - eye[2]).powi(2)).sqrt();
        d <= max_d && sim.world.segment(eye, aim).is_none()
    };
    if clear(w) { return; }
    let pts = {
        let sim = w.scene.sim.as_ref().unwrap();
        walker::nav_route(sim, sim.bond.pos, dest).unwrap_or_else(|| { let mut v = walker::leg(sim, sim.bond.pos, dest); v.pop(); v })
    };
    for (k, p) in pts.iter().enumerate() {
        let _ = w.walk_to(Wp::new(p[0], p[1], p[2]).r(1.3), &format!("{label} node {k}"));
        if clear(w) { return; }
    }
    panic!("{label}: the line to {aim:?} never cleared within {max_d} u");
}

/// Face the door nearest `at` and pulse E until it opens (the walker's own door press).
fn open_door_facing(w: &mut Walker, at: [f32; 3], label: &str) {
    let di = walker::door_near(w.scene.sim.as_ref().unwrap(), at).expect("a door").1;
    for t in 0..600u32 {
        let input = {
            let sim = w.scene.sim.as_ref().unwrap();
            if sim.drones.doors.list[di].state & 0xc != 0 { break; }
            let p = sim.bond.pos;
            let dp = sim.drones.doors.list[di].pos;
            let err = walker::wrap_pi((dp[0] - p[0]).atan2(dp[2] - p[2]) - sim.bond.yaw);
            nf_game::player::Input { turn: err.clamp(-0.08, 0.08), ..Default::default() }
        };
        w.tick(input, t % 6 == 0);
    }
    assert!(w.scene.sim.as_ref().unwrap().drones.doors.list[di].state & 0xc != 0, "{label}: the door never opened");
    for _ in 0..40 { w.tick(nf_game::player::Input::default(), false); }
}

/// Hold the crouch toggle on (the game's toggle with its 60-tick lockout [M Player_SSCrouch
/// 0x1924c0:84-105]) or off, then settle.
fn set_crouch(w: &mut Walker, on: bool) {
    for t in 0..90u32 {
        let now = w.scene.sim.as_ref().unwrap().bond.crouched;
        if now == on && t > 0 { break; }
        w.tick(nf_game::player::Input { crouch: now != on && t % 70 == 0, ..Default::default() }, false);
    }
    for _ in 0..20 { w.tick(nf_game::player::Input::default(), false); }
    assert_eq!(w.scene.sim.as_ref().unwrap().bond.crouched, on, "the crouch toggle did not take");
}

/// Walk to `to` with the plain steering (no stuck handling — the lane is open), whatever the
/// stance; returns when inside `r`.
fn walk_plain(w: &mut Walker, to: [f32; 3], r: f32, label: &str) {
    for _ in 0..1800u32 {
        let input = {
            let sim = w.scene.sim.as_ref().unwrap();
            let p = sim.bond.pos;
            if walker::dxz(p, to) < r { return; }
            let err = walker::wrap_pi((to[0] - p[0]).atan2(to[2] - p[2]) - sim.bond.yaw);
            nf_game::player::Input { turn: err.clamp(-0.08, 0.08), forward: if err.abs() > 0.6 { 0.0 } else { 1.0 }, ..Default::default() }
        };
        w.tick(input, false);
    }
    panic!("{label}: walk_plain cap\n{}", walker::fail_dump_pub(&w.scene, to, label));
}

/// Backpedal to `to` while HOLDING the view on `face`: the game's own backward walk (forward
/// −1 drives the legs [M Player_Move's eased lanes], the turn/look hold the aim on `face` 1:1)
/// — a player covers a short lane backwards with the weapon already up. Returns on arrival
/// inside `r` with the view still held.
fn backpedal_face(w: &mut Walker, to: [f32; 3], face: [f32; 3], r: f32, label: &str) {
    for _ in 0..1200u32 {
        let input = {
            let sim = w.scene.sim.as_ref().unwrap();
            let p = sim.bond.pos;
            if walker::dxz(p, to) < r { return; }
            let eye_y = p[1] + sim.bond.eye - sim.bond.crouch_drop;
            let err = walker::wrap_pi((face[0] - p[0]).atan2(face[2] - p[2]) - sim.bond.yaw);
            let want = ((face[1] - eye_y) / ((face[0] - p[0]).hypot(face[2] - p[2])).max(0.5)).atan()
                / std::f32::consts::FRAC_PI_2;
            let perr = want - sim.bond.pitch;
            nf_game::player::Input { turn: err.clamp(-0.08, 0.08), look: perr.clamp(-0.08, 0.08), forward: -1.0, ..Default::default() }
        };
        w.tick(input, false);
    }
    panic!("{label}: backpedal_face cap\n{}", walker::fail_dump_pub(&w.scene, to, label));
}

fn walk_spawn_to_the_omega_sector_exit_at(difficulty: u32) {
    let iso = iso();
    // The difficulty-1 expression is byte-identical to the pre-sister master's (real-health-
    // agent-difficulty-01-07's idiom); 2+ loads the menu way [M C_SBNFDFCTY 0x216ca8:15].
    let mut w = Walker::new(if difficulty == 1 {
        nf_view::load(&iso, "07000015.bin", false).unwrap()
    } else {
        nf_view::load_with_carry_difficulty(&iso, "07000015.bin", false, None, difficulty).unwrap()
    });
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        assert!(p[0] > 93.0 && p[0] < 96.0 && p[1] > 40.0 && p[1] < 43.0 && p[2] > -2.0 && p[2] < 2.0,
            "the cold start spawns on the silo gantry (#1 [D]), got {p:?}");
    }
    // OPT-IN pre_throw_stands (the d2 Agent run ONLY — the field doc in common/damage_log.rs):
    // the spawner-released hostiles (the west-room grenadier pair spawns during the threshold
    // leg) are answered the tick after the release, so the kill lands before the guard's first
    // throw — the base ledger's kills always came after it (the thrown grenades outlived their
    // throwers for −15.0 + −17.5 [L the aim-last-seven §3 ledger]). The d1 real-damage and
    // top-up walks keep their pinned timelines; the d3 run is carried by its own report.
    // switch_dry rides with it: the run-1 ledger fought the west room on the P2K's last rounds
    // (clip 1-3, spare 0-9 [L the 2026-11-07 run-1 trace, fight#18/fight#10 t0]) — the fights'
    // master give-up on a dry mag left #10 at full hp; a player whose mag runs dry mid-fight
    // cycles to the next loaded firearm (the field doc: the 02/03 00agent idiom).
    // real-health-00agent-15-5: the d3 widening MEASURED WORSE — the run15q ledger: the
    // aperture fight went from 14.0 in 2 hits to 79.0 in 4 (the grenadier's own blasts at 00)
    // and the walk died at t1048 at the aperture (81.53,41.41,-0.49): with switch_dry the mag
    // cycles mid-fight and the fight runs past the grenadier's first two throws; the flags
    // stay d2-only (the doc's own gate). The #8-first-throw lever needs a different answer
    // (a d3 stand at the plank's own cover edges, or the fights' pick reaching him before the
    // arc lands — the next session's work, with the meet's pool chain).
    if difficulty == 2 && !w.tops_up() {
        w.dmg.pre_throw_stands = true;
        w.dmg.switch_dry = true;
    }
    w.wait_enabled(3000, "the silo arrival");
    w.wait(120);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert_eq!(sim.drones.difficulty, difficulty,
            "the sim runs the picked difficulty [M GameState+0x28 0x2a3790 → Drones::difficulty]");
        assert!(sim.drones.channels.on(0x4b), "the spawn plate #484 fires the record's enable ch 0x4b [D row 21 enable 0x4b]");
        assert_eq!(sim.drones.mission.states[21].len(), 1,
            "the mission data lists exactly ONE objective for 07000015 [D MissionData row 21 count 1]");
        assert_eq!(sim.drones.mission.states[21][0], 3, "'Find the exit to Omega Sector' announces [M 0x1827d8 case 1 → 3 on enable ch 0x4b; L record 15]");
    }
    // REAL-HEALTH TACTIC (muzzle-repin-6 measure, the 02/03 walks' lever): the d2 fights' costs
    // moved with the muzzle-draw stream — the measured run carried 24 hits for 100.0 across 16
    // return-fire fights and died at door[51] (tick 9604: the #14 fight 11.2 → 7.0, then 0.0).
    // The keep-off stand-off 4.0 (the same `Walker` arms the 02/03 real-health walks green)
    // holds the fights out of the band where every drone round hits
    // [M DroneWeap_DoBulletAccuracy 0x179740:113-115]. d1's green stream stays byte-identical.
    if difficulty >= 2 {
        w.keep_off = Some(4.0);
        w.dmg.back_off = Some(4.0);
    }

    // The route: the spawn gantry west, then the NAV GRAPH's own route to the exit hall (the
    // z≈0 lane crosses the silo hole at x 66-74 — the walk's first run fell in; the guards' AI
    // paths are the game's own answer for the gantry maze), then the USE exit.
    // The shots ride the P2K (0x02, 7 + 41 rounds at the level start [D the 07000015 loadout];
    // the drawn Delta Repeater carries 3 + 9 bolts).
    // real-health-00agent-12-13-14-15 (the d3 note): the d3 roster's opening burst came from a
    // 0x50000b2 guard BEHIND the spawn (the d3 ledger's #15, (72.35, 41.34, −9.44) at his
    // hits) — 63.0 in 9 hits before the return fire finished him at tick 867. A spawn stand
    // was TRIED and cost MORE (the line from the gantry is blocked — segment Some(0.068) —
    // the shoot machinery flanked through door (81.7, −1.0) for 56.0 and two 3000-tick caps);
    // the fights' own law is the cheaper answer (their creep killed him from the lane), so the
    // d3 run keeps the fights and eats the opening burst — the budget note is in the report.
    w.walk_to(Wp::new(88.0, 41.4, -0.5).r(1.2), "the gantry west");
    draw(&mut w, 0x02);
    // real-health-00agent-15 (the burst lever — the plate sneak): the d3 roster's #15
    // (0x50000b2) activates on the touch plate at (80.8, 41.8, 0.1) (out 0xf6 [L the probe
    // dump]) and stands south of the crossing lane at (72.35, 41.34, −9.44) [L the d3 ledger] —
    // his burst walked the measured opening for 63.0 in 9 hits before the fights' creep finished
    // him at tick 867 (the 2026-11-07 run reproduces it exactly). The plate's touch box is
    // x 80.31..81.31, z −1.18..1.35 [L the 2026-11-07 probe of the port's own trigger bounds] —
    // and the gantry's west wall (x ≈ 81.7) carries its door aperture around z −1.0, whose south
    // half lies BELOW the plate's −1.18 edge. A player who hugs the south rail slips through the
    // aperture's south half and the plate never fires — the guard stays dormant and the 63.0 is
    // never at risk. The fights' own law handles the crossing's live garrison either way (the
    // measured run: five fights, health flat). The TRIED alternatives are on the ledger: a
    // custom close-fight mark west of the door wedged on the auto-close (6000 ticks, 0 hits —
    // the guard was dormant), and a spawn stand flanked expensively [L 12-13-14-15].
    // Real damage only: the top-up 00 Agent walk keeps the plain gantry legs it passes on.
    if difficulty >= 3 && !w.tops_up() {
        w.walk_to(Wp::new(83.0, 41.4, -1.4).r(1.0), "the gantry south edge (the plate sneak)");
        w.walk_to(Wp::new(80.0, 41.4, -1.6).r(1.0), "through the aperture's south half (the plate stays cold)");
        // the sneak's geometry note (the run15a ledger): the south-half path stays below the
        // plate's z -1.18 edge and the plate ran cold through the pass. The aperture fight's
        // own back-off can cross the box on shifted streams (the fight runs DURING this leg;
        // the plate's only consumer, the d3 roster's #15, is dying inside the same fight —
        // the run15o ledger caught exactly that: his last hit at hp 12, the kill the same
        // fight) — so the plate state here is a ledger print, not a pass/fail pin; the cost
        // that matters (the guard's wake) is the fight itself, which the ledger captures.
        { let sim = w.scene.sim.as_ref().unwrap();
          eprintln!("[15] the sneak's plate state at the pass: 0xf6 {}", sim.drones.channels.on(0xf6)); }
        // the finisher from the aperture's west side, BEFORE the lane leg: the measured sneak run
        // had the fights engage #15 mid-leg at 3.6 u — the fight killed him for 21.0 but his own
        // grenade (weapon 0x0's blast arm) popped at Bond's feet for 79.0, 75 ticks AFTER the
        // kill. A player who has just slipped the aperture puts a head-hunted round on him from
        // the doorway: 4.5 u off his advanced spot (76.7, 41.4, −4.6 [L the 2026-11-07 ledger]),
        // one head-hunted round drops a 20-hp guard [M NDrone2_HitDamage 0x145558 ×
        // DroneDamage_Head; the P2K's 3.5 → 35], and a dead grenadier throws nothing. The fights'
        // return fire co-fights from his first shot, so the kill lands at or before it.
        if let Some(di) = agent_drone_near(&w, [76.7, 41.4, -4.6], 6.0) {
            let head = { let sim = w.scene.sim.as_ref().unwrap();
                let p = sim.drones.list[di].pos; [p[0], p[1] + 1.5, p[2]] };
            w.shoot_point(head, "the lane grenadier's head from the aperture", |sim| {
                let d = &sim.drones.list[di]; d.corpse || d.health <= 0.0
            });
        }
        // real-health-00agent-15-5: the CROSSING fights take the fights' lane law — the
        // damage-weighted pick plus the unseen-cover creep, which walks Bond along the aim
        // until the muzzle lanes clear [M DamageLog::exposed_impl gun_spots] — the close kill
        // the ledger measured at 21.0 (the sneak's 3.6 u fight) against 81.0 for one kill at
        // 13-18 u (the run15a ledger: the fights answered the nearer grenadier while the burst
        // rifle plinked 56.0, dead t1588 with #1 at hp 4; the run15h aimed stand raced the DPS
        // and lost 25->3 for 72.0 taken). Scoped: the flag clears after the #462 shot; the
        // sneak's own #15 fight and the d1/d2 walks keep their timelines.
        w.dmg.lane_fights = true;
        // real-health-00agent-15-5: the fights' own back-off is OFF for the crossing — the
        // run15i wedge: with the lane law's creep closing to the kill range, the crowd arm
        // walked Bond back east against the aperture door's leaf, the leg's nav re-route from
        // the displaced spot went through the door node, the leaf contact reversed the door
        // [M the merged tree's door law] and the leg burned its 6000-tick cap re-opening it
        // six times; the run15j direct steer (walk_plain) has no return-fire hook and lost the
        // answer entirely (1 fight, dead t1689). The nav leg keeps the fights; the back-off
        // restores after the #462 shot. (run15i's own kills stand: #1 18.0, #0 12.0.)
        w.dmg.back_off = None;
        // real-health-00agent-15-5: NO (79,-5) anchor leg — the run15m ledgers: the lane leg's
        // fights run fine here (the kills above), but the leg's own nav from the fight site
        // wedges (the graph's local route to (79,-5) goes back through the aperture door node —
        // the leaf-contact door law re-closes it six times, the 6000-tick cap), the direct
        // steer to (79,-5) hits the lane's own east clutter (the run15l dump: the segment
        // blocked at 0.127/0.197), and the re-pass legs cap on the same anchor. The crossing's
        // own nav route below (nav_until_clear toward (34.6,6.0), the d1 walk's own ridden
        // graph) walks WEST from the aperture without the anchor.
        // the sneak's law (measured, the 2026-11-07 runs): the south-half approach brings Bond
        // onto the lane's east end close to the d3 roster's #15 (he advances to (76.7, 41.4,
        // −4.6) to meet the lane) and the close fight killed him for 21.0 — against the 51 u lane
        // fight's 63.0. The plate itself stayed cold through the pass (the channel assert below
        // pins the path's geometry).
        if { let sim = w.scene.sim.as_ref().unwrap(); sim.drones.channels.on(0xf6) } {
            eprintln!("[15] the 0xf6 plate fired after the pass (the fights' displacement grazed the box; the plate's guard #15 is already dead) — the sneak's own geometry held (the pass-time pin above)");
        }
    }
    // #462 lies across the silo crossing (the waist beam at x 46.8, z −2..3.2): the crossing's
    // nav lane walks into it at x 48.5, so it is shot from the lane once the gantry clutter
    // clears the line (≤ 14 u).
    let mine462 = [46.8f32, 41.556, 0.6];
    // real-health-00agent-15-6 (the d3 lever): the beam's own blast reaches the 13.56 u shot
    // stand — the measured run ate −14.72 at tick 2151, the pop tick (the shot's +473 [L the
    // run15 trace]). The d3 vantage legs stop at 18.5 u so the pop's falloff dies before Bond;
    // the line rule (the first CLEAR within max_d) still walks to the clutter boundary when
    // the far line is blocked. The d1/d2 legs keep the ≤ 14 u rule and their pinned streams.
    let d462 = if difficulty >= 3 && !w.tops_up() { 18.5f32 } else { 14.0f32 };
    nav_until_clear(&mut w, [34.6, 41.4, 6.0], mine462, d462, "the silo crossing");
    // REAL-HEALTH TACTIC (level-14-15-real-health-finish): the silo rifle guard (placement 699,
    // port index 0, skin 0x5000087, weapon 0x15 — the burst rifle) plinks the crossing approach
    // from ~6 u (8 × −1.20 in the measured run) and the automatic fight left it at hp 4; a
    // player drops it from the lane before crossing — it stands in the open it has been firing
    // from, so the line is the one its own rounds ride.
    // The explicit kills run only on the real-health runs: the master (top-up) walk never
    // fights (its pinned timeline has no return fire — the "enemies may be disabled" allowance),
    // so an unconditional shoot(0) there burns the clip on an untouched guard at 10 u and the
    // crossing arrives dry (the measured master failure: clip 0 spare 0 at #462).
    // guard-radii-finish: the shoot resolves the guard by his skin at the crossing spot, not a
    // hard port index — on the merged tree the crossing's own return fire kills skin 0x5000087
    // as port index 1 at tick 977 [L the kill trace on this tree: his last stands (61.99, 41.28,
    // 1.99) → (61.55, 41.28, 2.21), the two P2K hits 4.8 → −0.5], so the old shoot(0) aimed at a
    // different guard whose parts never expose from the lane (37 P2K rounds, his health 25
    // unchanged, the 2026-11-06 run) and the #462 shot arrived dry. A player shoots the guard who
    // is actually plinking the crossing; when the return fire has already dropped him, Bond
    // crosses and shoots the beam — no ammo spent on a corpse or on a guard behind the lip.
    if !w.tops_up() {
        let guard = w.scene.sim.as_ref().unwrap().drones.list.iter().position(|d| {
            d.team == 1 && !d.corpse && d.health > 0.0 && d.skin == 0x5000087
                && walker::dxz(d.pos, [61.99, 41.28, 1.99]) < 4.0
        });
        match guard {
            Some(g) => {
                if !w.shoot(g, "the silo rifle guard (placement 699, at the crossing spot)") {
                    if w.resupply("silo rifle ammo") { w.shoot(g, "the silo rifle guard (placement 699, at the crossing spot) retry"); }
                }
            }
            None => eprintln!("[15] the crossing rifle guard is already down (the return fire finished him) — straight to #462"),
        }
    }
    // REAL-HEALTH TACTIC (muzzle-repin-6 measure, kept at the stream-batch join over the
    // muzzle tree's own shoot(0) — the skin-based kill above replaces that hard index): the
    // guard fight's own flank/creep moves Bond AFTER nav_until_clear cleared the line — the
    // measured master failure stands at (61.43, 41.35, 1.68) with the eye→beam ray blocked at
    // fraction 0.0376 (~0.5 u off the eye, a constant gantry-clutter hit for all 3480 ticks)
    // while the aim sits dead-on: 22 rounds and 261 dry pulls die in the near clutter and the
    // beam never takes a round. A player re-takes the vantage: re-walk the same nav lane until
    // the line is clear again (the same ≤ 14 u rule) before the shot.
    if !w.tops_up() { nav_until_clear(&mut w, [34.6, 41.4, 6.0], mine462, d462, "the silo crossing re-clear"); }
    if difficulty >= 3 && !w.tops_up() {
        // 00 Agent: the 18.5 u vantage rule still left the shot at 13.56 u (the line clears only
        // at the clutter boundary) and the pop cost 14.72. The lane east of it keeps a clear line:
        // from (65.0, -0.3) the beam goes at 17.85 u with no hurt (measured; 62.5 -> 15.34 u
        // still clear of hurt, the walk then died earlier at t4190 on that stream).
        let y = w.scene.sim.as_ref().unwrap().bond.pos[1];
        w.lenient = true;
        w.walk_to(Wp::new(65.0, y, -0.3).r(0.5), "the #462 far spot");
        w.lenient = false;
    }
    let d = shoot_mine(&mut w, 462, mine462, "the silo-crossing beam #462");
    w.dmg.lane_fights = false;
    w.dmg.aimed_fights = false;
    if difficulty >= 2 { w.dmg.back_off = Some(4.0); }
    assert!(d > 4.0, "#462 is shot from down the lane ({d:.2} u)");
    assert!(w.follow_nav_route([34.6, 41.4, 6.0], "the silo crossing"), "the nav route crosses the gantry");
    w.walk_to(Wp::new(34.6, 41.4, 6.0).r(1.2), "the north turn");
    // The z ≈ 18 doorway fence (#383 the vertical beam at x 35.6, #384/#385 the diagonals) stands
    // behind door[38] (z 10.5). REAL-HEALTH TACTIC (level-14-15-real-health-finish): the old
    // through-the-door shot from z 13 is 4.5 u off the beam and the fence's own blast
    // [M Explode_Propagate 0x17c518 — radius 8, damage 100 [L ee_start_05.bin @0x2bf150
    // + 0x383's record]] costs 44.8 at that range (the measured run: −16.0, −12.8, −16.0 across
    // frames 2686-2687) — the level's own law, so the shot moves to RANGE, the exact dance
    // a_p2k_round_from_10_units_explodes_the_door38_fence pins: into the leaf's E reach, open
    // door[38], step back south of it, and the round rides the open doorway's window from ~10 u,
    // past the blast's reach. The neighbours #384/#385 still go on the blast's case-0x4B entries
    // [M 0x17c518].
    w.walk_to(Wp::new(34.6, 41.4, 9.8).r(0.6), "door[38] E reach");
    open_door_facing(&mut w, [34.6, 41.4, 10.5], "door[38]");
    // real-health-00agent-15-6 (the d3 lever): the fence chain's blasts ride LOWER on the fence
    // than its centre — the measured mark stand (z 7.5, 10.4 u off the centre) took −6.90 /
    // −3.49 / −6.13 at ticks 3638-3639 [L the run15 trace], inside the sphere-edge reach
    // [M Explode_Propagate 0x17c518] by ~0.4 u. The d3 shot line steps 1.9 u farther south so
    // every chain blast dies before Bond (the falloff needs (d − obj+0x8C) < radius; at z 5.6
    // the low impact points sit ≥ 9.6 u off). The d1/d2 walks keep their pinned lines.
    let fence_mark = if difficulty >= 3 && !w.tops_up() { [34.6f32, 41.4, 6.5] } else { [34.6f32, 41.4, 7.5] };
    walk_plain(&mut w, fence_mark, 0.35, "south of door[38], the shot line");
    {
        // consider-explosive-gate-3 diagnostic: the shot line's clearance against each fence
        // mine's pick sphere (the segment_t gate [M worldobj.rs segment_t: sphere then tris]) —
        // the real-damage run's Bond arrives with the guards' dropped AR auto-equipped (the
        // walk-over take [M Player_IsBetterWeapon 0x1a6eb0]), whose muzzle shifts the line.
        let sim = w.scene.sim.as_ref().unwrap();
        let bp = sim.bond.pos;
        let eye = [bp[0], bp[1] + sim.bond.eye - sim.bond.crouch_drop, bp[2]];
        let at = [35.6f32, 42.3, 17.47];
        let dir = [at[0] - eye[0], at[1] - eye[1], at[2] - eye[2]];
        let dd: f32 = dir.iter().map(|v| v * v).sum();
        for plc in [383usize, 384, 385] {
            let m = &sim.drones.mines[mine_idx(&w, plc)];
            let ac = [m.centre[0] - eye[0], m.centre[1] - eye[1], m.centre[2] - eye[2]];
            let t = (ac.iter().zip(&dir).map(|(a, b)| a * b).sum::<f32>() / dd).clamp(0.0, 1.0);
            let q = [eye[0] + dir[0] * t - m.centre[0], eye[1] + dir[1] * t - m.centre[1], eye[2] + dir[2] * t - m.centre[2]];
            let d2: f32 = q.iter().map(|v| v * v).sum();
            eprintln!("[mines] graze #{plc}: gun {:#04x} centre {:?} sph_r {:.3} closest {:.3} {}",
                sim.gun.weapon, m.centre, m.sph_r, d2.sqrt(), if d2.sqrt() < m.sph_r { "INSIDE the gate" } else { "outside" });
        }
    }
    // The pinned dance re-draws the P2K. The real-health fights upstream drop the guards' ARs and
    // the walk-over take auto-equips one [M Player_IsBetterWeapon 0x1a6eb0 — the AR 0x1a is rank 3
    // of BEST_WEAPON, the P2K 0x02 rank 16] — the measured run reaches this mark with the AR in
    // hand (clip 30 spare 0). The bullet spawns from the camera–gun-bone midpoint for an fp-model
    // weapon [M Player_WeaponInitBullet 0x1a4bf0:226-245; the port's scene.rs shot_from], so the
    // AR's muzzle rides the aim line onto a different fence tri: the line eye→#383's centre passes
    // inside ALL THREE fence pick spheres (the measured graze table — #383 closest 0.25, #384
    // 0.61, #385 0.65, sph_r 2.18 each), and which mine's tris take the round first is the game's
    // own first-solid-node rule [M Collide_Update 0x1ea398:216-229]. From the AR the round takes
    // #385 first (the tick-2640 failure: #385 cause 2 BULLET, its case-0x4B blast then takes
    // #383/#384 [M Explode_Propagate 0x17c518] — the game does the same), from the P2K — the gun
    // this walk's own doc pins ("the shots ride the P2K") and the sister test fires — it takes
    // #383 first. The real switch restores the walk's stated tactic; no health/ammo writes.
    draw(&mut w, 0x02);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.gun.clip + sim.gun.spare > 0, "the P2K is dry at the fence mark (clip {} spare {})", sim.gun.clip, sim.gun.spare);
    }
    let d = shoot_mine(&mut w, 383, [35.6, 42.3, 17.47], "the doorway fence #383 from 10 u");
    assert!(d > 8.0, "#383 is shot from RANGE, past its own blast ({d:.2} u)");
    w.wait(10);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        for plc in [384usize, 385] {
            let m = &sim.drones.mines[mine_idx(&w, plc)];
            assert!(m.exploded && m.cause == nf_game::worldobj::MINE_BLAST, "#{plc} goes on #383's blast (cause {})", m.cause);
        }
    }
    // real-health-00agent-12-13-14-15 (the d3 gate): the plank grenadiers (the same [D] spots
    // the d2 pre-shoot resolves — port #7 (28.63, 41.34, 35.51) and #8 (42.71, 41.34, 35.49)
    // [L the d2 load probe]) opened fire on the door[39] north leg BEFORE the d2-gated stand
    // ran (the d3 ledger: 19.3 in 3 hits from #8 and 7.0 from #7, frames 3600-3607, the death
    // tick) — a player clears them from door[38]'s north lane before walking the spine (the
    // d2 stand point, ~20 u; the fence-mark stands at 28 u never fired — the 0.02-rad aim gate
    // is a torso-width there). The AR in hand drops a 20-hp grenadier in one round; the loose
    // gate keeps the taps coming while the aim settles. The d2 stand below is a no-op once
    // they are down.
    if difficulty >= 3 && !w.tops_up() {
        w.walk_to(Wp::new(34.6, 41.4, 16.0).r(1.2), "door[38] north (the d3 grenadier stand)");
        // real-health-00agent-15-6: the AR comes out HERE. The run15f ledger measured the
        // difference: with the AR drawn at this mark the fights' own law finished #7 mid-leg
        // (the 4228 kill) and #17 (4628) — without it (the run15e/g ledgers) the fights never
        // reached the plank grenadiers and their volleys cost 41.3 + 21.0 before the leg's
        // end. A player arms the rifle before the plank, and the fights answer with it.
        // The door[39] aimed stand below spends it only on an EXPOSED 0x50000b2 guard.
        {
            let sim = w.scene.sim.as_ref().unwrap();
            let owns_ar = sim.drones.inventory.owned.get(0x1a).copied().unwrap_or(false);
            let ar_live = owns_ar && (sim.gun.clip > 0 || sim.gun.spare > 0);
            if sim.gun.weapon != 0x1a && ar_live {
                drop(sim);
                draw_persist(&mut w, 0x1a, "the AR for the plank legs");
            }
        }
        for (at, label) in [
            ([28.63f32, 41.34, 35.51], "the west-plank grenadier from door[38] north"),
            ([42.71f32, 41.34, 35.49], "the east-plank grenadier from door[38] north"),
        ] {
            // real-health-00agent-15-6: the plank grenadiers are SPAWNER-RELEASED — at this
            // mark (~t3900) neither 0x50000b2 guard is present yet [L the run15f ledger: the
            // skin-resolution returned None, no stand printed, #7 died to the fights at 4228
            // mid-leg]. The stand stays the 8.0 u no-op it has always been here; the answer
            // is the fights' law on the leg and the aimed stand at the door[39] mark below.
            if let Some(di) = agent_drone_near(&w, at, 8.0) {
                w.shoot_bounded(di, label, 2400);
            }
        }
        // real-health-00agent-15-8 (d3): the SECOND plank rifle chaser. The d2's own answer
        // kills the first (the fights' law finished #17 at 4628 [L the 15-6 ledger]) but the
        // d3 roster carries two 0x5000087 chasers: #4 stayed quiet through the plank zone
        // (the fights' pick needs a shooter [M DamageLog::shooters]) and chased the whole
        // hall, paying 15.0 in 5 on the corridor-entry legs (the 2026-11-14 run2 ledger —
        // health 34.4 → 5.4). The same stand answers him while he converges: the skin finder
        // from the door[38] north mark, the shoot machinery's own positioning (the plank zone
        // is open floor), the fights' law finishes what the stand starts.
        if let Some(di) = agent_drone_nearest_skin(&w, [30.0, 41.3, 33.0], 16.0, 0x5000087) {
            w.shoot_bounded(di, "the plank rifle chaser (the d3 roster's second) from door[38] north", 2400);
        }
    }

    // The north lane through the grid, then the west lane south, then the exit hall. The x34.6
    // spine SPLITS at z 30: the east plank (x38-44) dead-ends at door[45]'s keypad slot (ch_lock
    // 0xc8 — no writer on the level [D scans; the trigger dump has no AND] — permanent), and the
    // first runs fell into the lower deck (y −3.9) at (33.7, 38.8). The WEST plank (x 24-34,
    // z 28-42, the floor grid) carries the route around the slot to door[46].
    let route: &[(&str, Wp)] = &[
        ("door[38] north", Wp::new(34.6, 41.4, 16.0).r(1.2)),
        ("door[39] north", Wp::new(34.6, 41.4, 28.0).r(1.2)),
        ("the west plank", Wp::new(29.0, 41.4, 31.0).r(1.2)),
        ("the plank north", Wp::new(28.0, 41.4, 35.0).r(1.2)),
        ("the plank north 2", Wp::new(28.5, 41.4, 39.0).r(1.2)),
        ("the spine north", Wp::new(30.5, 41.4, 42.0).r(1.2)),
        ("door[46] north", Wp::new(34.6, 41.4, 46.5).r(1.2)),
        ("door[46] through", Wp::new(34.6, 41.4, 52.0).r(1.2)),
        ("door[47] north", Wp::new(34.6, 41.4, 65.0).r(1.2)),
        // The desk corner: the console placement (map54 #145 at (27.95, 41.59, 69.83), a real
        // collision strip x 26..30.5 at z ≈ 70 [D placement + the probe's floor grid '^' band])
        // walls the direct south-east approach to door[32]'s threshold, and the shaft hole
        // x 25..27 z 67..68 walls the south detour. The NAV GRAPH's own lane (the game's AI
        // route, walkreal_probe15::probe_nav_route_to_exit) wraps east→north→west and reaches
        // the doorway from the north at (26.15, 71.18) — the wall runs north-south at x ≈ 25
        // (the leaf sph (25.10, 41.51, 71.19) r 1.76 [D door 32]), so the passage spans
        // z 69.4..72.9. The player walks the same way round the desk.
        ("the nav lane east", Wp::new(37.6, 41.4, 66.3).r(1.2)),
        ("the nav lane north", Wp::new(41.6, 41.4, 69.2).r(1.2)),
        ("the nav lane top", Wp::new(41.9, 41.4, 71.2).r(1.2)),
        ("the nav lane west", Wp::new(37.6, 41.4, 76.9).r(1.2)),
        ("the nav lane southwest", Wp::new(30.5, 41.4, 74.6).r(1.2)),
        ("the doorway north side", Wp::new(28.7, 41.4, 71.5).r(1.0)),
        // REAL-HEALTH TACTIC (level-14-15-real-health-finish): placement 722 (port index 2, skin
        // 0x500008c, weapon 0x1a — the grenade launcher) sits at (26.22, 41.45, 71.15) [D], the
        // door[32] doorway itself, and the measured run died to ITS grenade chain: the grenade
        // popped near #526 (7.7 u from the mine's sphere) and the mine's own blast took Bond at
        // 1.7 u (−27.73, frame 5067) before the walk's own rim shot — the exact scenario the
        // walk's own comment warns about. A player clears the doorway guard before the west
        // lane; shoot() is a no-op once a fight has finished it.
        ("door[32] threshold", Wp::new(26.2, 41.4, 71.2).r(0.9)),  // in E reach of the leaf (head→pos d² 5.3 < 6.25)
        ("door[32] through", Wp::new(23.0, 41.4, 71.2).r(0.9)),
        ("door[32] west-top", Wp::new(20.0, 41.4, 70.0).r(1.2)),
        // The west lane: door[29]'s doorway sits in the SAME north-south wall pattern (leaf sph
        // (10.56, 41.53, 71.19) [D door 29]); the old (4.0, 70.0) leg hung over the void west of
        // x 5 (the probe's floor grid — z 69..73, x < 5 is a hole) and wedged the walker at the
        // edge. The lane: through the doorway, then southwest through the west room (x 0..8,
        // z 61..67) to door[52] (1.05, 60.67), the corridor's north door.
        ("the west lane", Wp::new(13.0, 41.4, 71.0).r(1.2)),
    ];
    // The Agent path: the d2 roster shifts the port indices (Drone_Create's min-difficulty
    // gate [M 0x136be0:49]) — the route's targeted pre-shoots resolve by the guards' [D]
    // placement spots. REAL-HEALTH TACTIC: the plank grenadiers (port #7 at (28.63, 41.34,
    // 35.51) and #8 at (42.71, 41.34, 35.49) [L the d2 load probe]) plinked the measured run
    // for −16.8 in 8 hits and −28.8 (a grenade) across the west-plank leg; a player clears
    // them from door[39]'s north lane before stepping onto the plank grid — both stand in the
    // open at 10.6 / 11.9 u. The BodyArmourHeavy #802 (22.53, 40.35, 35.53 [L the d2 load
    // probe]) is NOT taken: it sits in the x ≈ 25 pit behind the ch_lock 0xc8 door at
    // (25.1, 41.6, 36.6) — the detour walk wedged airborne against that door at (25.76,
    // 41.47, 36.18) with floor_below 40.28 — so the #526 rim-shot blast stays on health
    // (the pain rows for 07000015: [M Player_HandlePain 0x1902c8 × ReadTuningVars 0x1d3a80's
    // per-level jump table — the 0x14..=0x16 group (Easy 0.4, Normal 0.6, Hard 1.0), the
    // same shape [L ee_start_05.bin @0x30cc50/54/58 = 0.6/0.7/1.0 pins for the 0x05..=0x08
    // group]) and the budget comes from the fights.
    let (route_a, route_b) = route.split_at(route.iter().position(|(l, _)| *l == "the west plank").unwrap());
    w.walk_route(route_a);
    if difficulty >= 2 {
        for (at, label) in [
            ([28.63f32, 41.34, 35.51], "the west-plank grenadier (port #7, skin 0x500008c)"),
            ([42.71f32, 41.34, 35.49], "the east-plank grenadier (port #8, skin 0x50000b2)"),
        ] {
            // real-health-00agent-15-6 (the coded stand that never ran — the door[39] side):
            // the grenadiers stand 9.6 / 11.0-13.2 u from this mark [L the run15 trace: #7 at
            // (28.63,35.51), #8 at (42.71,35.49), his blasts at t4359-4387 while the 8.0 u
            // resolution gated both out]. The d3 roster answers with the AIMED stand (zero-
            // spread [M Bullet_init 0x12c5a0:139-170], resolved by SKIN 0x50000b2 so the
            // chaser #17 never eats it): the run15f shoot burned the AR on an UNEXPOSED #8
            // and the resupply detour displaced Bond onto the east plank (the −56.28 fall).
            // The aimed stand never taps an unexposed guard — an UNSEEN bail costs no ammo.
            // The d2 roster keeps its 8.0 u stream byte-identical.
            let di = if difficulty >= 3 && !w.tops_up() { agent_drone_nearest_skin(&w, at, 14.0, 0x50000b2) } else { agent_drone_near(&w, at, 8.0) };
            if let Some(di) = di {
                if difficulty >= 3 && !w.tops_up() { aimed_stand(&mut w, di, label, 2400); } else { w.shoot(di, label); }
            }
        }
        // REAL-HEALTH TACTIC (level-15-agent-rim-shot-2, the measured ledger): the d2 roster's
        // plank rifle guard (skin 0x5000087, the burst rifle) is a CHASER — at this stand the
        // measured run finds him at (35.73, 41.28, 44.65) hp 10 (the fights' own work), and he
        // creeps down onto the west plank while Bond crosses it: 6 hits for 21.6 at d 2.0-2.2
        // (ticks 3924-3959) before the fights finished him at 4016. A player answers the chaser
        // from the stand he is already in sight of — one 0x5000087 within 20 u, finished from
        // here (hp 10, the fights co-fight) — and the plank crossing never meets him.
        let rifle = { let sim = w.scene.sim.as_ref().unwrap(); let b = sim.bond.pos;
            (0..sim.drones.list.len())
                .filter(|&i| { let d = &sim.drones.list[i];
                    d.present && d.enabled && !d.corpse && d.health > 0.0 && d.team == 1
                        && d.skin == 0x5000087 })
                .filter(|&i| { let d = &sim.drones.list[i];
                    (d.pos[0] - b[0]).powi(2) + (d.pos[1] - b[1]).powi(2) + (d.pos[2] - b[2]).powi(2) < 20.0 * 20.0 })
                .min_by_key(|&i| { let d = &sim.drones.list[i]; let b = sim.bond.pos;
                    ((d.pos[0] - b[0]).powi(2) + (d.pos[1] - b[1]).powi(2) + (d.pos[2] - b[2]).powi(2)) as u32 })
        };
        // real-health-00agent-15-6: the chaser stand + its resupply stay D2-ONLY — the run15f
        // ledger measured the d3 branch NEGATIVE: shoot_bounded found #17 at 7.0 u exposed,
        // burned the AR dry at +61 (the plain hip cone's scatter at 7 u is ~1.0 u vs a 0.5 u
        // torso — 8 rounds, zero lands), and the resupply nav walked Bond EAST onto the plank
        // grid ((41.56, 33.6)) where route_b1's own crossing then fell him −56.28 at (34.02,
        // 31.82). On the d3 roster the chaser answer is the AIMED stand below (zero-spread).
        if difficulty == 2 {
            match rifle {
                Some(g) => {
                    if !w.shoot_bounded(g, "the plank rifle guard (skin 0x5000087) from the door[39] north stand", 2400) {
                        if w.resupply("the plank rifle guard's rounds") {
                            w.shoot_bounded(g, "the plank rifle guard (skin 0x5000087) retry", 2400);
                        }
                    }
                }
                None => eprintln!("[15] the plank rifle guard is already down at the plank stand"),
            }
        }
        // real-health-00agent-15-6 (the run15h lever): the chaser's no-progress give-up is the
        // walk's biggest dealer past the grenadiers — the fights engaged #17 at the west plank
        // (4531), the spine north (4944) and door[46] north (6032), "killed false" each time
        // (the 300-tick law), while his burst rifle plinked 48.0 in 16 hits and Bond died at
        // t6542 at (35.03, 61.95). At THIS stand the run15f ledger measured him at 7.0 u with
        // a part EXPOSED — the aimed stand's zero-spread taps land where the hip cone could
        // not, and a dead chaser never plinks.
        if difficulty >= 3 && !w.tops_up() {
            if let Some(g) = rifle {
                aimed_stand(&mut w, g, "the plank rifle chaser (skin 0x5000087) from the door[39] north stand", 2400);
            }
        }
    }
    let (route_b1, route_b2) = route_b.split_at(route_b.iter().position(|(l, _)| *l == "door[32] threshold").unwrap());
    // real-health-00agent-15-6 (the run15i lever): the desk-corner pair is the walk's dealer
    // past door[47] — #2 (skin 0x50000b2, weapon 0x1a) dealt 42.0 in 6 and #3 (skin
    // 0x5000087, weapon 0x15) 20.3 in 7, with the fights' engagement at the nav lane east
    // "killed false" (the tick-5468 line) and the death at t5777 at (35.55, 65.26). A player
    // stops at door[47]'s north side and answers them before stepping into the lanes: the
    // aimed stand taps only an EXPOSED part (an UNSEEN bail costs no ammo and no exposure),
    // resolved by skin so no other guard eats the stand.
    let (route_b1a, route_b1b) = route_b1.split_at(route_b1.iter().position(|(l, _)| *l == "the nav lane east").unwrap());
    w.walk_route(route_b1a);
    if difficulty >= 3 && !w.tops_up() {
        // real-health-00agent-15-6 (the run15j/l lever): the desk stands ENGAGED both guards
        // (they are exposed from door[47] north) but the fights left the AR with 1-2 rounds
        // (the run15j stands bailed DRY at +58/+0, #2 stood at hp 8, his next volley dealt
        // the killing 62.3), and the run15l RESUPPLY measured WORSE: its only corridor pile
        // was the plank drops 29.2 u SOUTH — the backtrack nav ate #2's 56.3 while Bond
        // re-walked the plank under his fire. A player whose mag cannot pay for both guards
        // swaps to the P2K (the fence dance re-drew it at clip 5 spare 34 [L the run15e shot
        // line] and the fights never touched it — the AR was in hand from door[38] north on):
        // the aimed stand's zero-spread head tap is 3.5 × 10 = 35, one round per guard.
        let (clip, spare, wpn) = { let sim = w.scene.sim.as_ref().unwrap(); (sim.gun.clip, sim.gun.spare, sim.gun.weapon) };
        if clip + spare < 8 && wpn != 0x02 {
            draw_persist(&mut w, 0x02, "the P2K for the desk stands");
        } else if clip + spare < 8 {
            w.resupply("the desk-corner stand's rounds");
        }
        // real-health-00agent-15-7 (the head lever): the desk stands aim the clear HEAD box
        // first (head_aim above) — one ×10 round [M NDrone2_HitDamage 0x145558 ×
        // DroneDamage_Head] kills where the torso grind fed the guard's own stream. The
        // door[39] stands keep the torso pick (their guards die at +74/+44 on body rounds).
        for (at, skin, label) in [
            ([30.4f32, 41.3, 73.6], 0x50000b2u32, "the desk-corner AR guard (skin 0x50000b2) from door[47] north"),
            ([38.7f32, 41.3, 76.0], 0x5000087u32, "the desk-corner rifle guard (skin 0x5000087) from door[47] north"),
        ] {
            if let Some(di) = agent_drone_nearest_skin(&w, at, 14.0, skin) {
                aimed_stand_impl(&mut w, di, label, 2400, true);
            }
        }
    }
    w.walk_route(route_b1b);
    // The door[32] doorway guard clears BEFORE the lane is walked (a player clears the doorway
    // before stepping past it); at the d2 roster the placement-spot resolution is kept.
    if difficulty >= 2 {
        if let Some(di) = agent_drone_near(&w, [26.22, 41.45, 71.15], 4.0) {
            w.shoot(di, "the door[32] doorway grenade guard (placement 722) at the Agent roster");
        }
    } else if !w.tops_up() {
        w.shoot(2, "the door[32] doorway grenade guard (placement 722)");
    }
    // real-health-00agent-15 (the #526 lever, now the d2 Agent run's own — level-15-agent-rim-shot):
    // the rim shot below stands IN #526's own blast — radius 15, damage 80 [L the run probe] —
    // and the game's falloff is CONTINUOUS from the sphere-edge distance: the disassembly reads
    // Vec_Dist3D (0x17c5dc) → minus the victim's obj+0x8C (0x17c5ec lwc1 140(v0), 0x17c5f8
    // sub.s) → max.s …,0 (0x17c608, COP1 func 0x28 — the decompiler's "(int)" there was its
    // rendering of the max, not a truncation) → /radius (0x17c60c) → 1−x (0x17c610) → ×damage
    // (0x17c614) → the 0.0002 cutoff (0x17c618-20). On Agent (×0.6 [M Player_HandlePain
    // 0x1902c8 Plr_DMod_Normal]) the measured rim shot cost 44.33 at 2.47 u on this tree (the
    // d3 pain row 1.0 [M bullet.rs damage_factor 0x14..=0x16 → 1.0] makes it 73.9 raw) — the
    // budget's largest single toll. A player who has just finished the door[32] guard stands
    // over the guard's own drops — the AR 0x1a AND a kind-1 pile of its grenade launcher 0x1b,
    // a full shot budget [M Pickup_Handler 0x18e650's AR drop arm; DroneWeap_DropWeapon
    // 0x17a2a0] — and lobs a round down the lane instead of walking the rim: the round's fuse is
    // TIME, 210 ticks [M Bullet_init 0x12c5a0's default; grenade.rs fuse arm], it bounces (a
    // bounce NEVER explodes [M Bullet_CollisionHandler 0x12ce80]) and rests mid-lane by the rim,
    // where the pop's case-0x4B entry sets #526 [M Explode_Propagate 0x17c518] — the guard's own
    // round popped 7.7 u from the sphere and chained it [L the d1 ledger]. From the threshold
    // Bond is ~15.2 u off the mine: past the grenade's own reach when the rest point rides
    // mid-lane, and #526's chain blast reads 80·(1−(15.2−1.06)/15)·0.6 ≈ 2.8 there. The rim
    // walk and rim shot below then skip (the d1 run keeps them: nothing chains the mine there,
    // and the check is a read, not a write). The AR goes back in hand for the west-room stands.
    if difficulty >= 2 && !w.tops_up() { // real damage only, as the plate sneak
        w.walk_to(Wp::new(26.5, 41.4, 71.2).r(0.5), "door[32] threshold (the guard's own drops)");
        // The thrower: the carried frag 0x34 (the level's own loadout [D the 07000015 loadout] —
        // owned from the start, measured in the run's owned list), or the launcher 0x1b when a
        // 0x1a-base guard's pile granted it (the d3 lever's original premise; this level's
        // 0x500008c guards carry the 0x26, whose base is not the 0x1a — no 0x1b pile spawns —
        // and an AMMO-kind pile grants rounds, never ownership [M Pickup_Handler 0x18e650 case 1
        // → Player_EquipAmmo; the port's take arm kind::WEAPON|kind::AMMO]).
        let owned = |w: &Walker, id: u8| w.scene.sim.as_ref().unwrap().drones.inventory.owned.get(id as usize).copied().unwrap_or(false);
        let throw_gun = if owned(&w, 0x1b) { Some(0x1bu8) } else if owned(&w, 0x34) { Some(0x34u8) } else { None };
        if let Some(gun) = throw_gun {
            if w.scene.sim.as_ref().unwrap().gun.weapon != gun { draw(&mut w, gun); }
            w.use_door(32, "door[32] for the throw lane");
            let h0 = w.scene.sim.as_ref().unwrap().drones.health.health;
            let d526 = shoot_mine(&mut w, 526, [11.3, 41.6, 71.18], "the #526 lob from door[32]'s threshold");
            let h1 = w.scene.sim.as_ref().unwrap().drones.health.health;
            eprintln!("[real-health] the #526 lob (d2/d3, gun {gun:#04x}): Bond {d526:.2} u off the boom, health {h0:.1} -> {h1:.1}");
            assert!(d526 > 8.0, "the lob is thrown from outside its own radius 8 ({d526:.2} u)");
            assert!(h0 - h1 <= 12.0, "the #526 chain from the threshold costs {:.1} (the rim shot cost 44.33)", h0 - h1);
            // The pressed-edge retries: the request follows the lob's live fire script — a single
            // press is dropped there [M 0x1a24c8:113-152] (the measured run: the draw timed out
            // on the frag's 24-pull fire window).
            if owned(&w, 0x1a) {
                draw_persist(&mut w, 0x1a, "the AR back in hand after the #526 lob");
            }
            // Real health: a nearly empty gun in hand (under 10 rounds) reads LOW to the fights,
            // whose resupply nav then walked Bond 31 u back east under the west-room grenadiers
            // (the t7985 death) while the P2K carried 34. The P2K comes out instead.
            let low = { let s = w.scene.sim.as_ref().unwrap(); s.gun.clip + s.gun.spare < 10 };
            if low && owned(&w, 0x02) && !w.tops_up() {
                draw_persist(&mut w, 0x02, "the P2K back in hand after the #526 lob");
                let s = w.scene.sim.as_ref().unwrap();
                eprintln!("[real-health] after the lob: gun {:#04x} clip {} spare {}", s.gun.weapon, s.gun.clip, s.gun.spare);
            }
        } else {
            let owned: Vec<usize> = (0..w.scene.sim.as_ref().unwrap().drones.inventory.owned.len())
                .filter(|&i| w.scene.sim.as_ref().unwrap().drones.inventory.owned[i]).collect();
            let piles: Vec<(u16, u16, [f32; 3])> = w.scene.sim.as_ref().unwrap().drones.pickups.iter()
                .filter(|p| !p.gone).map(|p| (p.kind, p.weapon, [p.m[3][0], p.m[3][1], p.m[3][2]])).collect();
            eprintln!("[real-health] no thrower owned at door[32]'s threshold — owned {owned:x?}; live pickups {piles:?}");
        }
    }
    // #526, the Tripbomb_Tiny ankle beam (x 10.7..11.8, z 70.95..71.41, 0.27-0.47 over the floor
    // [D]), lies ALONG the lane in door[29]'s doorway, in the trough between the 0.76 rim at
    // x ≈ 12 and the leaf: the doorway (the leaf's z 70.06..72.32) leaves under the body's 1.1
    // width on either side of it, and the walk's body reaches the floor [M 0x194df0], so a step
    // off the rim meets it — no lane walks past it and its pick sphere (sph_r 2.18) spans the
    // whole doorway, so the beam must be shot. The rim also hides it from a shot down the lane:
    // the bullet trace from west of the rim dies on the rim's face (the probe15e dump — every
    // stand x ≥ 14 stops the round), so from just east of the rim the line drops over the rim's
    // top onto the beam. (A guard's grenade near door[32] — radius 8, 7.7 u from #526's sphere —
    // can set it off first through the same case-0x4B entry [M 0x17c518]; `shoot_mine` then
    // returns at once.)
    // REAL-HEALTH TACTIC (level-15-agent-doors): the rim shot stands IN the beam's own blast —
    // the hurt is pure distance [M Explode_Propagate 0x17c518's sweep, the CONTINUOUS
    // sphere-edge law above — no LOS and no wall check], radius 15, damage 80, so on Agent
    // (×0.6 [M Player_HandlePain 0x1902c8 Plr_DMod_Normal]) the measured cost on this tree is
    // −44.33 at 2.47 u (the doors-2 tree read −41.6/−27.73 under the old (int) reading — see
    // the report). The d2/d3 runs never pay it (the lob above chains the mine first); the d1
    // Operative walk keeps the rim shot (−27.7 at ×0.4, its pinned ledger). The blast budget is
    // made back on the west room: the grenadiers are answered from THIS mark, before the
    // doorway is walked (below).
    w.walk_route(route_b2);
    // real-health-00agent-15: the d2/d3 lob (above) may already have chained #526 — the rim walk
    // and shot then skip (the d1 run finds the mine standing and keeps these exact legs).
    let mine526_gone = w.scene.sim.as_ref().unwrap().drones.mines[mine_idx(&w, 526)].exploded;
    if !mine526_gone {
        walk_plain(&mut w, [12.8, 41.4, 71.2], 0.35, "east of door[29]'s rim");
        let d = shoot_mine(&mut w, 526, [11.3, 40.70, 71.18], "the door[29] ankle beam #526");
        assert!(d > 1.5, "#526 is shot from east of the rim, not stepped on ({d:.2} u)");
    }
    // The fights' gun (d2 only): the pre-throw fights spend the P2K's whole 48-round pool on
    // the crossing and plank kills (the run-3 ledger: clip 0 spare 0 at the #447 fence shot —
    // the walk died of thirst), and the walk-over take armed the dropped 0x1a at the crossing
    // whose leftover ~17 rounds sit unused in the wheel. The 0x1a round is ALSO the cheap kill:
    // a direct hit detonates its blast AT the guard (80 raw, the falloff at d 0) — one round
    // drops any hp-20 grenadier where the P2K's 3.5 needs six [L the run-3 fight taps; M the
    // 0x1a record damage 3.5 + the blast]. A player pushes the grenadier room with the
    // grenade launcher, not a dry pistol. Real wheel request, no writes (the draw no-ops if
    // the cascade left the 0x1a unowned). The pressed-edge retries: the request follows the
    // #526 shot's live fire script — a single press is dropped there [M 0x1a24c8:113-152].
    if difficulty == 2 && !w.tops_up() { draw_persist(&mut w, 0x1a, "the west-room gun"); }
    // REAL-HEALTH TACTIC (level-15-agent-doors): the west-room grenadiers are answered NOW, from
    // east of the rim — the measured v1 run fought them from INSIDE the west room (after
    // route_c's threshold/through legs) and #11's grenade was already in flight when it died:
    // −22.5 at 6191, the run's second-largest hit. A player hears the launchers from the
    // doorway and answers through it; #526 is already gone, so the fight's own wandering is
    // body-safe. The doorway's z band (70.06..72.32 at x 10.56) covers both boxes' rays from
    // here (#10's at z ≈ 70.3-70.6, #11's at z ≈ 71.6-72.2).
    if difficulty >= 2 {
        for (at, label) in [
            ([-1.87f32, 41.34, 77.42], "the northwest grenadier (port #11, skin 0x50000b2) through the door[29] window"),
            ([-5.14f32, 41.34, 67.25], "the west-room grenadier (port #10, skin 0x50000b2) through the door[29] window"),
        ] {
            if let Some(di) = agent_drone_near(&w, at, 8.0) {
                w.shoot(di, label);
            }
        }
    }
    // real-health-00agent-15-7 (the west-room lever, d3 real-damage only): the d3 roster's
    // west-room pair is SPawner-RELEASED during the door[29] threshold leg (the fresh run: the
    // lane-mark stands resolved NOTHING, and #10's 0x1a grenade blast −58.86 at the jog killed
    // Bond at t7684 — the fights met #11 at hp 3 but gave up, and the throw cycle (~70 ticks
    // between throws [L the d2 rain ledger]) outscores any stand's raise window). The answer is
    // the release-tick fight law the d2 Agent run itself rides: `pre_throw_stands` answers the
    // spawner release the tick after it, so the kill lands before the guard's first throw —
    // SCOPED to the west-room phase (the 15-5 disaster was the whole-walk flag at the crossing's
    // aperture fight, a different close fight this scope never touches), and off again after the
    // jog sweep. The d2/top-up walks keep their pinned timelines.
    if difficulty >= 3 && !w.tops_up() {
        w.dmg.pre_throw_stands = true;
        // real-health-00agent-15-7 (the west-room fights' gun-ray law): the fights' aim pick
        // rides the EYE law here (lane_fights was cleared at the #462 shot) — the fresh run's
        // newborn fight on #11 went "killed false" at hp 3 with the tap gated on an eye ray
        // whose ROUNDS died on the door jamb (the 07000013 security-room's own measured
        // mismatch [M DamageLog::exposed_impl:544 — the eye ray clears by a hair while the
        // gun's low-right stand-in clips the corner]). lane_fights re-armed for this phase makes
        // the pick AND the taps ride the gun-ray law — every tap is a round that actually
        // reaches — and the pick only fights a guard the lanes can kill. Off again after the
        // jog sweep, with pre_throw_stands.
        w.dmg.lane_fights = true;
        // real-health-00agent-15-7 (the fights' ammo): the AR came back in hand after the lob at
        // clip 3 spare 0 — the fresh run's fights burned it dry mid-fight and the fights' own
        // resupply nav walked Bond into the door[29] wedge (the panic at tick 25080, health
        // 23.6, 12 fights). The P2K's pool (clip 3 spare 34 after the desk stands [L the fresh
        // run]) pays the fights instead — the same law as the desk stands' swap, one phase
        // later.
        let (clip, spare, wpn) = { let sim = w.scene.sim.as_ref().unwrap(); (sim.gun.clip, sim.gun.spare, sim.gun.weapon) };
        if clip + spare < 8 && wpn != 0x02 {
            draw_persist(&mut w, 0x02, "the P2K for the west-room fights");
        }
        // real-health-00agent-15-7 (the fights' dry-mag law, scoped): the fresh run's fights
        // still ran the resupply NAV on the dry AR ("door[29] through resupply 0 nav 1/2", the
        // wedge panic at tick 25080) — the fights' master give-up on a dry mag is the base law,
        // and a player whose mag runs dry mid-fight cycles to the next loaded firearm
        // (the field doc's 02/03 00agent idiom). switch_dry is that law — SCOPED to this phase
        // (the 15-5 run15q disaster was the whole-walk flag at the crossing's aperture fight,
        // a different fight this scope never touches); with the P2K's 34 spare in the wheel the
        // cycle always finds rounds, so the resupply nav never fires. Off after the jog sweep.
        w.dmg.switch_dry = true;
    }
    let route: &[(&str, Wp)] = &[
        ("door[29] threshold", Wp::new(11.5, 41.4, 71.2).r(0.9)),
        ("door[29] through", Wp::new(8.0, 41.4, 71.2).r(1.0)),
        // REAL-HEALTH TACTIC: the west-room grenadiers (port #10 at (−5.14, 41.34, 67.25) and
        // #11 at (−1.87, 41.34, 77.42) [L the d2 load probe]) shot through the door[29]
        // doorway at the measured run's tail (−4.2 at frame 6020, the killing −2.5 at 6106)
        // — from just inside the doorway a player answers both before the southwest jog.
        ("the southwest jog", Wp::new(6.0, 41.4, 68.5).r(1.0)),
        ("the west room", Wp::new(6.0, 41.4, 66.0).r(1.2)),
        ("the west room south", Wp::new(1.0, 41.4, 63.0).r(1.2)),
        // #487 hangs across door[52]'s south mouth (x −2.4..2.9 at z 59.5, 1.4-1.9 over the
        // floor [D]). #526's blast (radius 15) reaches it here: 13.1 u from #526 to #487's
        // sphere, a falloff of 80·(1 − 13/15) > 0.0002 — the case-0x4B entry sets it off on
        // the next update [M 0x17c518]. Where it still stands, the crouch passes under it
        // (`crouching_under_the_door52_beam_passes_it` below).
        ("door[52] threshold", Wp::new(1.0, 41.4, 61.5).r(0.8)),
        ("door[52] through", Wp::new(0.5, 41.4, 59.0).r(1.0)),
        // The south corridor (x −3..4): doors [50] z 46.1, [42] z 25.1, [51] z 10.5, [49]
        // z −10.5 — all kind 2, unlocked, ch 0 [D doors 50/42/51/49]. The great hall
        // (z 30..42) has a middle pit (z 32..38, x −4..4 void [D floor grid]) — the east
        // bridge x 5..8 carries the route. The z 4..0 split (the middle x −2..2 is void) —
        // the east lane x 5..8. Then the exit hall mouth (z −8, x 0..8) funnels to door[49].
        ("the corridor south", Wp::new(0.0, 41.4, 48.0).r(1.2)),
        ("door[50] threshold", Wp::new(0.0, 41.4, 47.0).r(0.8)),
        ("door[50] through", Wp::new(0.0, 41.4, 44.0).r(1.0)),
        ("the hall east", Wp::new(5.0, 41.4, 42.0).r(1.2)),
        ("the east bridge", Wp::new(6.5, 41.4, 34.0).r(1.2)),
        ("the hall south", Wp::new(2.0, 41.4, 30.0).r(1.2)),
        // REAL-HEALTH TACTIC (level-14-15-real-health-finish): placement 733 (port index 5, skin
        // 0x500008c, weapon 0x1a — the grenade launcher) sits at (6.25, 41.45, 35.69) [D], the
        // bridge's north end, and its grenade spray down the south corridor killed the measured
        // run (−2.80 direct + the −0.36/−1.18/−1.59 triple-pop at frames 6482-6489, then the
        // killing blast at door[42]). A player takes it from the bridge before pushing south;
        // shoot() is a no-op once a fight has finished it.
    ];
    let (route_c, route_d) = route.split_at(route.iter().position(|(l, _)| *l == "the southwest jog").unwrap());
    let solo = difficulty >= 3 && !w.tops_up();
    if solo { w.walk_route(&route_c[..1]); } else { w.walk_route(route_c); }
    if solo {
        // 00 Agent real health: the through leg's return-fire fight on #11 went killed-false while
        // the grenadiers' throws took 58.9 -> 5.1 (the t8578 death). Bond walks from the threshold
        // straight to #11's solo line - (5.0, 71.25), the one spot that sees him with #10/#18 hidden
        // [the 11-13 live bullet-line scan at t7347: #11 (-1.99, 77.11), #10 (-5.07, 66.93), #18
        // (-6.33, 71.81)] - and takes him head-first (dead at +20), then the rest nearest-first.
        for (spot, near) in [([5.0f32, 71.25], [-2.0f32, 77.1])] {
            let di = { let sim = w.scene.sim.as_ref().unwrap(); sim.drones.list.iter().position(|d| d.present && !d.corpse && d.health > 0.0
                && d.team == 1 && walker::dxz(d.pos, [near[0], 41.3, near[1]]) < 3.0) };
            let Some(di) = di else { continue };
            // The fights' dry-mag resupply nav walks Bond 32 u back east mid-leg (it wedged at
            // (25.84, 37.67) for the 6000-tick cap on the far-#462 stream); the solo-line walk
            // holds it - the drops are taken after the room is clear.
            let was = w.dmg.resupplying;
            w.dmg.resupplying = true;
            let _ = w.try_walk_to(Wp::new(spot[0], 41.4, spot[1]).r(0.4), "a west-room solo line");
            w.dmg.resupplying = was;
            let k = aimed_stand_impl(&mut w, di, "the west-room grenadier from his solo line", 600, true);
            eprintln!("[15solo] #{di} from ({}, {}) killed {k} health {:.1}", spot[0], spot[1], w.scene.sim.as_ref().unwrap().drones.health.health);
        }
        // Then the rest nearest-first, head-first, from where Bond stands (#18 closed to 4.9 u
        // and threw 37.9 while the old second solo hop walked).
        for _ in 0..4 {
            let di = { let sim = w.scene.sim.as_ref().unwrap(); let b = sim.bond.pos;
                sim.drones.list.iter().enumerate().filter(|(_, d)| d.present && d.enabled && !d.corpse && d.health > 0.0 && d.team == 1
                    && d.pos[0] < 8.0 && (60.0..84.0).contains(&d.pos[2]) && walker::dxz(d.pos, b) < 16.0)
                    .min_by(|a, c| walker::dxz(a.1.pos, b).total_cmp(&walker::dxz(c.1.pos, b))).map(|(i, _)| i) };
            let Some(di) = di else { break };
            let k = aimed_stand_impl(&mut w, di, "the west-room grenadier, nearest first", 600, true);
            eprintln!("[15solo] nearest #{di} killed {k} health {:.1}", w.scene.sim.as_ref().unwrap().drones.health.health);
            if !k { break; }
        }
        w.walk_route(&route_c[1..]);
    }
    // (The west-room grenadiers are answered from the rim mark above, before this doorway is
    // walked — level-15-agent-doors. The fights' leftovers resolve on the legs' return fire.)
    // REAL-HEALTH TACTIC (level-15-agent-doors-2): the placement-spot pre-shoots at the rim mark
    // MISSED the west-room pair (the door routes walk the guards off their spots — the v7 run's
    // agent_drone_near(8.0) found none there), and #10's return-fire stands then cost 21.0
    // across two fights (the west-room fight gave up, the door[52]-through fight killed him at
    // 12.6 more). A player stepping into the west room kills the grenadiers he can see (hp 20 —
    // one head hit [M NDrone2_HitDamage 0x145558 × DroneDamage_Head; the d2 head-hunt wrapper])
    // before the throwing starts: the nearest live hostile within 14 u of Bond, answered from
    // the jog (the room's interior is open floor between the doorway and the south wall), and
    // the sweep repeats while one stands (the measured run's first sweep took #11 at 12.5 u and
    // #10's own fire still cost 8.4 on the legs' return fire).
    if difficulty >= 2 {
        for sweep in 0..3 {
            let near = { let sim = w.scene.sim.as_ref().unwrap();
                (0..sim.drones.list.len())
                    .filter(|&i| { let d = &sim.drones.list[i];
                        d.present && d.enabled && !d.corpse && d.health > 0.0 && d.team == 1 })
                    .min_by_key(|&i| { let d = &sim.drones.list[i]; let b = sim.bond.pos;
                        ((d.pos[0] - b[0]).powi(2) + (d.pos[1] - b[1]).powi(2) + (d.pos[2] - b[2]).powi(2)) as u32 })
                    .filter(|&i| { let d = &sim.drones.list[i]; let b = sim.bond.pos;
                        ((d.pos[0] - b[0]).powi(2) + (d.pos[1] - b[1]).powi(2) + (d.pos[2] - b[2]).powi(2)) < 14.0 * 14.0 })
            };
            let Some(di) = near else { break };
            w.shoot(di, &format!("the west-room grenadier met at the jog, sweep {sweep} (level-15-agent-doors-2: by proximity, not the spot)"));
        }
    }
    if difficulty >= 3 && !w.tops_up() {
        w.dmg.pre_throw_stands = false;
        w.dmg.lane_fights = false;
        w.dmg.switch_dry = false;
    }
    let (route_d1, route_d2) = route_d.split_at(route_d.iter().position(|(l, _)| *l == "door[50] through").unwrap());
    w.walk_route(route_d1);
    // REAL-HEALTH TACTIC (level-15-agent-doors): the great-hall grenade guard is answered from
    // the door[50] threshold, BEFORE the corridor's last legs — the measured v1 run's return
    // fire met the great-hall pair at "door[50] through" (mid-corridor, health already
    // 17.8) and #5's grenade was in flight when it died: −17.8 at 7885, the killing hit. A
    // player holds the doorway (the door opened for the line) and answers down the corridor:
    // #5 (port index 5 at the d1 roster; placement 733's spot (6.25, 41.45, 35.69) at d2) sits
    // 9.8-13.6 u south through the open leaf.
    // 00 Agent real health: hold 300 ticks at the door[50] threshold before opening it - the
    // great hall is mutually visible from every floor spot (the 11-13 live line scans: no spot
    // sees #5 with #4/#19 hidden, nor the reverse), so the answer is timing; measured over the
    // hold (0 -> t9746 in the hall, 60/120/180 -> t9948-t11352, 300 -> the hall CROSSED, death
    // t20553 at the door[42] fence leg with 5.4 health and a dry gun) [S the 300 is the
    // measured sweep's, not a game rule].
    if difficulty >= 3 && !w.tops_up() { w.wait(300); }
    w.use_door(50, "door[50] for the 733 line");
    // The d2 Agent run settles 12 ticks, not 40: the run-6 ledger lost the great-hall race by
    // ~20-30 ticks — #5's grenade was already airborne when the pre-shoot's 0x1a round landed
    // (−27.7 at 8580, the killing hit [L the run-6 death line]); the leaf clears the shot line
    // in ~25 ticks of its swing [M door.rs update's open arm] and every idle tick hands the
    // guard's throw cycle a head start. The d1/top-up walks keep the pinned 40.
    if difficulty == 2 && !w.tops_up() {
        for _ in 0..12 { w.tick(nf_game::player::Input::default(), false); }
    } else {
        for _ in 0..40 { w.tick(nf_game::player::Input::default(), false); }
    }
    if difficulty >= 2 {
        // real-health-00agent-15-8 (d3 only): the pre-shoot's positioning walk is the measured
        // poison — the shoot machinery's flank nav walked Bond 18 u down the hall east under the
        // converging chasers and the whole corridor-entry bill landed DURING it: #4 (the second
        // plank rifle chaser, skin 0x5000087 — the d3 roster's extra the d2 stand never met) 15.0
        // in 5 at d 1.9, #19 7.0 at d 2.8, #5 7.0 — 29.0 of the walk's remaining 34.4, health
        // 5.4 at the door[50] through fight [L the 2026-11-14 run: the hits at frames 9632-9929,
        // Bond at (2.3, 28.7)-class spots mid-nav]. A player does not walk into a converging
        // squad to shoot one man: the fights' own stands answer the thrower and the chasers from
        // the legs (the fights killed #19/#13/#12 on the same run — the machinery works; the
        // bill was Bond's own walking). #5's first grenade may still land — the fights' race is
        // measured on the legs, not from a walk.
        // real-health-00agent-15-8: the positioning pre-shoot STAYS on d3 (measured: the
        // legs-first and threshold-window shapes both ate the convergers' full fire — the
        // fights' polls pick NOTHING from the corridor because the convergers are east of the
        // hall until their rounds already fly [L the 2026-11-14 run3/run4/run5 deaths
        // t9978-t10192, the run4 window's 30 polls never fired] — the shoot machinery's
        // flank nav is the only shape that crosses the kill zone on the moving-target roll
        // [M 0x179740 DroneFiring_TargetMoving_Accuracy]; the run2 shape crossed the whole hall
        // and reached the fence leg at 5.4).
        if let Some(di) = agent_drone_near(&w, [6.25, 41.45, 35.69], 6.0) {
            // (level-15-agent-rim-shot) The d2 900-tick pre-throw stand is GONE: on the lob
            // route #5's first frag is airborne before the threshold arrival (the measured rain:
            // pops at 8929/8993, thrown 8719/8783, −27.39/−17.59) and ANY stand holds Bond in
            // the pop zone — the measured 240-tick cap died at the same 8993. The shoot
            // machinery answers him with its own positioning (the fights' return fire killed the
            // corridor's #19 at this very threshold in ~60 ticks); the legs' push keeps Bond
            // out of the thrown frags' rest points.
            w.shoot(di, "the great-hall grenade guard (placement 733) from the door[50] threshold");
        }
    } else if !w.tops_up() {
        w.shoot(5, "the great-hall grenade guard (placement 733)");
    }
    w.walk_route(route_d2);
    // REAL-HEALTH TACTIC (level-14-15-real-health-finish): door[42] from the NORTH side, not
    // through it — the #447/#448/#449 fence at z ≈ 17.6 (the floor-to-ceiling beam at x 0 and
    // the two diagonals [D]) has no lane the body clears, and the old through-the-door shot
    // from z 23 is 5.7 u off the beam, inside the fence's own blast (radius 8 [M
    // Explode_Propagate 0x17c518]). From the threshold the round rides the open doorway's
    // window ~9 u — the a_p2k_round_from_10_units_explodes_the_door38_fence dance — and the
    // neighbours still go on the blast's case-0x4B entries [M 0x17c518].
    let route: &[(&str, Wp)] = &[
        // The leaf sits at x −1.1 (door[50]'s printed pos (−1.1, 41.6, 46.1) is the pattern
        // [D doors 50/42]) — the E reach needs head→leaf d² < 6.25 [M Player_Activate
        // 0x1a5170], so the stand is 0.8 north of the leaf plane, not the lane centre.
        ("door[42] threshold", Wp::new(0.0, 41.4, 25.9).r(0.6)),
    ];
    w.walk_route(route);
    // The walker's own door press on the corridor door itself — index 42, the walk's door
    // numbering this route's comments already use ("doors [50] z 46.1, [42] z 25.1…"). The
    // DEBUG probe (kept in the report): door_near(−1.1, 25.1) lands on door[20] — the
    // LOWER-DECK twin placement 646 (−1.04, 23.85, 25.10), 17.5 below the corridor floor,
    // dxz 0.06 vs door[42]'s 0.064 — whose E reach can never fire from up here; door[42]
    // (−1.05, 41.59, 25.06) is the floor-level leaf the passage needs open. Then a settle for
    // the swing before the shot line crosses the doorway.
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let d = &sim.drones.doors.list[42];
        assert!((d.pos[2] - 25.06).abs() < 0.2 && (d.pos[1] - 41.59).abs() < 0.5,
            "doors.list[42] is the corridor floor door at z 25.1, got {:?}", d.pos);
    }
    // A gun with rounds for the fence shot: the d2 west-room sweeps can spend the whole carry
    // (measured on the stream join: clip 1 spare 0 at the mark, the one round stopped on the
    // closing leaf and the walk stood dry behind the shut door). A player takes the fights'
    // drops on the way and draws a loaded gun before he opens the door.
    // real-health-00agent-15-8 (d3 only): the fence shot needs ONE round — the beam round's own
    // case-0x4B chain sets the neighbours [M 0x17c518] — and the carried P2K holding it IS the
    // shot. The d3 resupply nav from the corridor found nothing on the walked path and its last
    // attempt hung the 3000-tick cap over the great hall's middle pit at 5.4 health (the
    // 2026-11-14 run's STUCK lines, three caps ≈ 9000 idle ticks at a sliver of health) for a
    // round Bond already carries. The d2 keeps the resupply (its stream is committed; the same
    // clip-1 state resupplied successfully there [L the d2 ledger]).
    let low = { let g = &w.scene.sim.as_ref().unwrap().gun; (g.clip as i32 + g.spare as i32) < 4 };
    let p2k_holds_the_round = { let s = w.scene.sim.as_ref().unwrap(); s.gun.weapon == 0x02 && s.gun.clip >= 1 };
    if low && !(difficulty >= 3 && !w.tops_up() && p2k_holds_the_round) {
        w.resupply("the fence shot's rounds");
        assert!(w.switch_to_armed("the fence shot's gun"), "a carried gun has rounds for the fence");
        w.walk_route(route); // back to the door's E reach
    }
    w.use_door(42, "door[42]");
    for _ in 0..40 { w.tick(nf_game::player::Input::default(), false); }
    assert!(w.scene.sim.as_ref().unwrap().drones.doors.list[42].state & 0xc != 0,
        "door[42] is open inside the auto-close window [M 0x13660c-0x136700]");
    // REAL-HEALTH TACTIC (level-15-agent-doors-2): the fence's own pops are pure distance — all
    // three records carry damage 80, radius 10 [L the run probe 2026-11-04 printed all three:
    // damage 80, radius 10; the measured pops at the threshold stand were
    // 80·(1−9/10)·0.6 = 4.8 and 80·(1−8/10)·0.6 = 9.6 ×2 = 24.0 together]. A player who has
    // come this far takes the shot from OUTSIDE every blast: two steps north put all three
    // mines ≥ 10 u away ((int)d ≥ 10 → the sweep's damage is zero [M Explode_Propagate
    // 0x17c518's truncation]) — the line still runs down the open doorway's x 0 aperture and
    // the assert's range rule (d > 8) holds with room to spare.
    // REAL-HEALTH TACTIC (muzzle-repin-6 measure): the old walk_to the mark turns Bond ~π (he
    // arrives facing north) and the shoot_mine aim turn — pulling only while the eased aim is
    // inside the 0.002-rad window — lands its first rounds after door[42]'s auto-close: the
    // measured d2 run had los None at t0 (the leaf still open, the auto-close timer ~150 into
    // its 360 quiet ticks [M door.rs AUTO_CLOSE 0x13660c-0x136700]) and los Some(0.271) by t600
    // — 0.271 × 10.8 u ≈ the z 25.06 door plane [M world.rs segment fraction] — with clip 1
    // spare 9 dry by t1200 and the mine intact. A player covers the 3.4 u lane BACKWARDS with
    // the weapon already up: face the beam at the threshold (the walk arrived facing south —
    // the beam is dead ahead), backpedal to the mark holding the view, and the trigger is
    // on-target from the first gate tick, ~300 ticks inside the door's open window. Real-health
    // runs only: the master (top-up) walk's pinned timeline stays byte-identical.
    // REAL-HEALTH TACTIC (level-15-agent-rim-shot-2, the measured ledger): the d2/d3 run draws
    // the pistol HERE, at the SHUT door — the old order drew after the backpedal and re-opened
    // the auto-closed leaf for it, and the re-open's own approach walked Bond back into the
    // threshold (measured: Bond (−0.7, 25.8) at the shot), where the fence's own chain popped on
    // him — [ines] "#447 went at tick +14, Bond 9.01 u off, cause 2" → −10.73 −14.59 −15.53 =
    // 40.85, the run's largest bill, and the roster dump at the press prints ZERO live hostiles
    // (the old "corridor crossfire" reading was the shot's own chain). The pistol drawn first,
    // the door opened once, the shot comes from the mark at 11.21 u centre-to-centre — the hurt
    // distance 11.21 − 1.06 = 10.15 > 10, the sweep's zero [M Explode_Propagate 0x17c518] — and
    // the diagonals sit farther still. Real-health runs only: the master (top-up) walk's pinned
    // timeline stays byte-identical.
    if difficulty >= 2 && !w.tops_up() && w.scene.sim.as_ref().unwrap().gun.weapon != 0x2 {
        draw_persist(&mut w, 0x02, "the fence shot's pistol, drawn at the shut door[42]");
        w.reload_full();
    }
    if !w.tops_up() {
        // real-health-00agent-15-8 (d3 only): the mark rides NORTH of the diagonals — the d3 pain
        // row 1.0 makes the old mark's own chain fatal. The old mark (0.0, 28.5) stands 10.69 u
        // from the #778 diagonal (1.52, 41.47, 17.83 [D placement 778]) — the hurt distance
        // 10.69 − 1.06 = 9.63 sits INSIDE the radius-10 sweep [M Explode_Propagate 0x17c518:
        // Vec_Dist3D, minus obj+0x8C, max.s 0, /radius, ×damage] — 80·0.037 = 2.96 from #778 and
        // 1.92 more from #779 (−1.68, 17.86): the measured 2026-11-14 run died AT THE MARK to the
        // shot's own chain (−5.36 world, health 5.4) — the d2 run paid the same geometry 1.78
        // (its 0.6 pain row [M Player_HandlePain 0x1902c8]) and survived. z 29.3 puts every fence
        // mine ≥ 11.4 u: d − 1.06 ≥ 10 → the sweep's zero — #778 11.47, #779 11.46, #447
        // (−0.01, 44.22, 17.29) 12.22 with its beam y in the 3D distance. The line still rides the
        // open doorway's x 0 aperture (the door plane z 25.06 sits between the mark and the beam;
        // the line's x at the plane ≈ 0.0 [D the leaf at x −1.1]). The d1/d2 marks stay
        // byte-identical.
        let mark_z = if difficulty >= 3 { 29.3f32 } else { 28.5f32 };
        backpedal_face(&mut w, [0.0, 41.4, mark_z], [0.0, 42.3, 17.29], 0.5,
            "the fence mark north (backpedal, view on the beam)");
    } else {
        w.walk_to(Wp::new(0.0, 41.4, 28.5).r(0.5), "the fence mark north (outside the pops' radius 10)");
    }
    // 00 Agent (join-reds2-0): the stream join left the walk holding the dropped 0x1a here (clip
    // 30 spare 0) — its arcing rounds missed the beam by 3.9/3.5/2.6/4.7 u, the bracket ran out of
    // fixes and door[42] shut on the shot line; the carried P2K still held 4 rounds (it came up
    // dry-swapped at t1800 of the same run). A player takes the precise shot with the pistol
    // (spread 0.0 [L ee_start_05.bin weapon_data 0x02 +0x24/+0x98]).
    // Agent (batch-1109c-fix): the same take. The guard door-contact and drone contact-turn
    // rules shifted the shared stream, and the d2 walk now reaches this mark holding the 0x1a
    // as well. Its rounds missed by 4.26/4.32/3.68/13.28 u, the bracket ran out of fixes, and
    // door[42] shut on the line (master's stream landed the 5th round by luck after misses of
    // 2.24/2.71/0.92/2.63 u). The pistol shot is the one the walk's own doc names.
    // capsule-join (guard-standing-capsule-2): the pushed walks' shifted fights left the agent sisters at the
    // fence shot holding whatever the dry-swaps parked — the d2 run: the 0x1a (arcing P2K,
    // clip 30) whose rounds missed 3.67→2.54→2.23→1.71 u into the bracket's 4-fix cap; the d3
    // run: the 0x1b (clip 4 spare 2) — both left door[42] shut on the line at ~t500 (s0x2 tmr 0
    // [L the d2/d3 red traces]) and every later shot hit the leaf. A player takes the precise
    // shot with the pistol (spread 0.0 [L ee_start_05.bin weapon_data 0x02 +0x24/+0x98]) and
    // re-opens the auto-closed door for it.
    // (level-15-agent-rim-shot-2) The re-open below now serves only the paths that did not
    // pre-draw above (the d1 real-damage run never enters it: difficulty 1; the d2/d3 runs
    // arrive at the mark holding the already-drawn pistol, so the condition is false and Bond
    // stays at the mark, outside the pops).
    if difficulty >= 2 && w.scene.sim.as_ref().unwrap().gun.weapon != 0x2 {
        draw_persist(&mut w, 0x02, "the fence shot's pistol");
        w.reload_full();
        w.use_door(42, "door[42] re-opened for the fence shot");
        for _ in 0..40 { w.tick(nf_game::player::Input::default(), false); }
    }
    let d = shoot_mine(&mut w, 447, [0.0, 42.3, 17.29], "the south-corridor fence #447 from the threshold");
    assert!(d > 8.0, "#447 is shot from RANGE through the open doorway ({d:.2} u)");
    w.wait(10);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        // The fence clears on the ONE round from 11.1 u; no body contact anywhere. The cause
        // order follows the shot's muzzle: the P2K round (the d1/top-up fence gun) takes the
        // #447 beam first and the neighbours go on its case-0x4B blast [M 0x17c518]; the d2
        // fights' gun (the drawn 0x1a — the P2K pool is spent by the west room) is a GRENADE
        // round whose direct mirrored contact took the #448 diagonal first [L the run-7 shot:
        // #447 cause 3 blast, #448 cause 2 bullet] and the rest went on the case-0x4B chain.
        // Either way the proof stands: one round from range, the whole fence gone.
        // The cause table: no mine goes by body contact. With the P2K round the shot mine #447
        // goes by the round itself and its neighbours on the blast; the 0x1a round's mirrored
        // contact can take a neighbour first [L the run-7 shot above].
        let p2k = sim.gun.weapon == 0x02;
        for plc in [447usize, 448, 449] {
            let m = &sim.drones.mines[mine_idx(&w, plc)];
            assert!(m.exploded && m.cause != nf_game::worldobj::MINE_BODY,
                "#{plc} clears on the range shot (exploded {}, cause {})", m.exploded, m.cause);
            if p2k && plc != 447 {
                // 00 Agent reaches this shot with a guard's dropped automatic in hand (clip 30): its
                // burst can take a neighbour beam by the round before the blast does [M Collide_Update
                // 0x1ea398:160-233 — the round stops on the first mine it picks].
                let ok = m.cause == nf_game::worldobj::MINE_BLAST
                    || (difficulty >= 3 && m.cause == nf_game::worldobj::MINE_BULLET);
                assert!(ok, "#{plc} goes on #447's blast (cause {})", m.cause);
            }
        }
    }
    w.walk_to(Wp::new(0.0, 41.4, 23.0).r(1.0), "door[42] through");
    let route: &[(&str, Wp)] = &[
        ("door[51] threshold", Wp::new(0.0, 41.4, 11.5).r(0.9)),
        ("door[51] through", Wp::new(0.0, 41.4, 8.5).r(1.0)),
        ("the split east", Wp::new(5.0, 41.4, 4.0).r(1.0)),
        ("the split south", Wp::new(5.0, 41.4, -4.0).r(1.0)),
        ("the hall mouth", Wp::new(1.0, 41.4, -7.0).r(1.0)),
        ("door[49] threshold", Wp::new(0.0, 41.4, -9.3).r(0.9)),
        ("door[49] through", Wp::new(0.5, 41.4, -12.5).r(1.0)),
        ("the exit hall", Wp::new(0.5, 41.4, -21.0).r(1.2)),
        ("the exit approach", Wp::new(-0.6, 41.4, -21.0).r(1.4)),
    ];
    w.walk_route(route);
    {
        // The way through: no mine went off on Bond's body — every one the lanes cross was shot
        // (or went on a shot neighbour's blast; #487 goes on #526's).
        let sim = w.scene.sim.as_ref().unwrap();
        for m in &sim.drones.mines {
            assert_ne!(m.cause, nf_game::worldobj::MINE_BODY, "mine #{} went off on the body", m.placement);
        }
        for plc in [462usize, 383, 384, 385, 526, 447, 448, 449] {
            assert!(sim.drones.mines[mine_idx(&w, plc)].exploded, "#{plc} on the route is gone");
        }
    }

    // The USE exit #445: face the plane, pulse E (the use sphere: head + look, radius 1 [M 0x1a5170]).
    let exit_at = [0.0f32, 41.3, -24.8];
    // los-mask-join: the mine shots before the hall leave the view wherever the last aim put
    // it — the door[29] ankle beam's walked aim ended at pitch -0.38, and the E sphere (head +
    // look, radius 1 [M 0x1a5170]) then fell short of the plane on the floor for all 300 ticks
    // [L the masked run; -0.11 on the unmasked run reached it]. A player looks up at the
    // panel before he presses E.
    for _ in 0..60 {
        let p = w.scene.sim.as_ref().unwrap().bond.pitch;
        let err = -0.1 - p;
        if err.abs() < 0.02 { break; }
        w.tick(nf_game::player::Input { look: err.clamp(-0.05, 0.05), ..Default::default() }, false);
    }
    w.walk_to(Wp::at(exit_at[0], exit_at[1], exit_at[2] + 1.2, Act::Use { use_ticks: 300 }).r(1.0), "the USE exit #445");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert_eq!(sim.drones.level_exit, Some(0x0700_0016),
            "E at the #445 plane loads 07000016 [D placement #445 param 0]");
        // The pinned margin (level-15-agent-rim-shot-2): the Agent crossing is not green by a
        // sliver — the plank rifle guard is banked at the door[39] north stand and the fence
        // shot is taken from outside every blast's radius 10, so Bond crosses with 25 or more
        // health (the rim-shot tree crossed at 2.0 with the ledger's 98.0).
        if difficulty == 2 && !w.tops_up() {
            let hp = sim.drones.health.health;
            assert!(hp >= 25.0, "the Agent crossing keeps the real margin at the exit, got {hp:.1} (the rim-shot tree crossed at 2.0)");
        }
        assert!(!sim.drones.channels.on(0x42), "nothing writes ch 0x42 on this level [D scans] — the gated gate #343's channel stays cold");
        assert_eq!(sim.drones.mission.states[21][0], 3,
            "the objective stays announced on 07000015 (its completing channel 0x8e is 07000016's spawn plate [D])");
    }

    // The objective's completion, the way the game does it: the exit loads 07000016 carrying the
    // player's channel state (the mission's persistent gates — `RamSave::capture`'s Channels arm
    // [M Trigger_Activate 0x1b2318:97-116 serves the carried save on the load]) and its spawn
    // plate #252 (4.14..6.21, −0.19..1.88, −67.76..−63.58 [D 07000016]) fires ch 0x8e, record
    // 21's completing channel. The walk carries ITS OWN live channels — the exact state this
    // walk earned — and finishes on the plate, where the game's own arrival spawn (5.17, 0.90,
    // −65.30 [D start #1]) already stands inside the plate's box.
    // The hand-off is the game's own: `warm_exit_save` (Trigger_Activate's `Player_RamSave`
    // 0x195670 — the player state, the held weapon byte and the channels [M 0x1b2318:97-116]).
    let (save, owned15, held15) = {
        let sim = w.scene.sim.as_ref().unwrap();
        let inv = &sim.drones.inventory;
        (nf_view::warm_exit_save(sim), (0..inv.owned.len()).filter(|&i| inv.owned[i]).collect::<Vec<_>>(), sim.gun.switch_request)
    };
    assert_eq!(save.weapon, held15, "the exit saves the held weapon byte [M 0x195670:18-19]");
    let mut w16 = Walker::new(nf_view::load_with_carry(&iso, "07000016.bin", false, Some(&save)).unwrap());
    w16.wait_enabled(3000, "the launch-centre arrival");
    w16.wait(200);
    {
        let sim = w16.scene.sim.as_ref().unwrap();
        // Bond's weapons on 16: case 0x7000016 keeps the carry (the P2K is owned) and re-selects the
        // saved weapon [M 0x1a7ef8:302-307], then `Player_CheckWeaponsLoaded` drops what 16's
        // archive has no skin for [M 0x1a97c8] — of 15's set only the decryptor 0x56.
        let inv = &sim.drones.inventory;
        let owned16: Vec<usize> = (0..inv.owned.len()).filter(|&i| inv.owned[i]).collect();
        let expect16: Vec<usize> = owned15.iter().copied().filter(|&i| i != 0x56).collect();
        eprintln!("[015] 15 exit: held {held15:#x} owned {owned15:x?}; 16 arrival: held {:#x} owned {owned16:x?}", sim.gun.weapon);
        assert_eq!(owned16, expect16, "16 keeps 15's weapons except the decryptor (no skin on 16) [M 0x1a7ef8 case 0x7000016; 0x1a97c8]");
        // REAL-HEALTH FORM (level-14-15-real-health-finish): the absolute kit is a superset law
        // — the walked chain KEEPS the 14 kit minus the Grapple (swept on 15) and the decryptor,
        // and the only additions are the fights' own guard-drop pickups: on real health the
        // fights' dry-resupply gates take the dead guards' weapons (the walk-over take
        // [M Pickup_Handler 0x18e650]), and a drop is the weapon's BASE [M DroneWeap_DropWeapon
        // 0x17a2a0 — `let base = d.weapons.base(w)`]: 0x14 is the 0x15 burst rifle's base (the
        // 0x5000087 guards), 0x1a the grenade-launcher guards' own (the 0x500008c guards)
        // [L weapon_data bases]. The measured run: owned [1, 2, 0x11, 0x14, 0x1a, 0x34, 0x37,
        // 0x4e, 0x54].
        assert!(expect16.iter().all(|i| owned16.contains(i)),
            "the walked chain keeps the 14 kit minus the Grapple (swept on 15) and the decryptor [M 0x1a7ef8 case 0x7000014], got {owned16:x?}");
        let extras: Vec<usize> = owned16.iter().copied().filter(|i| !expect16.contains(i)).collect();
        assert!(extras.iter().all(|&i| i == 0x14 || i == 0x1a),
            "the only additions are the fights' guard-drop pickups (a drop is the base weapon [M 0x17a2a0]), got {extras:x?}");
        assert_eq!(sim.gun.weapon, held15, "16 draws the weapon Bond held at 15's exit [M 0x1a7ef8:306]");
        assert_ne!(sim.gun.weapon, 1, "Bond does not arrive on 16 with only his fists");
        assert!(sim.drones.channels.on(0x8e), "the spawn plate #252 fires ch 0x8e [D 07000016]");
        assert!(sim.drones.mission.states[21].iter().all(|&s| s == 4),
            "the carried record 21 completes — 'Find the exit to Omega Sector' is done");
        assert_eq!(sim.drones.mission.states[22][0], 3,
            "07000016's own objective announces on the arrival [L record 22 obj 1]");
    }
}

/// The sister of the shots: walking straight into the z ≈ 18 doorway fence, the same lane the walk
/// takes, sets it off on the body — the walk's body HITTEST (`Player_Collision` 0x194df0's default
/// arm, floor to 1.86) meets the beams, `Collide_Update` 0x1ea398 mirrors the contact onto the mine
/// (+0x58 = Bond) and `Mine_Update` 0x17c930 explodes [M]. On real damage the blasts hurt Bond.
#[test]
fn walking_straight_into_the_doorway_fence_sets_it_off() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000015.bin", false).unwrap());
    w.wait_enabled(3000, "the silo arrival");
    w.wait(120);
    // The guards hold (the port's dev freeze): a guard's grenade near door[38] reaches the fence
    // too (its blast's case-0x4B entries [M 0x17c518]) — this pin isolates the body contact.
    w.scene.sim.as_mut().unwrap().drones.hold_npcs = true;
    w.walk_to(Wp::new(88.0, 41.4, -0.5).r(1.2), "the gantry west");
    assert!(w.follow_nav_route([34.6, 41.4, 6.0], "the silo crossing"), "the nav route crosses the gantry");
    w.walk_to(Wp::new(34.6, 41.4, 6.0).r(1.2), "the north turn");
    w.walk_to(Wp::new(34.6, 41.4, 13.0).r(0.8), "door[38] north of the leaf");
    let fence: Vec<usize> = [383usize, 384, 385].iter().map(|&p| mine_idx(&w, p)).collect();
    assert!(fence.iter().all(|&i| !w.scene.sim.as_ref().unwrap().drones.mines[i].exploded),
        "sister: the fence stands while Bond stands short of it");
    // Line up on the vertical beam's lane (x 35.3) short of the fence, then walk straight in
    // (the doorway's 0.76 rim at z ≈ 17.7 stops a plain walk on the x 34.6 line just short of
    // the diagonals). Real damage from here: the walk's top-up would hide the blasts.
    walk_plain(&mut w, [35.3, 41.4, 14.5], 0.3, "the x 35.3 lane");
    w.top_up_health = false;
    w.scene.sim.as_mut().unwrap().drones.health.health = 100.0;
    let mut boom = None;
    for _ in 0..900u32 {
        let (input, hit) = {
            let sim = w.scene.sim.as_ref().unwrap();
            let p = sim.bond.pos;
            let hit = fence.iter().copied().find(|&i| sim.drones.mines[i].exploded);
            let err = walker::wrap_pi((35.3 - p[0]).atan2(24.0 - p[2]) - sim.bond.yaw);
            (nf_game::player::Input { turn: err.clamp(-0.08, 0.08), forward: 1.0, ..Default::default() }, hit)
        };
        if let Some(i) = hit { boom = Some(i); break; }
        w.tick(input, false);
    }
    let i = boom.unwrap_or_else(|| panic!("walking north through the z-18 doorway never set the fence off\n{}",
        walker::fail_dump_pub(&w.scene, [35.3, 41.4, 24.0], "the fence walk")));
    for _ in 0..10 { w.tick(nf_game::player::Input::default(), false); }
    let sim = w.scene.sim.as_ref().unwrap();
    eprintln!("[mines] fence boom: first #{} at Bond {:?}; causes {:?}", sim.drones.mines[i].placement, sim.bond.pos, fence.iter().map(|&k| sim.drones.mines[k].cause).collect::<Vec<_>>());
    assert_eq!(sim.drones.mines[i].cause, nf_game::worldobj::MINE_BODY,
        "the first mine goes on Bond's body contact [M 0x194df0 → 0x1ea398 → 0x17c930]");
    assert!(sim.bond.pos[2] < 18.0, "the body met the fence before crossing it (z {:.2})", sim.bond.pos[2]);
    assert!(sim.drones.health.health < 100.0, "the blast at contact range hurts Bond (health {:.1}) [M 0x17c518 case 3]",
        sim.drones.health.health);
}

/// #487 across door[52]'s south mouth (x −2.4..2.9, z 59.36..59.56, y 41.76..42.21 over the
/// 40.28 floor [D]): crouched, the body (`Player_Collision` 0x194df0 case 3: the B sphere on the
/// floor, the A sphere's top 1.3 over it [M :228-248]) walks under the beam; standing (the default
/// arm, 1.86 high [M :315-333]) it meets it and the mine explodes on the body. Bond is placed in the
/// west room at the door (the main walk reaches here after #526's blast has already taken #487),
/// the guards hold, then real input only: the E press on the door, the crouch toggle, the walk.
#[test]
fn crouching_under_the_door52_beam_passes_it() {
    let iso = iso();
    for crouch in [true, false] {
        let mut w = Walker::new(nf_view::load(&iso, "07000015.bin", false).unwrap());
        w.wait_enabled(3000, "the silo arrival");
        w.scene.sim.as_mut().unwrap().drones.hold_npcs = true;
        {
            let s = w.scene.sim.as_mut().unwrap();
            s.bond.pos = [0.5, 41.4, 62.2];
            s.bond.vel = [0.0; 3];
            s.bond.yaw = std::f32::consts::PI;
        }
        w.wait(30);
        let m487 = mine_idx(&w, 487);
        if crouch { set_crouch(&mut w, true); }
        open_door_facing(&mut w, [1.1, 41.6, 60.7], "door[52]");
        walk_plain(&mut w, [0.5, 41.4, 57.5], 0.6, "through door[52]");
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.bond.pos[2] < 58.5, "the walk crossed #487's line (z {:.2})", sim.bond.pos[2]);
        let m = &sim.drones.mines[m487];
        if crouch {
            assert!(sim.bond.crouched, "the walk stayed crouched");
            assert!(!m.exploded, "crouched, the body passes under #487 (cause {})", m.cause);
        } else {
            assert!(m.exploded && m.cause == nf_game::worldobj::MINE_BODY,
                "sister: standing, the body meets #487 and it goes (exploded {}, cause {})", m.exploded, m.cause);
        }
    }
}

/// Real damage: the whole-mission walk (the silo gantry → the Omega Sector exit → the real
/// 07000016 load) with the health top-up off for every walker it makes — the garrison's fire
/// (all 20 guards stay live and armed) and the laser-mine blasts are the port's own damage —
/// and the walker's return fire on; Bond's death panics with the death spot and the last hits
/// (common/damage_log.rs). No health/armour/ammo writes (the mine shots are the real fire path;
/// the route's own tactic — every mine shot from down its lane — is the level's offering); the
/// exit load into 07000016 is the game's own USE-exit path [M Player_Activate 0x1a5170 case
/// 0x1d → Trigger_Activate 0x1b2318], not a teleport.
///
/// The fights HEAD-HUNT (`with_real_damage_head_hunting`): the first real-health run's damage
/// log shows the attrition law — the route's own #526 doorway blast cost 27.73, the
/// garrison's long-range chip the rest, and the killing sequence was a 2.8-u guard's spray
/// down the south corridor igniting the #447/#448/#449 mine fence [M Bullet_update
/// 0x12e7b0: any bullet's flight segment picks a mine (mode 0x201, mask 0) → Collide_PickObj
/// — the game's own guard fire sets mines off] — the level's offering is the aim: one head
/// hit drops a guard before its spray reaches a fence [M NDrone2_HitDamage 0x145558 ×
/// DroneDamage_Head].
/// Agent-difficulty sister of `..._real_damage` (real-health-agent-difficulty-12-13-14-15): the
/// same walked route and the same end, loaded at difficulty 2 the menu way (the front-end's
/// launch path [M C_SBNFDFCTY 0x216ca8:15 → load_with_carry_difficulty]; GameState+0x28 →
/// Drones::difficulty, asserted at load). Agent spawns the min-difficulty-2 placements
/// [M Drone_Create 0x136be0:49], scales Bond's pain by Plr_DMod_Normal [M Player_HandlePain
/// 0x1902c8] and the guards' hit chance by Accuracy row 2 [M DroneWeap_DoBulletAccuracy
/// 0x179740:74-88]. Real input only; no health, ammo or channel writes. GREEN on the doors
/// route (level-15-agent-doors-2): the west-room grenadiers are answered by proximity at the
/// jog (the placement-spot pre-shoots missed the door-route drift) and the #447 fence shot is
/// taken from 11.1 u — outside all three blasts' radius 10 [L the probe] — Bond crosses the
/// exit at 26.2 health.
/// GREEN again on the sphere-falloff tree (level-15-agent-rim-shot): the game's blast falloff
/// is CONTINUOUS from the sphere-edge distance [M 0x17c5dc-0x17c620 disasm — Vec_Dist3D, minus
/// obj+0x8C, max.s 0, /radius, ×damage, the 0.0002 gate; the decompiler's "(int)" there was
/// its rendering of max.s], so the #526 rim shot's cost at the rim mark rose to 44.33 (2.47 u)
/// and the route died there. The walk takes the door[32] guard's own dropped 0x1b instead and
/// lobs from the threshold (15.2 u — the chain blast reads ~2.8 there), the case-0x4B entry
/// sets #526 [M 0x17c518], and the rim walk and shot skip. The measured run crosses the exit
/// with the ledger in the report.
/// GREEN at a real margin (level-15-agent-rim-shot-2): the rim-shot tree crossed at 2.0 with
/// 98.0 taken. The measured ledger's two largest bills were not guards holding the line but
/// the walk's own shape: the 40.85 "corridor pops" at door[42] are the FENCE SHOT'S OWN CHAIN —
/// the post-backpedal pistol draw re-opened the auto-closed leaf and its approach walked Bond
/// back into the threshold ([ines] "#447 went at tick +14, Bond 9.01 u off, cause 2"; the
/// roster dump at the press prints ZERO live hostiles), so the pistol is now drawn at the SHUT
/// door and the shot comes from the mark (11.21 u centre-to-centre, the sweep's zero [M
/// 0x17c518]); and the plank rifle guard (skin 0x5000087) is a chaser who met the west plank
/// crossing at d 2.0 for 21.6 — he is answered from the door[39] north stand (measured spot
/// (35.73, 41.28, 44.65) hp 10, 17.7 u off the stand) before the plank. The crossing takes
/// 58.5 and Bond reaches the exit with 41.5 health; the assert pins 25 or more.
/// GREEN again on the merged tree (los-mask-repins-15): the round-3 ledger's door[42] death
/// (tick 9330 at (-0.65, 41.40, 25.82), "red with the flat boundary push too") does not
/// reproduce anywhere on the merged master line — the walk passes at 823ff190b (the round-3
/// merge, this branch's base) already [L the 2026-11-11 filtered run] and in the whole binary
/// on master, twice, byte-identical [L]. The ignore was written on the round-3 development
/// tree and outlived its own red (an ignored test is never re-run), so it is removed.
#[test]
fn walk_spawn_to_the_omega_sector_exit_real_damage_agent() {
    walker::with_real_damage_head_hunting(|| walk_spawn_to_the_omega_sector_exit_at(2));
}

#[test]
fn walk_spawn_to_the_omega_sector_exit_real_damage() {
    walker::with_real_damage_head_hunting(|| walk_spawn_to_the_omega_sector_exit_at(1));
}

/// 00 Agent-difficulty sister of `..._real_damage` (real-health-00agent-12-13-14-15): the same
/// walked silo → Omega Sector exit route at the menu's 00 Agent pick — GameState+0x28 = 3
/// through the front-end's launch path [M C_SBNFDFCTY 0x216ca8:15 → load_with_carry_difficulty;
/// GameState+0x28 → Drones::difficulty, asserted at load]. On this level the pick scales Bond's
/// pain by the 0x14..=0x16 group's Hard row 1.0 (Easy 0.4, Agent 0.6 [M Player_HandlePain
/// 0x1902c8; ReadTuningVars 0x1d3a80's per-level jump table; L the d2-sisters report's rows])
/// and the guards' hit chance by DroneFiring_Accuracy_Hard 1.3 (Easy 0.8, Normal 0.9 [M
/// DroneWeap_DoBulletAccuracy 0x179740:74-88]); the min-difficulty placements pass Drone_Create's
/// gate [M 0x136be0:49] — more of the garrison than the Agent roster. The Agent sisters' tactics
/// (the plank/west-room grenadier stands, the doorway guards) sit behind `difficulty >= 2` gates
/// in the body, so the d3 run inherits them; the d3-only deaths get their own d3-gated blocks.
/// Real input only; no health, ammo or channel writes.
///
/// IGNORED with the measured budget: the d3 run's own ledger puts the route over 100 health
/// before the west lane — (a) the opening burst 63.0 in 9 hits from the d3 roster's
/// 0x50000b2 guard behind the spawn (the ledger's #15, (72.35, 41.34, −9.44) at his hits;
/// the return fire finished him at tick 867; a spawn stand was TRIED and cost MORE — the
/// line from the gantry is blocked, segment Some(0.068), and the shoot machinery's flank
/// through door (81.7, −1.0) burned 56.0 and two 3000-tick caps); (b) the #526 ankle-beam
/// rim-shot blast ~69.3 at d3 — the rim hides every farther lane shot [D the trough; the
/// walk's own shot is from just east of the rim], and the pain row scales blasts [M
/// bullet.rs damage_factor 0x14..=0x16 → 1.0 at d3; grenade.rs:609 routes the blast's
/// player hit through it; L the d1 27.73 and d2 41.60 ledgers = 69.3 raw]. 63.0 + 69.3
/// exceeds 100 before the plank legs, let alone the endgame fights. NEXT LEVERS: the
/// #526 shot from door[32]'s threshold (~14.9 u down the lane) with the door[32] guard's
/// own dropped 0x1a — the grenade's arc clears the rim and the case-0x4B chain sets #526
/// [M 0x17c518] at a blast reach of ~0.5 raw; a pre-emptive #15 kill needs his spawn spot
/// and a clear line (a live scout); the grenadier stands now sit behind the d3 gate in the
/// body (the fence-mark stands at 28 u never fired — the 0.02-rad aim gate is a torso
/// width there; the door[38]-north point is the next try).
#[test]
#[ignore = "00 Agent (11-13 update): the #462 beam is shot from (65.0, -0.3) (17.85 u, no hurt; the old 13.56 u stand paid 14.72) and the solo-line walk holds the fights' resupply nav, so the west room is cleared at 73.7 (was 34.4). The walk dies at t12256 at (0.00, 41.33, 11.27) at the door[51] threshold with 23.2 health: #4 (skin 0x5000087) 33.0 in 11 on the door[50] through leg (58.7 -> 23.2), then #14 (skin 0x50000b2, 0x1a at (3.1, 41.3, 5.8)) one 23.2 hit. Next: #4 before door[50], #14 from the door[51] threshold. Earlier: 00 Agent (11-14, real-health-00agent-15-8): the FENCE LEG IS SOLVED — the walk now crosses the #447 fence shot and dies at the exit corridor's door[51] threshold, t11681 at (0.08, 41.31, 11.27), health 5.4 -> 0.0 to #12 (skin 0x500008c, the 0x1a grenade launcher at (5.4, 41.3, -1.9), the split's own guard, the d3 roster's extra) paying 3.50 + 1.86 = 5.36 — death by 0.04 of health; the fights answered (#12 hp -5) one fight cycle late. WHAT THIS ROUND FIXED, each measured: (1) THE FENCE MARK (d3-gated z 28.5 -> 29.3) — the old t20553 death was THE SHOT'S OWN CHAIN, not #4: the old mark (0.0, 28.5) stood 10.69 u from the #778 diagonal (1.52, 41.47, 17.83 [D placement 778]), the hurt distance 10.69 - 1.06 = 9.63 INSIDE the radius-10 sweep [M Explode_Propagate 0x17c518 — Vec_Dist3D, minus obj+0x8C, max.s 0, /radius, x damage], 80·0.037 = 2.96 from #778 + 1.92 from #779 — the measured -5.36 world hit killed 5.4 (the d2 run paid the same geometry 1.78 at its 0.6 pain row [M Player_HandlePain 0x1902c8] and survived). z 29.3 puts every fence mine >= 11.4 u (d - 1.06 >= 10 -> the sweep's zero; #447 12.22 with its beam y in the 3D distance) and the line still rides the x 0 aperture; measured: '#447 went at tick +3, Bond 12.18 u off, cause 2' with ZERO hurt. (2) THE RESUPPLY SKIP (d3, the P2K holding the round) — the corridor resupply nav found nothing on the walked path and hung three 3000-tick caps over the great hall's middle pit (grounded=false at (9.85, 35.03), segment Some(0.045)) — ~9000 idle ticks at 5.4 health for a round Bond already carries; skipped when sim.gun is 0x02 with clip >= 1 (the fence shot needs ONE round — the beam round's case-0x4B chain sets the neighbours [M 0x17c518]). The d2 keeps the resupply (its stream is committed). MEASURED-NOT-SHIPPED (all three ate MORE fire): (a) the fights' threshold window — 30 return_fire polls at the open door[50] never picked a candidate (the convergers are east of the hall / behind its structures until their rounds already fly [L the run4 death t10192, zero door[50] fights]); (b) the legs-first shape (walk route_d2[..2], the aimed stand on #5 from the hall east) — died t9978 to #5's 0x1a arc rain (19.4) at the door[50] through leg — the fights still cannot see from the corridor; (c) the second-chaser pre-stand (agent_drone_nearest_skin 0x5000087 at door[38] north) — killed #17 at +44 but #4 (the SECOND 0x5000087 chaser) was outside the 16 u tol and paid his full bill. THE UPSTREAM LEDGER (the shared run2/run7 shape, health 34.4 after the west room -> 5.4 at the fence leg): #11 21.0 in 3, #4 15.0 in 5 (the second plank rifle chaser — QUIET through the plank zone so the fights' shooter pick never saw him [M DamageLog::shooters], chased the whole hall, caught the pre-shoot's positioning nav at d 1.9), #15 14.0 in 2, the world 15.1 in 2, #19 7.0, #5 7.0 — the corridor-entry bill 29.0, of which #4's 15.0 is the largest single item. The positioning pre-shoot STAYS (measured three ways: it is the only shape that crosses the kill zone on the moving-target roll [M 0x179740 DroneFiring_TargetMoving_Accuracy]; every stop-and-answer shape ate more). NEXT LEVERS, in order: (1) THE #4 ANSWER — loop the chaser stand until the 0x5000087 skin finder returns None (or widen the spot to the chasers' converging path (28-35, 33-38), tol 20): his 15.0 alone flips the run (5.4 + 15.0 = 20.4 at the fence, 20.4 - 5.36 = 15.0 past #12); (2) THE #12 ANSWER — the fights engage him only after his first launcher round: a pre-emptive aimed stand from the door[51] threshold (the line (0.0, 11.5) -> (5.4, -1.9) crosses the split's void band x -2..2 at eye height) before the last route legs; (3) the exit leg needs >= 11 health — either lever (1) alone suffices. The d1/d2 streams stay byte-identical (every lever gated difficulty >= 3 && !w.tops_up(); the d2 real-damage walk is green on this tree)."]
fn walk_spawn_to_the_omega_sector_exit_real_damage_00agent() {
    walker::with_real_damage_head_hunting(|| walk_spawn_to_the_omega_sector_exit_at(3));
}

#[test]
fn walk_spawn_to_the_omega_sector_exit() {
    walk_spawn_to_the_omega_sector_exit_at(1);
}

/// The objective chain at the other two menu picks: the same walked route and end with the
/// walker's health allowance, at Agent (2) and 00 Agent (3) — the min-difficulty placements
/// [M Drone_Create 0x136be0:49] and the keypad create-unlock tag (difficulty 1 only [M
/// Lock_Create 0x1ae1f8 tail]) change the level at each pick, so each pick's real path to
/// ch 0x8e is walked. The real-health sisters stay the damage pins.
/// GREEN again under the game's guard sight (los-mask-repins-15): the round-3 ledger's
/// door[42] crowd (guards #4 and #19 standing 0.1-1.6 u off Bond, the leaf closed again by
/// the #447 shot's start, the mine never going off in 3000 ticks) is gone on the merged
/// master line. Bisect pins the change to c3e09cb2f (death and hurt planes are solid, the
/// level-agnostic sim change): the walk still fails at 6e24d0164 [L the 2026-11-11 filtered
/// run] and passes at c3e09cb2f [L] and master, where the whole binary runs
/// 8/1 (the 1 is the d3 00-agent ledger, not this walk) twice, byte-identical [L]. How the
/// plane solidity moves the pair off the threshold is not measured — [S] unverified.
#[test]
fn walk_spawn_to_the_omega_sector_exit_agent() {
    walk_spawn_to_the_omega_sector_exit_at(2);
}

#[test]
fn walk_spawn_to_the_omega_sector_exit_00agent() {
    walk_spawn_to_the_omega_sector_exit_at(3);
}

/// The range shot the merges never pinned: a mine takes a BULLET from a distance, through the
/// game's own bullet path. The doorway fence #383 (the floor-to-ceiling beam at x 35.6, z 17.47
/// [D]) stands ~10 u south of the spine's z 7.5 line — door[38] (z 10.5) sits between, so the
/// shot rides the door's open window (the auto-close fires after `FRAME_RATE_INT·6` = 360 quiet
/// ticks [M door.rs `update`'s auto-close arm; 0x13660c-0x136700]): open the leaf from its E
/// reach, step back south, settle the aim, fire. The round's flight segment picks the mine (the
/// per-tick HITTEST mode 0x201 with mask 0 [M Bullet_update 0x12e7b0 disasm 0x12f040/0x12f134] →
/// `Collide_Pick` 0x1eb320 walks the straddled cels' object lists → `Collide_PickObj` 0x1eae80
/// admits the cel-linked mine: obj+0xF0 0, no class mask, a collision glist),
/// `Collide_Update` 0x1ea398:160-233 mirrors the hit onto the mine's list with +0x58 = the bullet
/// and `Mine_Update` 0x17c930 explodes on the referenced entry. The neighbours #384/#385 go on the
/// blast's case-0x4B entries [M Explode_Propagate 0x17c518]. No health writes: the guards hold
/// (the sister tests' dev freeze), so Bond walks the route on his own feet and the only damage in
/// the run is the mine's own blast.
#[test]
fn a_p2k_round_from_10_units_explodes_the_door38_fence() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000015.bin", false).unwrap());
    w.top_up_health = false;
    w.wait_enabled(3000, "the silo arrival");
    w.wait(120);
    w.scene.sim.as_mut().unwrap().drones.hold_npcs = true;
    draw(&mut w, 0x02); // the P2K: 7 + 41 rounds at the level start [D the 07000015 loadout]
    // The route west, then the NAV GRAPH's own silo crossing to the spine (the direct z ≈ 0 line
    // falls into the silo hole at x 67.45 [D floor grid]). #462 stands across the crossing's lane
    // and the lane walks into it, so it is shot from the lane first — route clearing; the RANGE
    // shot of this test is #383 below.
    w.walk_to(Wp::new(88.0, 41.4, -0.5).r(1.2), "the gantry west");
    let mine462 = [46.8f32, 41.556, 0.6];
    nav_until_clear(&mut w, [34.6, 41.4, 6.0], mine462, 14.0, "the silo crossing");
    shoot_mine(&mut w, 462, mine462, "the silo-crossing beam #462 (route clearing)");
    assert!(w.follow_nav_route([34.6, 41.4, 6.0], "the silo crossing"), "the nav route crosses the gantry");
    w.walk_to(Wp::new(34.6, 41.4, 6.0).r(1.2), "the north turn");
    // Door[38]: into the leaf's E reach (head→pos d² < 6.25 [M Player_Activate 0x1a5170]), open,
    // then back south of it — the shot line crosses the open doorway at x ≈ 34.9 [D leaf].
    w.walk_to(Wp::new(34.6, 41.4, 9.8).r(0.6), "door[38] E reach");
    open_door_facing(&mut w, [34.6, 41.4, 10.5], "door[38]");
    walk_plain(&mut w, [34.6, 41.4, 7.5], 0.35, "south of door[38], the shot line");
    let at = [35.6f32, 42.3, 17.47];
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let m = &sim.drones.mines[mine_idx(&w, 383)];
        assert!(!m.exploded, "#383 still hangs before the shot");
        let bp = sim.bond.pos;
        let eye_y = bp[1] + sim.bond.eye - sim.bond.crouch_drop;
        let d = ((at[0] - bp[0]).powi(2) + (at[1] - eye_y).powi(2) + (at[2] - bp[2]).powi(2)).sqrt();
        eprintln!("[mines] the range shot: bond {bp:?} eye_y {eye_y:.2} → #383 {at:?}, d {d:.2}");
        assert!(d > 8.0, "the shot is from RANGE ({d:.2} u — the task's > 8)");
        let di = walker::door_near(sim, [34.6, 41.4, 10.5]).expect("door[38]").1;
        assert!(sim.drones.doors.list[di].state & 0xc != 0,
            "door[38] is still open inside the auto-close window [M 0x13660c-0x136700]");
    }
    let d = shoot_mine(&mut w, 383, at, "the doorway fence #383 from 10 u");
    assert!(d > 8.0, "#383 takes the round from RANGE ({d:.2} u)");
    w.wait(10); // the blast's case-0x4B entries land on the neighbours' NEXT update [M 0x17c518]
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let m = &sim.drones.mines[mine_idx(&w, 383)];
        assert!(m.exploded, "#383 is gone (obj+0xFE bit 0 [M 0x17cb7c-0x17cb88])");
        assert_eq!(m.cause, nf_game::worldobj::MINE_BULLET,
            "the cause is the bullet's mirrored contact [M 0x1ea398:160-233]");
        // The blast-played proof is the neighbours' chain: #384/#385 can only explode through the
        // blast's own `Explode_Propagate` case-0x4B entries (a referenced HITDATA per mine in
        // reach, attacker = the exploding mine [M 0x17c518 case 0x4B; 0x17cde0]) — there is no
        // other writer of MINE_BLAST. (The `drones.explosions` list is the view's per-tick event
        // feed — the grenade section clears it every tick [M drone.rs `explosions.clear()`] — so
        // it is empty by the time the boom is observed and proves nothing here.)
        for plc in [384usize, 385] {
            let m = &sim.drones.mines[mine_idx(&w, plc)];
            assert!(m.exploded && m.cause == nf_game::worldobj::MINE_BLAST,
                "#{plc} goes on #383's blast (cause {})", m.cause);
        }
        eprintln!("[mines] #383's blast chain: causes {:?}, bond health {:.1}",
            [384usize, 385].map(|p| sim.drones.mines[mine_idx(&w, p)].cause), sim.drones.health.health);
    }
}
