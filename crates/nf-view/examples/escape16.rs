//! Probe [L-shaped, in-port]: Bond re-enabled at the pit floor (the cinema hand-back pose),
//! then REAL INPUT only — try to escape the hurt stack before the pulse (the port models ch 243
//! at launch+1611): attempt A = run NE to the east blast-room door (17.33,-21.63,-169.38, opens
//! at 1350 [D keys]); attempt B = run S onto the Booster2 lower surface (bbox y -20.59 [D]).
//! Logs pos/hp/grounded/ch243 every 50 ticks. Opt-in via NF_ISO.
use nf_game::player::Input;

fn carry(on: &[u32]) -> nf_game::player::RamSave {
    let mut ch = nf_game::triggers::Channels::default();
    for &c in on { ch.set(c, 1, 0); }
    nf_game::player::RamSave::capture(&Default::default(), 0, &nf_game::pickup::Inventory::level_start(0x73, 34), &ch)
}

fn attempt(tag: &str, yaw_to: [f32; 3], jump_at: bool) {
    let iso = std::env::var("NF_ISO").unwrap();
    let mut scene = nf_view::load_with_carry(&iso, "07000016.bin", false, Some(&carry(&[0x47, 0x48, 0x4a, 0x54, 0xce, 0x4b]))).expect("load");
    {
        let sim = scene.sim.as_mut().unwrap();
        for _ in 0..600 { sim.step_bond(Input::default(), false, false, false); }
        sim.bond.pos = [2.4, 1.2, -175.5]; // the bridge slab: the chain fires (the probe-only setup)
    }
    let mut reenabled = false;
    let mut pulse = None;
    let mut dead = None;
    for t in 0..3400u32 {
        // drive: before the re-enable, idle; after, run toward the target with the look set.
        let mut inp = Input::default();
        let (p, yaw, pitch) = { let s = scene.sim.as_ref().unwrap(); (s.bond.pos, s.bond.yaw, s.bond.pitch) };
        if p[1] < -20.0 { reenabled = true; }
        if reenabled && pulse.is_none() && scene.sim.as_ref().unwrap().drones.channels.on(243) { pulse = Some(t); }
        if reenabled {
            // steer: yaw input toward yaw_to (the port's Input::turn is a delta lane)
            let dx = yaw_to[0] - p[0]; let dz = yaw_to[2] - p[2];
            let want = dz.atan2(dx); // world yaw for +x east
            let mut d = want - yaw;
            while d > std::f32::consts::PI { d -= 2.0 * std::f32::consts::PI; }
            while d < -std::f32::consts::PI { d += 2.0 * std::f32::consts::PI; }
            inp.turn = d.clamp(-0.05, 0.05);
            inp.forward = 1.0;
            if jump_at && t % 40 == 0 { inp.jump = true; }
        }
        scene.sim.as_mut().unwrap().step_bond(inp, false, false, false);
        let s = scene.sim.as_ref().unwrap();
        if t % 100 == 0 || s.drones.health.health <= 0.0 {
            eprintln!("{tag} t{t}: pos({:.2},{:.2},{:.2}) hp {:.1} g {} 243 {} re{}",
                s.bond.pos[0], s.bond.pos[1], s.bond.pos[2], s.drones.health.health, s.bond.grounded,
                s.drones.channels.on(243) as u8, reenabled as u8);
        }
        if s.drones.health.health <= 0.0 { dead = Some(t); break; }
    }
    eprintln!("{tag} DONE re@pulse {pulse:?} dead {dead:?}");
}

fn main() {
    attempt("A-east-blast-room", [17.33, -21.63, -169.38], false);
    attempt("B-booster-south", [2.38, -21.0, -187.0], true);
}
