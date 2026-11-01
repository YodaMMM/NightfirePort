//! The tripwire objects on 0700000a / 0700000b / 07000014 and the placed laser tripbomb shot from
//! range.
//!
//! The 0xFE placement kind is `Mine_Create` 0x17ce28's only creator [M
//! `parsemap_create_dynamic_objects` 0x1d04f0 case 0xfe, the only call site of Mine_Create in
//! ACTION.ELF]. The census below walks every map of the three levels: none carries a 0xFE
//! placement [D]. The level beams there are 0xEB `laserbeam` touch triggers (mask 8, the player
//! touch [D docs/levels/0700000a.md, 0700000b.md]); the shootable laser on these levels is the
//! class-5 tripbomb the 0x3a weapon places, whose shot-from-range rule is
//! `Bullet_handle_object_destruction` 0x12cd98 [M], not `Mine_Update` 0x17c930. The 0xFE leg (a
//! round from range detonates the mine) is pinned on 07000015 by walkreal_07000015_live.
//!
//! The tripbomb leg (07000014 — the level carries the gadget's model 0x0200074f and two
//! PICKUP_Tripbomb placements #1341/#1342 [D]; 0700000a/0700000b's archives carry no 0x0200074f
//! object [D the archive census]):
//! - the round's pick: `Collide_PickObj` 0x1eae80's 0x201 arm against the gadget's world sphere
//!   (`Control_BuildWorldSph` 0x133070: glist+0x10 posed, radius glist+0x1C · |obj+0xE8| — 0.24266
//!   [D], obj+0xE8 = 1.0 [M 0x12e7b0:122]) then `Collide_Intersect` 0x1ebaa8's object-space mesh
//!   refine against the glist's 1312 collision tris [D];
//! - the take: `Bullet_CollisionHandler` 0x12ce80's object arm (state 2) → 0x12cd98 → the gadget
//!   deletes, the 8/100 blast with script 0x0600_0052 from the gadget's own record, and the beam
//!   object (BU+0x38) deletes with it (the record's +0x70 & 0x80000) [M].
//!
//! Opt-in: set `NF_ISO` to the disc image. `NF_TRIPBOMB_SHOTS=<dir>` also writes before/after
//! renders of the shot as PNGs.

use nf_formats::{disc, iso9660::IsoImage, level::Level};
use nf_game::player::{Bond, Input};
use nf_view::gpu;
use nf_view::scene::{Scene, Sim, ViewOpts};

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

/// Every map's 0xFE placement count, per level archive.
fn mine_placements(iso: &IsoImage, lvl: &str) -> (usize, usize) {
    let (entries, bin) = disc::files_bin_entries(iso).unwrap();
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case(lvl)).unwrap_or_else(|| panic!("{lvl} in FILES.BIN"));
    let data = iso.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).unwrap();
    let level = Level::parse(&data).unwrap();
    let n = level.maps.iter().map(|m| m.placements.iter().filter(|p| p.flags & 0xff == 0xfe).count()).sum();
    (level.maps.len(), n)
}

#[test]
fn the_tripwire_levels_carry_no_0xfe_mine() {
    let Some(iso_path) = iso() else { eprintln!("skip: NF_ISO not set"); return };
    let iso = IsoImage::open(&iso_path).unwrap();
    for (lvl, maps) in [("0700000a.bin", 38), ("0700000b.bin", 51), ("07000014.bin", 55)] {
        let (n_maps, mines) = mine_placements(&iso, lvl);
        assert_eq!(n_maps, maps, "{lvl}: the census walks every map [D]");
        assert_eq!(mines, 0, "{lvl}: no 0xFE placement on any map [D]");
    }
    // Sister: the same count finds the 0xFE mines where they exist (the nine Tripbomb_3rd cels).
    let (_, mines15) = mine_placements(&iso, "07000015.bin");
    assert_eq!(mines15, 9, "sister: 07000015 carries the nine 0xFE mines [D census_07000015]");
}

