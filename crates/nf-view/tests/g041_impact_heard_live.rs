//! g041-effects: a round's surface impact enters the drones' hearing list.
//!
//! Evidence [M]: `Effect_Bullet` 0x1ce8a8:86-90 plays the `EffectInfo[mat]+0x14` impact sound with
//! `Sound_Play3D(100.0, -1.0, -1.0, id, hit, 0, 0, 0)`; the `param_8 == 0` arm takes the sound's
//! inner/outer radii and its weight from `SFXOutputData[id]` [M Sound_Play3D 0x1dd548:27-52,
//! :96-98] and nothing lowers the weight after (`Effect_Body` 0x1cf400 halves its own with
//! `Sound_ModAlertness`; `Effect_Bullet` does not). `Sound_Alertness` 0x1ddc48 sums every
//! non-looped node of that list for each drone. [D] the impact rows weigh 20..50 with outer
//! radii 15..30 u and none is looped. The port played the impact on the audio pool only, so
//! the drones never heard where a round landed.
//!
//! Real path: `nf_view::load` 07000005 + `Sim::step_bond` with the P2K (the wall behind the
//! fix-wallhug-prompt spot — the probe-proven hit). Opt-in via `NF_ISO`; silent.

use nf_game::player::Input;
use nf_view::scene::Sim;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

fn aim_inputs(sim: &Sim, target: [f32; 3]) -> (f32, f32) {
    let head = sim.bond.head();
    let d = [target[0] - head[0], target[1] - head[1], target[2] - head[2]];
    let l = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt().max(1e-4);
    let want_yaw = d[0].atan2(d[2]);
    let want_pitch = (d[1] / l).clamp(-1.0, 1.0).asin() / std::f32::consts::FRAC_PI_2;
    let dy = want_yaw - sim.bond.yaw;
    let dy = dy - std::f32::consts::TAU * (dy / std::f32::consts::TAU).round();
    let dp = want_pitch - sim.bond.pitch;
    (dy.clamp(-0.05, 0.05), dp.clamp(-0.05, 0.05))
}

#[test]
fn a_rounds_wall_impact_is_heard_at_the_banks_weight() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set - skipping"); return };
    let mut scene = nf_view::load(&iso, "07000005.bin", false).expect("load 07000005");
    let mut sim = scene.sim.take().expect("sim");
    let idle = Input::default();
    for _ in 0..12000 { if !sim.disabled { break; } sim.step_bond(idle, false, false, false); }
    for _ in 0..60 { sim.step_bond(idle, false, false, false); }
    assert!(!sim.disabled, "the intro NIS never released Bond");
    let spot = [-18.28f32, 38.19, -71.83];
    let target = [-28.28f32, 39.70, -71.83];
    sim.bond.pos = spot;
    let wd = sim.weapons.clone();
    let rounds = wd.clip_size(0x02).max(1);
    let _ = sim.drones.inventory.equip_weapon(&wd, 0x02, rounds, true);
    sim.gun.switch_request = 0x02;
    for _ in 0..120 {
        sim.step_bond(idle, false, false, false);
        if sim.gun.weapon == 0x02 { break; }
    }
    assert_eq!(sim.gun.weapon, 0x02, "the P2K must draw");
    sim.bond.pos = spot;
    for _ in 0..100 {
        let (turn, look) = aim_inputs(&sim, target);
        sim.step_bond(Input { turn, look, ..Default::default() }, false, false, false);
    }
    let surf = sim.surfaces.clone().expect("05 carries the surface table");
    let table = sim.sfx.clone().expect("the SFXOutputData table");
    let impact_ids: Vec<u32> = surf.impact.iter().copied().filter(|&v| v != 0xffff).map(u32::from).collect();
    let mut heard = None;
    let mut played = false;
    for _ in 0..120 {
        sim.step_bond(Input::default(), true, false, false);
        if sim.drones.audio.active_ids().iter().any(|id| impact_ids.contains(id)) { played = true; }
        if let Some(s) = sim.drones.sounds.iter().find(|s| impact_ids.contains(&(s.id & 0xfffff))) { heard = Some(s.clone()); break; }
    }
    assert!(played || heard.is_some(), "the control: the P2K's wall hit plays its impact sound");
    let s = heard.expect("the impact sound must enter the drones' hearing list (Sound_Play3D's node list)");
    let h = table.get(s.id);
    assert_eq!((s.inner, s.outer, s.weight), (h.inner, h.outer, h.weight), "the bank's own radii and FULL weight");
    assert!(s.weight > 0.0);
    let dx = [s.pos[0] - spot[0], s.pos[2] - spot[2]];
    assert!((dx[0] * dx[0] + dx[1] * dx[1]).sqrt() > 3.0, "heard at the wall hit, not at Bond ({:?})", s.pos);
}
