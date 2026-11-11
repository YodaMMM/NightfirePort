//! The guard's body hitting the ground plays the surface's body sound, and a headless run's sound
//! pool frees its one-shots by their length.
//!
//! Evidence [M]:
//! - `AnimProcessScriptCmds` 0x1c1b38 case 6 (op 4, event 6): `Effect_Create(1.0, obj+0xdc, obj,
//!   0xc0)` at command time [:406-409]. `Effect_Create` 0x1ce688 returns at once on a 0 contact
//!   material; in single player (`MPSettings+0x180 == 0`) flag 0x80 with 1/2/0x10 clear calls
//!   `Effect_Body` 0x1cf400, whose 0x40 arm runs `Sound_Play3D(100.0, -1.0, -1.0,
//!   EffectInfo[mat]+0x18, hitdata+0x40)` and `Sound_ModAlertness(0.5, h)`: the drones hear it at
//!   half the bank weight.
//! - The guards' death scripts carry the event at the frame the body lands (anim_fx_cmd_census).
//! - Sound end: a one-shot's node dies when the driver stops reporting it playing
//!   (`Sound_UpdateSounds` 0x1dda70); the driver drops an item when its voice's end flag comes up.
//!   With no device the port had no end flags, so headless nodes lived for the whole run and the
//!   256-node pool filled. `nf_view::attach_headless_sound` runs the driver with
//!   `nf_audio::VoiceClock` in the device's place.
//!
//! Real path: `nf_view::load` + `Sim::step_bond` inputs (the P2K kill through the trigger). Opt-in
//! via `NF_ISO`; silent (no output device is opened).

use nf_game::drone::state;
use nf_game::player::Input;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

fn dxz(a: [f32; 3], b: [f32; 3]) -> f32 { ((a[0] - b[0]).powi(2) + (a[2] - b[2]).powi(2)).sqrt() }

fn is_dead(s: u8) -> bool { matches!(s, state::DEAD | state::FADE | state::FADE_FAST | state::DEATH_ANIM) }

fn steer(sim: &mut nf_view::scene::Sim, gi: usize, fire: bool) {
    let (turn, look) = {
        let bp = sim.bond.pos;
        let g = sim.drones.list[gi].pos;
        let (dx, dz) = (g[0] - bp[0], g[2] - bp[2]);
        let dy = (g[1] + 0.9) - (bp[1] + sim.bond.eye - sim.bond.crouch_drop);
        let err = dx.atan2(dz) - sim.bond.yaw;
        let want_pitch = (dy / dxz(bp, g).max(0.5)).atan() / std::f32::consts::FRAC_PI_2;
        (err.clamp(-0.08, 0.08), (want_pitch - sim.bond.pitch).clamp(-0.08, 0.08))
    };
    sim.step_bond(Input { turn, look, ..Default::default() }, fire, false, false);
    if sim.drones.health.health < 1000.0 { sim.drones.health.health = 1000.0; }
}

/// The first live guard within 20 u of Bond's start on 07000005.
fn start_guard(sim: &nf_view::scene::Sim) -> usize {
    *sim.drones.llist.iter().find(|&&i| {
        let d = &sim.drones.list[i];
        d.present && d.enabled && !d.corpse && d.health > 0.0 && dxz(d.pos, sim.bond.pos) < 20.0
    }).expect("a guard near the 07000005 start")
}

/// The guard's body sound: the body column at the material under his feet (the port's feet ray).
fn body_id(sim: &nf_view::scene::Sim, feet: [f32; 3]) -> Option<(u32, Option<u32>)> {
    let (_, _, m) = sim.world.segment_surface([feet[0], feet[1] + 0.5, feet[2]], [feet[0], feet[1] - 1.0, feet[2]], true)?;
    let s = sim.surfaces.as_ref()?;
    if m & 0x3f == 0 { return None; }
    Some((nf_formats::sfx::SurfaceSounds::get(&s.body, m)?, nf_formats::sfx::SurfaceSounds::get(&s.footstep, m)))
}

