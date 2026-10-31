//! thrown-satchel-pose: a satchel thrown by Bond keeps `Bullet_init`'s spawn alignment in flight —
//! its mesh AND its countdown digits — and turns to the stick pose once it parks.
//!
//! The game's rules, cited:
//! - **Spawn** — `Bullet_init` aligns the bullet's matrix ONCE: `Mat_Align2Dir(obj+0x90, BU+0x00,
//!   (0,1,0), (1,0,0))` with BU+0x00 = the normalised direction plus its spread [M 0x12c5a0:180-185].
//! - **Flight** — `Bullet_update` case 1 rewrites obj+0x90 only on `+0x6C & 0x8000` (the spin) or
//!   the `+0x6C & 0x4` guided arm [M 0x12e7b0:163-217]; the gravity arm re-normalises BU+0x00 alone
//!   [M :246-265]. The satchel record's +0x6C is 0xc08 [L ee_start_05.bin weapon_data 0x3b] — the
//!   arc never turns the drawn pose.
//! - **Stick** — `Bullet_CollisionHandler` aligns obj+0x90 to −normal [M 0x12ce80:256-266].
//! - **Digits** — `Bullet_DoTrails`' 0x3b tail poses the two LEDs on obj+0x90 every tick in every
//!   state < 2 [M 0x12d890:485-553] (`satchel_digit_matrix`).
//!
//! The throw: 07000005, the satchel drawn through the panel give + the switch machine at the
//! fix-satchel-stick-play spot, one trigger tick, the real fire pipeline. Opt-in: NF_ISO. NF_SILENT.

use nf_game::player::{Bond, Input};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

const SATCHEL_MESH: u32 = 0x0200_07b3;
const LED0: u32 = 0x0200_0791;
const SPOT: [f32; 3] = [105.97, 24.87, 110.12];
const YAW: f32 = -4.369;
const PITCH: f32 = -0.166;

type Mat4 = nf_formats::anim::Mat4;

fn norm(v: [f32; 3]) -> [f32; 3] {
    let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    [v[0] / l, v[1] / l, v[2] / l]
}
fn dot(a: [f32; 3], b: [f32; 3]) -> f32 { a[0] * b[0] + a[1] * b[1] + a[2] * b[2] }
fn pose(dir: [f32; 3], pos: [f32; 3]) -> Mat4 {
    let mut m = nf_game::searchlight::align2dir(dir, [0.0, 1.0, 0.0], [1.0, 0.0, 0.0]);
    m[3] = [pos[0], pos[1], pos[2], 1.0];
    m
}
fn max_diff(a: &Mat4, b: &Mat4) -> f32 {
    let mut d = 0.0f32;
    for r in 0..4 { for c in 0..3 { d = d.max((a[r][c] - b[r][c]).abs()); } }
    d
}

fn draw_satchel() -> nf_view::scene::Scene {
    let mut scene = nf_view::load_weapons(&iso(), "07000005.bin", false, &[0x3b]).expect("load 05");
    let (_msg, switch_to) = nf_view::panel::apply(&mut scene, &[
        ("do".to_string(), "weapon".to_string()), ("id".to_string(), 0x3b.to_string()), ("rounds".to_string(), "5".to_string())]);
    assert_eq!(switch_to, Some(0x3b));
    let sim = scene.sim.as_mut().unwrap();
    sim.bond = Bond::new(SPOT, YAW);
    sim.bond.pitch = PITCH;
    sim.disabled = false;
    for _ in 0..180 {
        sim.step_bond(Input { switch_to: Some(0x3b), ..Default::default() }, false, false, false);
        if sim.gun.weapon == 0x3b && sim.gun.switch_state == 0 && sim.gun.reload_state == 0 { break; }
    }
    assert_eq!(sim.gun.weapon, 0x3b, "the satchel is drawn");
    for _ in 0..150 { sim.step_bond(Input::default(), false, false, false); }
    scene
}