/// The keyed laser beams of the three levels (the 3-key moving `laserbeam` triggers [D
/// goggles_draws_live: 0700000a 3, 0700000b 8, 07000014 8]) carry no mask bit 2: `Trigger_Hit`
/// 0x1b1fc8:124-163 never reads a class-5 (bullet) contact for them, so a round through one does
/// nothing in the game — only the touch bits fire them [M]. Sister: the counts pin the filter.
#[test]
fn the_level_laser_beams_ignore_rounds() {
    let Some(iso) = iso() else { eprintln!("skip: NF_ISO not set"); return };
    for (lvl, keyed) in [("0700000a.bin", 3usize), ("0700000b.bin", 8), ("07000014.bin", 8)] {
        let scene = nf_view::load(&iso, lvl, false).expect("load");
        let sim = scene.sim.as_ref().unwrap();
        let beams: Vec<_> = sim.drones.triggers.iter().filter(|t| t.motion.is_some()).collect();
        assert_eq!(beams.len(), keyed, "{lvl}: the keyed beam count [D]");
        for t in &beams {
            eprintln!("[beams] {lvl} #{} mask {:#x}", t.placement, t.mask);
            assert_eq!(t.mask & 2, 0, "{lvl} #{}: no bullet bit (mask {:#x})", t.placement, t.mask);
        }
    }
}

const P2K: u8 = 0x02;
const TRIP: u8 = 0x3a;
const MESH: u32 = 0x0200_074f;

type V3 = [f32; 3];
fn sub(a: V3, b: V3) -> V3 { [a[0] - b[0], a[1] - b[1], a[2] - b[2]] }
fn add(a: V3, b: V3) -> V3 { [a[0] + b[0], a[1] + b[1], a[2] + b[2]] }
fn mul(a: V3, s: f32) -> V3 { [a[0] * s, a[1] * s, a[2] * s] }
fn dot(a: V3, b: V3) -> f32 { a[0] * b[0] + a[1] * b[1] + a[2] * b[2] }
fn len(a: V3) -> f32 { dot(a, a).sqrt() }
fn norm(a: V3) -> V3 { let l = len(a).max(1e-9); mul(a, 1.0 / l) }

fn eye(sim: &Sim) -> V3 { [sim.bond.pos[0], sim.bond.pos[1] + sim.bond.eye - sim.bond.crouch_drop, sim.bond.pos[2]] }

/// Turn Bond's view straight onto `at` (the yaw/pitch the fire path reads).
fn face(sim: &mut Sim, at: V3) {
    let d = norm(sub(at, eye(sim)));
    sim.bond.yaw = d[0].atan2(d[2]);
    sim.bond.pitch = d[1].clamp(-1.0, 1.0).asin() * 2.0 / std::f32::consts::PI;
}

/// Give `id` through the dev panel's weapon give and draw it with the real switch request.
fn give_and_draw(scene: &mut Scene, id: u8, rounds: u32) {
    let (_, switch_to) = nf_view::panel::apply(scene, &[
        ("do".to_string(), "weapon".to_string()), ("id".to_string(), id.to_string()), ("rounds".to_string(), rounds.to_string())]);
    assert_eq!(switch_to, Some(id), "the panel switches to {id:#04x}");
    for _ in 0..240 {
        scene.sim.as_mut().unwrap().step_bond(Input { switch_to: Some(id), ..Default::default() }, false, false, false);
        let gun = &scene.sim.as_ref().unwrap().gun;
        if gun.weapon == id && gun.switch_state == 0 && gun.reload_state == 0 { return; }
    }
    panic!("{id:#04x} never drew");
}

/// The live stuck tripbomb's index.
fn tripbomb(sim: &Sim) -> Option<usize> {
    sim.drones.grenades.iter().position(|g| g.weapon_type == TRIP as u16 && g.stuck && g.state < 2)
}

