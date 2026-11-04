//! glass-pass-every-level census: every story level's PLACED breakables (Breaks.list at load),
//! by kind — the placed 0x20 panes [D]. Prints only. [D]
use std::env;

fn main() {
    unsafe { env::set_var("NF_SILENT", "1"); }
    let iso = env::var("NF_ISO").expect("NF_ISO");
    let levels = ["07000001","07000002","07000003","07000004","07000005","07000006","07000007",
                  "07000008","07000009","0700000a","0700000b","0700000c","0700000d","07000011",
                  "07000012","07000013","07000014","07000015","07000016","0700001b"];
    for lv in levels {
        let bin = format!("{lv}.bin");
        match nf_view::load(&iso, &bin, false) {
            Ok(scene) => {
                let sim = scene.sim.as_ref().unwrap();
                let mut by_kind: std::collections::BTreeMap<u8, Vec<usize>> = Default::default();
                for b in &sim.drones.breaks.list {
                    by_kind.entry(b.kind).or_default().push(b.placement);
                }
                let summary = by_kind.iter()
                    .map(|(k, v)| format!("kind {k}: {} ({})", v.len(),
                        if v.len() > 12 { format!("{}..", &v[..12].iter().map(|p| p.to_string()).collect::<Vec<_>>().join(",")) } else { v.iter().map(|p| p.to_string()).collect::<Vec<_>>().join(",") }))
                    .collect::<Vec<_>>().join("; ");
                println!("{lv}: breaks {} [{summary}]", sim.drones.breaks.list.len());
            }
            Err(e) => println!("{lv}: LOAD FAILED {e}"),
        }
    }
}