/// The drawn matrices of the satchel's projectile slot and its two digit slots (tens, units).
fn drawn(scene: &nf_view::scene::Scene) -> (Mat4, Mat4, Mat4) {
    let sim = scene.sim.as_ref().unwrap();
    let g = sim.drones.grenades.iter().find(|g| g.weapon_type == 0x3b).unwrap();
    let (tens, units) = g.phase.as_ref().unwrap().digits.expect("the 0x3b tail wrote the digits");
    let (layout, _) = scene.skin_layout();
    let (pal, _) = scene.skin_palette(&layout, None, 240.0, nf_view::scene::ViewOpts::default(), None);
    let at = |pred: &dyn Fn(&nf_view::scene::Attachment) -> bool, what: &str| {
        let ai = scene.attachments.iter().position(|a| pred(a)).unwrap_or_else(|| panic!("no {what} slot"));
        pal[layout.attach_base as usize + ai]
    };
    let mesh = at(&|a| a.projectile == Some((0, SATCHEL_MESH)), "satchel projectile");
    let t = at(&|a| a.effect == Some((4, 0, LED0 + tens as u32)), "tens digit");
    let u = at(&|a| a.effect == Some((5, 0, LED0 + units as u32)), "units digit");
    (mesh, t, u)
}

#[test]
fn a_thrown_satchel_keeps_its_spawn_pose_in_flight_and_the_stick_pose_at_rest() {
    let Ok(_) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set — skipping"); return };
    std::env::set_var("NF_SILENT", "1");
    let mut scene = draw_satchel();
    // One trigger tick, then idle until the throw releases.
    let sim = scene.sim.as_mut().unwrap();
    sim.step_bond(Input { fire: true, ..Default::default() }, true, false, false);
    let mut spawn_vel = None;
    for _ in 0..240 {
        sim.step_bond(Input::default(), false, false, false);
        if let Some(g) = sim.drones.grenades.iter().find(|g| g.weapon_type == 0x3b) { spawn_vel = Some(g.velocity); break; }
    }
    let spawn_dir = norm(spawn_vel.expect("the throw released"));
    {
        let g = sim.drones.grenades.iter().find(|g| g.weapon_type == 0x3b).unwrap();
        let a = g.phase.as_ref().unwrap().align;
        for c in 0..3 { assert!((a[c] - spawn_dir[c]).abs() < 1e-5, "the box carries the spawn direction: {a:?} vs {spawn_dir:?}"); }
    }
    // Fly until the arc has turned the velocity well away from the spawn direction (still flying).
    let mut turned = false;
    for _ in 0..200 {
        scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false);
        let s = scene.sim.as_ref().unwrap();
        let g = s.drones.grenades.iter().find(|g| g.weapon_type == 0x3b).expect("still live");
        assert!(!g.stuck, "the satchel parked before its arc turned");
        if g.state == 1 && dot(norm(g.velocity), spawn_dir) < 0.9 { turned = true; break; }
    }
    assert!(turned, "the arc turned the flight direction more than ~25° off the spawn direction");
    let (g_pos, g_vel) = { let s = scene.sim.as_ref().unwrap(); let g = s.drones.grenades.iter().find(|g| g.weapon_type == 0x3b).unwrap(); (g.pos, g.velocity) };
    eprintln!("mid-flight: pos {g_pos:?} vel {g_vel:?} spawn dir {spawn_dir:?} tick {}", scene.sim.as_ref().unwrap().drones.frame);
    let (mesh, tens, units) = drawn(&scene);
    let want = pose(spawn_dir, g_pos);
    assert!(max_diff(&mesh, &want) < 1e-4, "the flying mesh draws at the spawn alignment: {mesh:?} vs {want:?}");
    let wt = nf_view::scene::satchel_digit_matrix(&want, g_pos, true);
    let wu = nf_view::scene::satchel_digit_matrix(&want, g_pos, false);
    assert!(max_diff(&tens, &wt) < 1e-4, "tens digit on the spawn-aligned charge: {tens:?} vs {wt:?}");
    assert!(max_diff(&units, &wu) < 1e-4, "units digit on the spawn-aligned charge: {units:?} vs {wu:?}");
    // Sister: the velocity pose (the old draw) is measurably different here — the check can fail.
    let old = pose(norm(g_vel), g_pos);
    assert!(max_diff(&mesh, &old) > 0.2, "the velocity pose differs from the spawn pose mid-flight");
    let old_t = nf_view::scene::satchel_digit_matrix(&old, g_pos, true);
    assert!(max_diff(&tens, &old_t) > 0.01, "the velocity-posed tens digit differs");

    // At rest: the stick re-aligns the charge to −normal.
    let mut stuck = None;
    for _ in 0..600 {
        scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false);
        let s = scene.sim.as_ref().unwrap();
        let g = s.drones.grenades.iter().find(|g| g.weapon_type == 0x3b).expect("still live");
        if g.stuck { stuck = Some((g.pos, g.phase.as_ref().unwrap().normal, g.phase.as_ref().unwrap().align)); break; }
    }
    let (p, n, a) = stuck.expect("the satchel stuck");
    eprintln!("stuck: pos {p:?} normal {n:?} tick {}", scene.sim.as_ref().unwrap().drones.frame);
    for c in 0..3 { assert!((a[c] + n[c]).abs() < 1e-6, "the stick re-aligns to −normal"); }
    let (mesh, tens, _) = drawn(&scene);
    let want = pose([-n[0], -n[1], -n[2]], p);
    assert!(max_diff(&mesh, &want) < 1e-4, "the stuck mesh draws at −normal");
    let wt = nf_view::scene::satchel_digit_matrix(&want, p, true);
    assert!(max_diff(&tens, &wt) < 1e-4, "the stuck tens digit rides the −normal pose");
    // Sister: the spawn pose is not the stick pose.
    assert!(max_diff(&mesh, &pose(spawn_dir, p)) > 0.2, "the spawn pose differs from the stick pose");
}