/// Shooting the 05 start guard: when his death script reaches its body event, the body sound of
/// the floor under him enters the hearing list at his feet, at half the bank's weight. Before the
/// port, op 4 event 6 was dropped and no body sound ever played.
#[test]
fn a_shot_guards_body_hitting_the_floor_plays_the_body_sound() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set - skipping"); return };
    let mut scene = nf_view::load(&iso, "07000005.bin", false).expect("load 07000005");
    nf_view::attach_headless_sound(&mut scene);
    nf_view::settle_start_draw(scene.sim.as_mut().unwrap());
    let gi = start_guard(scene.sim.as_ref().unwrap());
    let g = scene.sim.as_ref().unwrap().drones.list[gi].pos;
    nf_view::dev::Dev { scenario: None, ..Default::default() }.teleport(&mut scene, [g[0] - 3.0, g[1], g[2]], 0.0);
    let sim = scene.sim.as_mut().unwrap();
    for _ in 0..60 {
        sim.step_bond(Input { switch_to: Some(2), ..Default::default() }, false, false, false);
        if sim.gun.weapon == 2 { break; }
    }
    assert_eq!(sim.gun.weapon, 2, "the P2K is drawn");
    // The game's aim assist is on by default (PlayerSetting+1 = 1 [L]); the scene's default is the mouse's off.
    sim.auto_aim = true;
    for _ in 0..30 { steer(sim, gi, false); }
    let mut died = None;
    for t in 0..900u32 {
        steer(sim, gi, t % 9 == 0);
        if is_dead(sim.drones.list[gi].state) { died = Some(t); break; }
    }
    died.unwrap_or_else(|| panic!("the guard dies under the P2K: hp {} state {:#x}", sim.drones.list[gi].health, sim.drones.list[gi].state));
    let (body, foot) = body_id(sim, sim.drones.list[gi].feet()).expect("a body sound on the floor under the guard");
    assert_ne!(Some(body), foot, "the body column differs from the footstep column on this floor (the count is the body's alone)");
    let weight = sim.sfx.as_ref().map(|t| t.get(body).weight).unwrap_or(0.0);
    let mut heard = Vec::new();
    for t in 0..600u32 {
        steer(sim, gi, false);
        let feet = sim.drones.list[gi].feet();
        for s in &sim.drones.sounds {
            if s.id == body && dxz(s.pos, feet) < 1.0 && (s.pos[1] - feet[1]).abs() < 1.0 { heard.push((t, s.weight)); }
        }
    }
    eprintln!("[body] id {body:#x} heard {heard:?}");
    assert!(!heard.is_empty(), "the death script's event 6 plays the body sound at the guard's feet");
    assert!(heard.iter().all(|&(_, w)| (w - weight * 0.5).abs() < 1e-4), "at half the bank weight (Sound_ModAlertness 0.5)");
}

/// With the headless driver attached, a fired shot's node is freed once its sample has played,
/// as the device's end flag frees it in play. Before the clock, no node of a deviceless run ever
/// ended (the driver never saw an end flag), so the shot's node stayed for the whole run.
#[test]
fn a_headless_runs_shot_sound_frees_its_node_after_its_length() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set - skipping"); return };
    let mut scene = nf_view::load(&iso, "07000005.bin", false).expect("load 07000005");
    nf_view::attach_headless_sound(&mut scene);
    nf_view::settle_start_draw(scene.sim.as_mut().unwrap());
    let sim = scene.sim.as_mut().unwrap();
    for _ in 0..60 {
        sim.step_bond(Input { switch_to: Some(2), ..Default::default() }, false, false, false);
        if sim.gun.weapon == 2 { break; }
    }
    for _ in 0..60 { sim.step_bond(Input::default(), false, false, false); }
    let before = sim.drones.audio.live_ids();
    sim.step_bond(Input::default(), true, false, false);
    let after = sim.drones.audio.live_ids();
    let fresh: Vec<u32> = after.iter().copied().filter(|id| after.iter().filter(|x| *x == id).count() > before.iter().filter(|x| *x == id).count()).collect();
    assert!(!fresh.is_empty(), "the shot queued a sound node: {before:x?} -> {after:x?}");
    let mut gone = None;
    for t in 0..600u32 {
        sim.step_bond(Input::default(), false, false, false);
        let now = sim.drones.audio.live_ids();
        if fresh.iter().all(|id| now.iter().filter(|x| *x == id).count() <= before.iter().filter(|x| *x == id).count()) { gone = Some(t); break; }
    }
    eprintln!("[pool] shot ids {fresh:x?} freed after {gone:?} ticks");
    let gone = gone.expect("the shot's node is freed once its sound has played");
    assert!(gone > nf_game::sound::START_GRACE, "not before the driver's start grace");
}
