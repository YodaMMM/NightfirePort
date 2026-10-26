//! camera-beam-satchel-draw: the three undrawn pieces reach the draw lists at the right pose.
//!
//! The game's rules, cited:
//! - **The camera sight beam** — the lens proxy (param 0) is a gfx object posed at the camera's
//!   own matrix (`Mat_Copy(lens+0x90, obj+0x90)` [M Sensor_SetBeam 0x1af398:58]) and stretched to
//!   the beam's hit length through obj+0xE8: range by default (:62,72), the mask-8
//!   `Collide_RayIntersect` first-hit distance (HITDATA+0xC) when the ray hits (:78). SetBeam runs
//!   inside `Sensor_InCone` (0x1af4f0:54) — every sweep/track tick — and again in the return state
//!   (0x1af618:238). The draw is the object's plain glist path (`View_DrawObjectGlist` 0x1e6780:43-45;
//!   the Camera_Beam glist flags 0x241 [D probe] carry no billboard bit) with the create-time
//!   OnlyScaleZ bit (0x1af010:102) putting the scale on the mesh's length axis — the Camera_Beam
//!   mesh 0x02000360 is unit-length along local +z (z −0.0085..1.0283, x/y ±0.16 [D probe,
//!   "Camera_Beam" on 0700000a/14/4a]).
//! - **The 4a satchel charges** — the shooters fire weapon 0x3d, whose record family (+0x02) is
//!   0x3b [D cameras-turrets.md §3 + bullet_range_census]; the projectile mesh `weapon+0x5C`
//!   0x020007b3 "OBJECT_SatchelCharge" draws in flight (`Bullet_update` state 0's
//!   `hashtable_set_object_to_entity_gfx`, disasm 0x12e8f0 [M]) and, landed, aligned to the hit
//!   normal ([M Bullet_CollisionHandler 0x12ce80:256-266]); the two countdown digit objects
//!   `SatchelDigits[sec/10]/[sec%10]` spawn EVERY tick in every state < 2 ([M Bullet_DoTrails
//!   0x12d890:418-553], sec = fuse/60 :421-422) with the per-quarter-second beep 0x447 (:435-443).
//! - **The GT turret wreck** — DESTROYED case 6 smokes every 10th game frame:
//!   `Gas_Init(0.05, 0.25, cel, obj+0x30, rot-euler, 0, glist 0x2000039, 7, 0x48403080, 1)`
//!   [M GT_Update 0x180498 case 6, decompile L440-451] — the kind-7 shape the searchlight site
//!   already pins (life 128, the decompile-dropped 11th arg [M 0x1cbce0 sibling]).
//!
//! Opt-in: `NF_ISO`. NF_SILENT everywhere. All three FAIL on the pre-task tree (the beam slot
//! did not exist; the landed charge collapsed to a zero matrix and the drone-fired charges had
//! no digits; the wreck smoke was never emitted).

use nf_game::gt::gt_state;
use nf_game::player::Input;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn settle(scene: &mut nf_view::scene::Scene, cap: u32) {
    for _ in 0..cap {
        scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false);
        if !scene.sim.as_ref().unwrap().disabled { break; }
    }
}

fn opts() -> nf_view::scene::ViewOpts { nf_view::scene::ViewOpts::default() }

/// The camera slot layout: doors + SSs + breakables, then 2 slots per lock, GUNIMP_SLOTS per
/// gun emplacement, then 3 per security camera (body, mount, beam) — `lock_slot`'s chain.
fn camera_slot(scene: &nf_view::scene::Scene, d: &nf_game::drone::Drones, si: usize, part: usize) -> usize {
    let base = d.doors.list.len() + d.sss.list.len() + d.breaks.list.len()
        + 2 * d.locks.len()
        + (1 + nf_game::gunimp::TURRET_ARTS.len()) * scene.gunimp_placements;
    base + 3 * si + part
}