/// The gadget's world sphere centre (obj+0x80): its pose · glist+0x10 [M 0x133070:36-40].
fn body_centre(sim: &Sim, gi: usize) -> V3 {
    let g = &sim.drones.grenades[gi];
    let m = g.pose();
    let c = sim.models.get(&MESH).expect("the tripbomb model's glist bounds").centre;
    let mut out = [m[3][0], m[3][1], m[3][2]];
    for k in 0..3 { out[k] += c[0] * m[0][k] + c[1] * m[1][k] + c[2] * m[2][k]; }
    out
}

/// Bond's distance to the beam segment of the armed tripbomb `gi` (the 1.0-u trigger test's input).
fn beam_dist(sim: &Sim, p: V3) -> f32 {
    sim.drones.tripbomb_beams.iter().map(|&(a, b)| {
        let ab = sub(b, a);
        let t = (dot(sub(p, a), ab) / dot(ab, ab).max(1e-9)).clamp(0.0, 1.0);
        len(sub(add(a, mul(ab, t)), p))
    }).fold(f32::INFINITY, f32::min)
}

/// The corridor the shot runs down: the long east-west service corridor south of the level's
/// centre (open 40+ u west of the guard post at (12.24, 1.6, −22.34), ~7 u wide [the
/// level's own collision, probed from the guard spawns]).
const CORRIDOR: V3 = [12.24, 1.6, -22.34];

