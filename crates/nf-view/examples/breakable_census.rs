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
                // The MATERIAL census (the glass-pierce task): every kind-2 pane's shape
                // materials — the flight's glass-pass reads the STATIC tri's surface byte
                // (& 0x3f == 0x10) [M 0x12cf54-0x12cf60], so which panes are material 0x10
                // decides which sheets the rounds pass.
                let mut mats: std::collections::BTreeMap<u8, usize> = Default::default();
                let mut mat_panes: Vec<String> = Vec::new();
                for b in sim.drones.breaks.list.iter().filter(|b| b.kind == 2) {
                    let ms: &[u8] = sim.world.shape_materials(b.shape_key).unwrap_or(&[]);
                    let mut hist: std::collections::BTreeMap<u8, usize> = Default::default();
                    for m in ms { let k = *m & 0x3f; *hist.entry(k).or_default() += 1; }
                    for (k, c) in &hist { *mats.entry(*k).or_default() += c; }
                    let has10 = hist.keys().any(|k| *k == 0x10);
                    mat_panes.push(format!("{}:{}{}", b.placement,
                        if b.shape_key == 0 { "no-shape" } else { "" },
                        if has10 { String::new() } else {
                            format!("[mats {}]", hist.iter().map(|(k, c)| format!("{k:#04x}x{c}")).collect::<Vec<_>>().join(","))
                        }));
                }
                let mat_summary = mats.iter().map(|(k, c)| format!("{k:#04x}x{c}")).collect::<Vec<_>>().join(",");
                eprintln!("{lv}: kind-2 pane tri materials [{mat_summary}]; non-0x10 panes: {}",
                    mat_panes.iter().filter(|s| !s.ends_with(":")).cloned().collect::<Vec<_>>().join(" "));
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
