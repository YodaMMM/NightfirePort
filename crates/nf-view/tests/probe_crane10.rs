//! probe10: the rotate crane's rotation — full matrix rows + SP state + a rider on the deck.
//! Print-only.

#[path = "common/walker_b2.rs"]
mod walker;

use walker::{Act, Walker, Wp};
use nf_game::player::Input;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn load_real() -> Walker {
    let cold = nf_view::load_with_carry(&iso(), "0700000d.bin", false, None).expect("load cold");
    let save = {
        let sim = cold.sim.as_ref().unwrap();
        nf_game::player::RamSave::capture(&sim.drones.health, sim.gun.switch_request, &sim.drones.inventory, &sim.drones.channels)
    };
    let mut w = Walker::new(nf_view::load_with_carry(&iso(), "0700000d.bin", false, Some(&save)).expect("load warm"));
    for c in [0x1bu8, 0x1c] { w.scene.sim.as_mut().unwrap().drones.set_channel(c); }
    w.wait_enabled(3000, "the intro");
    w.wait(120);
    w
}

fn pos(w: &Walker) -> [f32; 3] { w.scene.sim.as_ref().unwrap().bond.pos }

#[test]
#[ignore = "probe10: the rotate matrix and the rider; run with --ignored --nocapture"]
fn probe10_rotate_rider() {
    let mut w = load_real();
    // Straight to the crane deck (the proven landing spot) via a teleport — the crossing
    // itself is proven in probe8; this probe isolates the rotation.
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [199.25, 14.4, -260.16];
        sim.bond.vel = [0.0; 3];
    }
    w.wait(20);
    eprintln!("[probe10] on the deck {:?}", pos(&w));
    let used = w.try_walk_to(Wp::at(199.6, 13.4, -263.9, Act::Use { use_ticks: 240 }), "E #450");
    w.wait(5);
    eprintln!("[probe10] E #450 ok {used} ch0xa {}", w.scene.sim.as_ref().unwrap().drones.channels.on(0xa));
    for k in 0..26 {
        w.wait(30);
        let sim = w.scene.sim.as_ref().unwrap();
        let m = sim.drones.scripts.entity_matrix((0x0600_0a08, 1));
        let sp = sim.drones.scripts.players.iter().find(|p| p.placement == 492);
        let (r00, r02, r20, r22) = m.map(|m| (m[0][0], m[0][2], m[2][0], m[2][2])).unwrap_or((0.0, 0.0, 0.0, 0.0));
        eprintln!("[probe10] t{} bond {:?} rot [[{r00:.3},{r02:.3}],[{r20:.3},{r22:.3}]] sp492 {:?}",
            (k + 1) * 30, pos(&w), sp.map(|p| p.state));
    }
}