/// The scenario on 07000014: Bond stands 2.5 u short of the corridor's west end wall and throws a
/// real tripbomb at it (the 0x3a fire pipeline); it sticks and arms; Bond then stands ≥ 15 u down
/// the corridor, off the beam, with a clear line to the body, holding the P2K. The level's guards
/// are parked (`enabled = false`) so none engages Bond or crosses the beam during the setup.
/// Returns the scene and the gadget index.
fn armed_tripbomb_and_a_far_shooter(iso: &str) -> (Scene, usize) {
    let mut scene = nf_view::load(iso, "07000014.bin", false).expect("load 07000014");
    {
        let sim = scene.sim.as_mut().unwrap();
        sim.disabled = false;
        for d in sim.drones.list.iter_mut() { d.enabled = false; }
        for _ in 0..5 { sim.step_bond(Input::default(), false, false, false); }
    }
    give_and_draw(&mut scene, TRIP, 20);
    let mut placed = None;
    {
        let sim = scene.sim.as_mut().unwrap();
        // The corridor's west end wall (the long open run: 40 u at head height).
        let far = add(CORRIDOR, [-60.0, 0.0, 0.0]);
        let (t, n) = sim.world.segment_hit(CORRIDOR, far).expect("the corridor's west end wall");
        let wall_x = CORRIDOR[0] - 60.0 * t;
        eprintln!("[tripbomb] west end wall at x {wall_x}, normal {n:?}");
        assert!(CORRIDOR[0] - wall_x > 20.0 && n[0] > 0.9, "the end wall faces back down the corridor ({wall_x}, {n:?})");
        let stand = [wall_x + 2.5, CORRIDOR[1], CORRIDOR[2]];
        let top = add(stand, [0.0, 1.0, 0.0]);
        let tf = sim.world.segment(top, sub(top, [0.0, 6.0, 0.0])).expect("a floor at the throw spot");
        sim.bond = Bond::new([stand[0], top[1] - 6.0 * tf + 1.1, stand[2]], -std::f32::consts::FRAC_PI_2);
        for _ in 0..10 { sim.step_bond(Input::default(), false, false, false); }
        let e = eye(sim);
        face(sim, [wall_x, e[1] + 0.1, e[2]]);
        for _ in 0..240 {
            sim.step_bond(Input { fire: true, ..Input::default() }, true, false, false);
            if sim.drones.grenades.iter().any(|g| g.weapon_type == TRIP as u16) { break; }
        }
        for _ in 0..120 {
            sim.step_bond(Input::default(), false, false, false);
            if let Some(gi) = tripbomb(sim) {
                let nrm = sim.drones.grenades[gi].phase.as_ref().unwrap().normal;
                if nrm[0] > 0.9 { placed = Some(gi); }
                break;
            }
        }
    }
    let gi = placed.expect("the real tripbomb throw sticks to the end wall");
    // The shooter's spot: in front of the wall (w·n ≥ 0.35), 16 u out, ≥ 3 u off the beam line,
    // standing on a floor, with a clear line from the eye to the body centre.
    let (pos, nrm) = {
        let sim = scene.sim.as_ref().unwrap();
        let g = &sim.drones.grenades[gi];
        (g.pos, g.phase.as_ref().unwrap().normal)
    };
    let side = norm([nrm[2], 0.0, -nrm[0]]);
    let mut spot = None;
    'spots: for dist in [16.0f32, 18.0, 20.0, 17.0, 22.0] {
        for k in 0..32u32 {
            let a = (k as f32 / 32.0 - 0.5) * std::f32::consts::PI; // −90°..+90° about the normal
            let w = norm(add(mul(norm([nrm[0], 0.0, nrm[2]]), a.cos()), mul(side, a.sin())));
            if dot(w, nrm) < 0.35 { continue; }
            let off = mul(w, dist);
            let beam_off = len(sub(off, mul(nrm, dot(off, nrm))));
            if beam_off < 3.0 { continue; }
            let xz = add(pos, off);
            let sim = scene.sim.as_ref().unwrap();
            let top = [xz[0], pos[1] + 1.5, xz[2]];
            let Some(tf) = sim.world.segment(top, sub(top, [0.0, 6.0, 0.0])) else { continue };
            let floor = [top[0], top[1] - 6.0 * tf, top[2]];
            if (floor[1] - pos[1]).abs() > 3.0 { continue; }
            spot = Some(([floor[0], floor[1] + 1.1, floor[2]], dist));
            let _ = &spot;
            // Check it live: stand, settle, then the line of sight and the beam clearance.
            let mut trial = Bond::new(spot.unwrap().0, 0.0);
            trial.yaw = sub(pos, floor)[0].atan2(sub(pos, floor)[2]);
            let s2 = scene.sim.as_mut().unwrap();
            let keep = s2.bond.clone();
            s2.bond = trial;
            for _ in 0..10 { s2.step_bond(Input::default(), false, false, false); }
            let target = body_centre(s2, gi);
            let e = eye(s2);
            let clear = s2.world.segment(e, target).is_none_or(|t| t > 0.995);
            let far_enough = len(sub(target, e)) >= 15.0;
            let moved = len(sub(s2.bond.pos, spot.unwrap().0)) < 0.5;
            if clear && far_enough && moved && tripbomb(s2) == Some(gi) { break 'spots; }
            s2.bond = keep;
            spot = None;
        }
    }
    let (_, dist) = spot.expect("a shooter spot 16+ u out with a clear line to the tripbomb");
    let sim = scene.sim.as_mut().unwrap();
    // The beam arms 120 ticks after the stick; wait it out at the spot.
    for _ in 0..200 {
        if sim.drones.grenades[gi].phase.as_ref().unwrap().armed && !sim.drones.tripbomb_beams.is_empty() { break; }
        sim.step_bond(Input::default(), false, false, false);
    }
    assert!(sim.drones.grenades[gi].phase.as_ref().unwrap().armed, "the tripbomb armed");
    assert!(!sim.drones.tripbomb_beams.is_empty(), "the armed tripbomb draws its beam");
    assert!(beam_dist(sim, sim.bond.pos) > 2.0, "Bond stands clear of the beam");
    eprintln!("[tripbomb] stuck at {pos:?} normal {nrm:?}; Bond at {:?}, {dist} u out", sim.bond.pos);

    give_and_draw(&mut scene, P2K, 40);
    let sim = scene.sim.as_ref().unwrap();
    assert_eq!(tripbomb(sim), Some(gi), "the tripbomb survives the setup");
    (scene, gi)
}