#[test]
fn the_camera_sight_beam_draws_at_the_pose_stretched_to_its_hit_length() {
    let Ok(_) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set — skipping"); return };
    std::env::set_var("NF_SILENT", "1");
    let mut scene = nf_view::load(&iso(), "0700000a.bin", false).expect("load 0700000a");
    settle(&mut scene, 300);
    let si = scene.sim.as_ref().unwrap().drones.sensors.list.iter()
        .position(|s| s.lens != 0 && s.state != 3).expect("0700000a carries lens-armed sensors [D probe]");
    let (beam_len, range, mtx, pos) = {
        let s = scene.sim.as_ref().unwrap();
        let sn = &s.drones.sensors.list[si];
        // The SetBeam write [M 0x1af398:72-78]: either the range (clear ray) or a hit distance.
        assert!(sn.beam_len > 0.0 && sn.beam_len <= sn.range + 1e-3,
            "lens obj+0xE8 = range or the ray hit distance, got {} (range {})", sn.beam_len, sn.range);
        (sn.beam_len, sn.range, sn.matrix, sn.pos)
    };
    // The beam length is LIVE — a wall inside the 5..7-unit ranges must shorten it on some
    // sweep pose. (Not pinned: which pose hits depends on the level; the bound is the [M] pin.)
    let _ = (range, pos);
    // The draw chain: the third camera slot's palette matrix is the camera pose with the z row
    // scaled to the beam length, and the slot's object-space verts are the Camera_Beam mesh [D].
    let sim = scene.sim.as_ref().unwrap();
    let slot = camera_slot(&scene, &sim.drones, si, 2);
    let (layout, _attrs) = scene.skin_layout();
    let (pal, _cols) = scene.skin_palette(&layout, None, 240.0, opts(), None);
    let m = pal[layout.door_base as usize + slot];
    assert!(m[3][0] > -9000.0, "the beam slot is not collapsed (the old tree had no third slot)");
    for r in 0..2 {
        let d: f32 = (0..3).map(|c| (m[r][c] - mtx[r][c]).abs()).fold(0.0, f32::max);
        assert!(d < 1e-3, "beam palette row {r} matches the camera pose obj+0x90 [M 0x1af398:58]");
    }
    // Row 2 keeps the camera's forward DIRECTION (the unscaled pose's row 2) — the length is
    // the obj+0xE8 stretch, asserted next.
    let mtx_len: f32 = (mtx[2][0] * mtx[2][0] + mtx[2][1] * mtx[2][1] + mtx[2][2] * mtx[2][2]).sqrt();
    let zlen0: f32 = (m[2][0] * m[2][0] + m[2][1] * m[2][1] + m[2][2] * m[2][2]).sqrt();
    let dir_dot = (m[2][0] * mtx[2][0] + m[2][1] * mtx[2][1] + m[2][2] * mtx[2][2]) / (zlen0 * mtx_len);
    assert!(dir_dot > 0.999, "the beam's z keeps the camera's forward direction (dot {dir_dot})");
    let zlen = zlen0;
    assert!((zlen - beam_len).abs() < 1e-3,
        "the z row carries the obj+0xE8 stretch: {zlen} vs beam_len {beam_len} [M 0x1af398:72-78 + OnlyScaleZ 0x1af010:102]");
    let verts: Vec<[f32; 3]> = scene.char_verts.iter().filter(|v| v.door == Some(slot as u16)).map(|v| v.pos).collect();
    assert!(!verts.is_empty(), "the beam slot carries vertices");
    let max_z = verts.iter().fold(f32::MIN, |a, p| a.max(p[2]));
    let max_xy = verts.iter().fold(0.0f32, |a, p| a.max(p[0].abs()).max(p[1].abs()));
    assert!(max_z > 0.9 && max_z < 1.1 && max_xy < 0.25,
        "the slot's mesh is the unit-length Camera_Beam [D probe 0x02000360 z −0.0085..1.0283 x/y ±0.16], got z {max_z} xy {max_xy}");
}

