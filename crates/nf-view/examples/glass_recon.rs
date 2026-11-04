//! glass-pass-every-level recon: per level, each placed kind-2 pane's bounds/health + every
//! live drone within 25 u of it (the "guard behind glass" candidates). Prints only. [D]
use std::env;

fn main() {
    unsafe { env::set_var("NF_SILENT", "1"); }
    let iso = env::var("NF_ISO").expect("NF_ISO");
    let levels = ["07000001","07000003","07000006","07000007","07000008","07000009",
                  "0700000a","07000012","07000013","07000014"];
    for lv in levels {
        let bin = format!("{lv}.bin");
        let scene = match nf_view::load(&iso, &bin, false) { Ok(s) => s, Err(e) => { println!("{lv}: LOAD FAILED {e}"); continue } };
        let sim = scene.sim.as_ref().unwrap();
        let player = Some(sim.drones.player_pos).map(|p| p).unwrap_or([0.0;3]);
        println!("== {lv} player at {:.1},{:.1},{:.1}", player[0], player[1], player[2]);
        for b in sim.drones.breaks.list.iter().filter(|b| b.kind == 2) {
            let c = [(b.lo[0]+b.hi[0])/2.0, (b.lo[1]+b.hi[1])/2.0, (b.lo[2]+b.hi[2])/2.0];
            print!("  pane {} hp {:.2} c {:.1},{:.1},{:.1} dx {:.1} dy {:.1} dz {:.1}", b.placement, b.health, c[0], c[1], c[2],
                b.hi[0]-b.lo[0], b.hi[1]-b.lo[1], b.hi[2]-b.lo[2]);
            let mut near = vec![];
            for (i, d) in sim.drones.list.iter().enumerate() {
                if d.health <= 0.0 { continue }
                let dd = ((d.pos[0]-c[0]).powi(2)+(d.pos[1]-c[1]).powi(2)+(d.pos[2]-c[2]).powi(2)).sqrt();
                if dd < 25.0 { near.push(format!("#{}({:.0}u)", i, dd)); }
            }
            if !near.is_empty() { print!(" guards: {}", near.join(" ")); }
            println!();
        }
    }
}