/// The drone-fired 0x3b bullet (the 4a lift shooter's dropped charge): `init_fuse` stores the spawn
/// direction and the flying charge's digits ride it. The shooter drops it straight down, so the
/// live `dir` never turns here; the sister turns `dir` the way the gravity arm does for any
/// non-vertical launch and the digits stay on the spawn pose.
#[test]
fn the_4a_shooters_charge_draws_its_digits_on_the_spawn_pose() {
    let Ok(_) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set — skipping"); return };
    std::env::set_var("NF_SILENT", "1");
    let mut scene = nf_view::load(&iso(), "0700004a.bin", false).expect("load 0700004a");
    scene.sim.as_mut().unwrap().bond = Bond::new([3.2, -7.0, -3.8], 0.0);
    let tens_matrix = |scene: &nf_view::scene::Scene, tens: u8| {
        let (layout, _) = scene.skin_layout();
        let (pal, _) = scene.skin_palette(&layout, None, 240.0, nf_view::scene::ViewOpts::default(), None);
        let ai = scene.attachments.iter().position(|a| a.effect == Some((4, 0, LED0 + tens as u32))).expect("tens slot");
        pal[layout.attach_base as usize + ai]
    };
    for _ in 0..4000 {
        scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false);
        let s = scene.sim.as_ref().unwrap();
        let Some(bi) = s.drones.bullets.iter().position(|b| b.family(&s.weapons) == 0x3b && !b.landed && b.state < 2 && b.digits.is_some()) else { continue };
        let b = s.drones.bullets[bi].clone();
        let sd = b.spawn_dir.expect("init_fuse stored the spawn direction");
        assert!(dot(sd, b.dir) > 0.9999, "the straight drop keeps dir on the spawn direction: {sd:?} vs {:?}", b.dir);
        let (tens, _) = b.digits.unwrap();
        let want = nf_view::scene::satchel_digit_matrix(&pose(sd, b.pos), b.pos, true);
        assert!(max_diff(&tens_matrix(&scene, tens), &want) < 1e-4, "the falling charge's tens digit rides the spawn pose");
        // Sister: an arc-turned dir (45° off the drop) leaves the drawn digits on the spawn pose, and
        // the dir-following pose differs from it.
        let turned = norm([0.0, -1.0, 1.0]);
        scene.sim.as_mut().unwrap().drones.bullets[bi].dir = turned;
        let m = tens_matrix(&scene, tens);
        assert!(max_diff(&m, &want) < 1e-4, "a turned dir does not turn the digits");
        let old = nf_view::scene::satchel_digit_matrix(&pose(turned, b.pos), b.pos, true);
        assert!(max_diff(&m, &old) > 0.05, "the dir-following pose differs");
        return;
    }
    panic!("the shooter's charge flew within 4000 ticks of the ride");
}