#[test]
fn the_4a_satchel_draws_in_flight_landed_with_digits_and_beeps() {
    let Ok(_) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set — skipping"); return };
    std::env::set_var("NF_SILENT", "1");
    let mut scene = nf_view::load(&iso(), "0700004a.bin", false).expect("load 0700004a");
    settle(&mut scene, 300);
    // The real arm path: shooter #286's enable channel 0x22 [D cameras-turrets.md §3] — the
    // game's switch write; the shooter's tick fires `Bullet_init(0, 0, obj, weapon 0x3d…)`
    // straight down [M Shooter_Update 0x17d0c0:28-37].
    scene.sim.as_mut().unwrap().drones.set_channel(0x22);
    let mut spawned = None;
    for _ in 0..90 {
        scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false);
        let s = scene.sim.as_ref().unwrap();
        if let Some(i) = s.drones.bullets.iter().position(|b| b.family(&s.weapons) == 0x3b) { spawned = Some(i); break; }
    }
    let sim = scene.sim.as_ref().unwrap();
    let bi = spawned.expect("the shooter fired its 0x3b charge on the arm channel [M Shooter_Update 0x17d0c0:28-37]");
    // In flight: the DoTrails tail wrote the countdown digits [M 0x12d890:418-422] and the
    // quarter-second beep reaches the sound list [M :435-443] (first gate ~(900−896) ticks in).
    assert!(sim.drones.bullets[bi].digits.is_some(), "the DoTrails tail wrote digits in flight");
    drop(sim);
    let mut beeped = false;
    for _ in 0..60 {
        scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false);
        if scene.sim.as_ref().unwrap().drones.sounds.iter().any(|s| s.id == 0x447) { beeped = true; break; }
    }
    assert!(beeped, "the countdown beep 0x447 reached the sound list [M 0x12d890:441-443]");
    // The projectile-mesh attachment carries the satchel mesh for this bullet index.
    let (sim, want_mesh, bpos) = { let s = scene.sim.as_ref().unwrap();
        let w = s.weapons.projectile_mesh(s.drones.bullets[bi].weapon as usize);
        assert_eq!(w, 0x0200_07b3, "the satchel's weapon+0x5C mesh [D probe OBJECT_SatchelCharge]");
        (bi, w, s.drones.bullets[bi].pos) };
    let _ = bpos;
    // The charge falls at ~4 u/s [L this run's trace] — bring it to a floor through the same
    // flight path (the sim-state technique the render tests use): teleport just above the
    // first surface straight below, then let Bullet_update's world-hit land it.
    let fl = scene.sim.as_ref().unwrap().world.floor_below(
        scene.sim.as_ref().unwrap().drones.bullets[bi].pos, 400.0).expect("a floor under the charge");
    scene.sim.as_mut().unwrap().drones.bullets[bi].pos = [
        scene.sim.as_ref().unwrap().drones.bullets[bi].pos[0], fl + 0.5,
        scene.sim.as_ref().unwrap().drones.bullets[bi].pos[2]];
    let mut landed_i = None;
    for _ in 0..300 {
        scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false);
        let s = scene.sim.as_ref().unwrap();
        if s.drones.bullets.get(bi).is_some_and(|b| b.landed) { landed_i = Some(bi); break; }
    }
    let sim = scene.sim.as_ref().unwrap();
    let bi = landed_i.expect("the charge landed on the floor within 5 s of the drop");
    let b = &sim.drones.bullets[bi];
    // The stick normal + the digits + the fuse shape [M 0x12ce80:256-266, 0x12d890:418-422].
    let n = b.stick_normal.expect("the landing captured the surface normal");
    let nl = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
    assert!(nl > 0.99, "the stick normal is a unit normal, len {nl}");
    let (tens, units) = b.digits.expect("the DoTrails tail wrote the countdown digits");
    let sec = tens as i32 * 10 + units as i32;
    assert!((0..=15).contains(&sec), "sec = fuse/60 of the 900-tick fuse [M 0x12d890:421-422], got {sec}");
    let bpos = b.pos;
    // The draw chain: the landed charge's mesh sits at the charge, z-aligned to −normal.
    let (layout, _attrs) = scene.skin_layout();
    let (pal, _cols) = scene.skin_palette(&layout, None, 240.0, opts(), None);
    let ai = scene.attachments.iter()
        .position(|a| a.projectile == Some(((nf_view::scene::PROJECTILE_SLOT_BASE + bi) as u16, want_mesh)))
        .expect("a projectile slot carries the satchel mesh for the landed bullet's index");
    let m = pal[layout.attach_base as usize + ai];
    let tr = (m[3][0] - bpos[0], m[3][1] - bpos[1], m[3][2] - bpos[2]);
    let tr2 = tr.0 * tr.0 + tr.1 * tr.1 + tr.2 * tr.2;
    assert!(tr2 < 0.25, "the landed charge's mesh sits at the charge (d² {tr2}) — the old tree collapsed it to a zero matrix");
    let zlen: f32 = (m[2][0] * m[2][0] + m[2][1] * m[2][1] + m[2][2] * m[2][2]).sqrt();
    let dot = m[2][0] * -n[0] + m[2][1] * -n[1] + m[2][2] * -n[2];
    assert!((zlen - 1.0).abs() < 1e-3 && dot > 0.99,
        "the landed pose aligns z to −normal [M 0x12ce80:256-266] (zlen {zlen} dot {dot})");
    // The digits draw: kind-4/5 effect slots with the sec's LED meshes near the charge.
    for (kind, digit) in [(4u8, tens), (5, units)] {
        let di = scene.attachments.iter()
            .position(|a| a.effect == Some((kind, 0, 0x0200_0791 + digit as u32)))
            .unwrap_or_else(|| panic!("kind-{kind} digit slot carries LED {digit} [M 0x12d890:485-553]"));
        let dm = pal[layout.attach_base as usize + di];
        let d2: f32 = (0..3).map(|c| dm[3][c] - bpos[c]).map(|d| d * d).sum();
        assert!(d2 < 1.0, "the {kind} digit draws at the charge (d² {d2}) — the old tree drew no digits for drone-fired charges");
    }
}