/// One real P2K round at `target`: the view turned onto it, the trigger pulled (pressed and
/// released on alternate ticks, the refire edge) through `step_bond` until a round leaves the gun,
/// then stepped until its flight is over. Returns whether a round flew.
fn fire_one(scene: &mut Scene, target: V3) -> bool {
    let sim = scene.sim.as_mut().unwrap();
    assert_eq!(sim.gun.weapon, P2K, "the P2K is drawn");
    face(sim, target);
    let mut flew = false;
    for t in 0..240u32 {
        let pull = !flew && t % 2 == 0;
        sim.step_bond(if pull { Input { fire: true, ..Input::default() } } else { Input::default() }, pull, false, false);
        let live = !sim.drones.player_bullets.is_empty();
        if flew && !live { break; }
        flew |= live;
    }
    flew
}

fn blast_script_playing(sim: &Sim) -> bool { sim.drones.scripts.players.iter().any(|p| p.scripts[0] == 0x0600_0052) }

/// A real P2K round from 16+ u at the stuck tripbomb's body detonates it: the gadget leaves the
/// list, the 0x0600_0052 blast script plays, and the beam is gone [M 0x12cd98]. Before the fix
/// the round flew through the gadget into the wall and nothing happened.
#[test]
fn a_p2k_round_from_range_detonates_the_stuck_tripbomb() {
    let Some(iso) = iso() else { eprintln!("skip: NF_ISO not set"); return };
    let (mut scene, gi) = armed_tripbomb_and_a_far_shooter(&iso);
    let (target, cam, dist) = {
        let sim = scene.sim.as_ref().unwrap();
        let t = body_centre(sim, gi);
        let e = eye(sim);
        (t, e, len(sub(t, e)))
    };
    assert!(dist >= 15.0, "the shot is from 15+ u ({dist})");
    assert!(!blast_script_playing(scene.sim.as_ref().unwrap()), "no blast before the shot");
    shot(&scene, cam, target, "before.png");
    let mut went = None;
    for k in 0..12u32 {
        assert!(fire_one(&mut scene, target), "round {} flew", k + 1);
        if tripbomb(scene.sim.as_ref().unwrap()).is_none() { went = Some(k + 1); break; }
    }
    let rounds = went.expect("a P2K round at the body from range detonates the stuck tripbomb");
    eprintln!("[tripbomb] detonated by round {rounds} from {dist:.2} u");
    let sim = scene.sim.as_ref().unwrap();
    assert!(sim.drones.grenades.iter().all(|g| g.weapon_type != TRIP as u16), "the gadget deleted (obj+0xFE |= 1)");
    assert!(blast_script_playing(sim), "the 0x0600_0052 blast plays from the gadget's record [M 0x12cd98]");
    assert!(sim.drones.tripbomb_beams.is_empty(), "the beam deleted with it (BU+0x38, +0x70 & 0x80000)");
    shot(&scene, cam, target, "after.png");
}

/// Sister: the same setup, rounds aimed at the wall 1.5 u beside the gadget (beyond its 0.24266
/// glist radius and every P2K spread) leave it standing — armed, beam drawn, no blast.
#[test]
fn a_p2k_round_that_misses_leaves_the_tripbomb_armed() {
    let Some(iso) = iso() else { eprintln!("skip: NF_ISO not set"); return };
    let (mut scene, gi) = armed_tripbomb_and_a_far_shooter(&iso);
    let target = {
        let sim = scene.sim.as_ref().unwrap();
        let n = sim.drones.grenades[gi].phase.as_ref().unwrap().normal;
        add(body_centre(sim, gi), mul(norm([n[2], 0.0, -n[0]]), 1.5))
    };
    let mut fired = 0;
    for _ in 0..6 {
        if fire_one(&mut scene, target) { fired += 1; }
    }
    assert!(fired >= 6, "six rounds left the gun");
    let sim = scene.sim.as_ref().unwrap();
    assert_eq!(tripbomb(sim), Some(gi), "the missed tripbomb stays placed");
    assert!(sim.drones.grenades[gi].phase.as_ref().unwrap().armed, "still armed");
    assert!(!sim.drones.tripbomb_beams.is_empty(), "its beam still drawn");
    assert!(!blast_script_playing(sim), "no blast");
}

