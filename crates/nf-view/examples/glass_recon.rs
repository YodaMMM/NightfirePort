//! glass-pass-every-level recon: per level, each placed kind-2 pane's bounds/health + every
//! live drone within 25 u of it (the "guard behind glass" candidates). Prints only. [D]
use std::env;

fn main() {
    unsafe { env::set_var("NF_SILENT", "1"); }
    let iso = env::var("NF_ISO").expect("NF_ISO");
    let levels = ["07000006","07000008","07000014"];
    for lv in levels {
        let bin = format!("{lv}.bin");
        let mut scene = match nf_view::load(&iso, &bin, false) { Ok(s) => s, Err(e) => { println!("{lv}: LOAD FAILED {e}"); continue } };
        {
            let sim = scene.sim.as_ref().unwrap();
            let mut b = scene.sim.as_mut().unwrap();
            b.step_bond(nf_game::player::Input::default(), false, false, false);
            b.step_bond(nf_game::player::Input::default(), false, false, false);
        }
        let sim = scene.sim.as_ref().unwrap();
        let bp = sim.bond.pos;
        println!("== {lv} bond at {:.1},{:.1},{:.1} crouch {}", bp[0], bp[1], bp[2], sim.bond.crouched);
        for b in sim.drones.breaks.list.iter().filter(|b| b.kind == 2) {
            let c = [(b.lo[0]+b.hi[0])/2.0, (b.lo[1]+b.hi[1])/2.0, (b.lo[2]+b.hi[2])/2.0];
            print!("  pane {} hp {:.2} c {:.1},{:.1},{:.1} dx {:.1} dy {:.1} dz {:.1}", b.placement, b.health, c[0], c[1], c[2],
                b.hi[0]-b.lo[0], b.hi[1]-b.lo[1], b.hi[2]-b.lo[2]);
            let mut near = vec![];
            for (i, d) in sim.drones.list.iter().enumerate() {
                if d.health <= 0.0 { continue }
                let dd = ((d.pos[0]-c[0]).powi(2)+(d.pos[1]-c[1]).powi(2)+(d.pos[2]-c[2]).powi(2)).sqrt();
                if dd < 25.0 { near.push(format!("#{}({:.0}u at {:.1},{:.1},{:.1})", i, dd, d.pos[0], d.pos[1], d.pos[2])); }
            }
            if !near.is_empty() { print!(" guards: {}", near.join(" ")); }
            println!();
        }
    }
}