#[test]
fn the_gt_wreck_smokes_the_case6_gas_init() {
    let Ok(_) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set — skipping"); return };
    std::env::set_var("NF_SILENT", "1");
    let mut scene = nf_view::load(&iso(), "0700000a.bin", false).expect("load 0700000a");
    settle(&mut scene, 300);
    let ti = scene.sim.as_ref().unwrap().drones.gt_turrets.iter()
        .position(|t| t.param4 == 18 && t.facing == 1).expect("the fixture wall gun (ch 18, facing 1)");
    // The kill: health is a separate field from the 0xffff invulnerable selector [M 0017ffa0:92-96].
    scene.sim.as_mut().unwrap().drones.gt_turrets[ti].apply_damage(101.0);
    scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false);
    assert_eq!(scene.sim.as_ref().unwrap().drones.gt_turrets[ti].state, gt_state::DESTROYED,
        "the death gate flips to DESTROYED [M 0x180498:188-198]");
    // The case-6 smoke: a kind-7 puff, scale 0.05 → 0.25, rgba 0x48403080, mesh 0x2000039 —
    // the exact Gas_Init args [M 0x180498 case 6], life 128 [M 0x1cbce0 sibling].
    let mut pi = None;
    for _ in 0..40 {
        scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false);
        let s = scene.sim.as_ref().unwrap();
        pi = s.drones.gas.puffs.iter().position(|p| p.kind == 7 && p.mesh == 0x0200_0039 && p.scale == 0.05);
        if pi.is_some() { break; }
    }
    let sim = scene.sim.as_ref().unwrap();
    let pi = pi.expect("the wreck smoke entered the gas pool within 40 ticks of the %10 gate");
    let puff = &sim.drones.gas.puffs[pi];
    assert_eq!(puff.rgba, [72.0, 64.0, 48.0, 128.0], "the Gas_Init colour 0x48403080 [M case 6]");
    assert_eq!(puff.life, 128.0, "the kind-7 life 128 [M 0x1cbce0 sibling]");
    // The draw: the gas slot whose attachment reads this pool index sits at the puff.
    let (layout, _attrs) = scene.skin_layout();
    let (pal, cols) = scene.skin_palette(&layout, None, 240.0, opts(), None);
    let ai = scene.attachments.iter().position(|a| a.gas.map(|(s, m)| s as usize == pi && m == 0x0200_0039).unwrap_or(false))
        .expect("a gas draw slot reads the wreck puff's pool index");
    let m = pal[layout.attach_base as usize + ai];
    let d2: f32 = (0..3).map(|c| m[3][c] - puff.pos[c]).map(|d| d * d).sum();
    assert!(d2 < 4.0, "the wreck puff's billboard sits at the puff (d² {d2})");
    let c = cols[layout.col_att as usize + ai];
    assert!((c[0] - 72.0 / 255.0).abs() < 1e-4 && (c[3] - 128.0 / 255.0).abs() < 1e-4,
        "the colour slot carries the live rgba/255 [M View_SetupRenderModes_Tweak 0x1e5888:46-49]");
}