/// Render the scene from `cam` looking at `at` (the glass_guards_14 harness) and write it to
/// `$NF_TRIPBOMB_SHOTS/<name>` when that is set.
fn shot(scene: &Scene, cam: V3, at: V3, name: &str) {
    let Some(dir) = std::env::var_os("NF_TRIPBOMB_SHOTS") else { return };
    let d = norm(sub(at, cam));
    let (yaw, pitch) = (d[0].atan2(d[2]), d[1].clamp(-1.0, 1.0).asin());
    let (w, h) = (1280u32, 720u32);
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
    let (device, queue) = pollster::block_on(async {
        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions::default()).await.expect("adapter");
        adapter.request_device(&Default::default()).await.expect("device")
    });
    let format = wgpu::TextureFormat::Rgba8Unorm;
    let mut r = gpu::Renderer::new(&device, &queue, format, scene, false, false);
    let cam = glam::Vec3::from(cam);
    let cp = pitch.cos();
    let fw = glam::Vec3::new(cp * yaw.sin(), pitch.sin(), cp * yaw.cos()).normalize();
    let s = fw.cross(glam::Vec3::Y).normalize();
    let u = s.cross(fw);
    // A narrow field (≈ 12°) so the 0.46-u gadget at 16+ u is visible.
    let fov = 0.21f32;
    let f = 1.0 / (fov * 0.5).tan();
    let proj = glam::Mat4::from_cols_array_2d(&[
        [f / (w as f32 / h as f32), 0.0, 0.0, 0.0], [0.0, f, 0.0, 0.0],
        [0.0, 0.0, 4000.0 / (0.05 - 4000.0), -1.0], [0.0, 0.0, 0.05 * 4000.0 / (0.05 - 4000.0), 0.0]]);
    let view = glam::Mat4::from_cols_array_2d(&[
        [s.x, u.x, -fw.x, 0.0], [s.y, u.y, -fw.y, 0.0], [s.z, u.z, -fw.z, 0.0],
        [-s.dot(cam), -u.dot(cam), fw.dot(cam), 1.0]]);
    let vp = (proj * view).to_cols_array_2d();
    let ticks = scene.sim.as_ref().map_or(0, |s| s.drones.frame);
    let vis = r.cel_visibility(cam.to_array(), vp, scene.sim.as_ref().map(|s| &s.world), None);
    let mut opts = ViewOpts::default();
    opts.right = s.to_array();
    opts.up = u.to_array();
    r.update_skin(&queue, scene, None, ticks as f32, opts, vis.as_deref());
    r.lights = scene.frame_lights();
    let target = device.create_texture(&wgpu::TextureDescriptor {
        label: None, size: wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 }, mip_level_count: 1, sample_count: 1,
        dimension: wgpu::TextureDimension::D2, format, usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC, view_formats: &[],
    });
    let tview = target.create_view(&Default::default());
    let depth = gpu::depth_view(&device, w, h);
    let mut enc = r.render(&device, &queue, &tview, &depth, vp, vp, cam.to_array(), ticks, &[], &gpu::HudFrame::default(), None, None);
    let row = (4 * w).next_multiple_of(256);
    let buf = device.create_buffer(&wgpu::BufferDescriptor { label: None, size: (row * h) as u64, usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ, mapped_at_creation: false });
    enc.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo { texture: &target, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
        wgpu::TexelCopyBufferInfo { buffer: &buf, layout: wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(row), rows_per_image: Some(h) } },
        wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 });
    queue.submit([enc.finish()]);
    buf.map_async(wgpu::MapMode::Read, .., |r| r.expect("map"));
    device.poll(wgpu::PollType::wait_indefinitely()).expect("poll");
    let data = buf.get_mapped_range(..).expect("map range");
    let mut rgba = Vec::with_capacity((4 * w * h) as usize);
    for y in 0..h as usize { rgba.extend_from_slice(&data[y * row as usize..][..4 * w as usize]); }
    let p = std::path::Path::new(&dir).join(name);
    std::fs::write(&p, nf_formats::png::encode_rgba(w, h, &rgba)).expect("write png");
    eprintln!("wrote {}", p.display());
}
