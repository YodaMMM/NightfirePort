//! The disc census of the anim-script effect commands 6 and 7 (`AnimProcessScriptCmds`
//! 0x1c1b38's case 6 = `Effect_Create(1.0, obj+0xdc, obj, 0xc0)` — the body-impact 3D sound
//! `EffectInfo[mat]+0x18` + alertness 0.5 via `Effect_Body` 0x1cf400; case 7 = the NPC muzzle
//! event: `Bullet_init` from the weapon datum's muzzle bone + `AnimDatumSetEntity` +
//! `Light_Create(weapon+0x58, 2.0, bone, rgb weapon+0x56/55/54, 0, kind 3, 1, 0)` +
//! `Gas_Init(0.05, 0.1, cel, bone, (0, Rand_FRand(2π), 0), 0, 0x2000039, 7, 0x48403080, end 1)`
//! unless `weapon+0x68 & 0x2000`). fx-scrape-and-impacts row 8.
//!
//! Opt-in via NF_ISO; walks every FILES.BIN level archive's kind-5 anim scripts [D].

use std::collections::{BTreeMap, HashMap};

fn scripts_of<R: std::io::Read + std::io::Seek>(iso: &nf_formats::iso9660::IsoImage<R>, name: &str, read: &dyn Fn(&str) -> Option<Vec<u8>>) -> HashMap<u32, nf_formats::anim::Script> {
    let Some(data) = read(name) else { return HashMap::new() };
    let Ok(lb) = nf_formats::levelbin::LevelBin::parse(&data) else { return HashMap::new() };
    lb.entries.iter().filter(|e| e.kind == 5)
        .filter_map(|e| nf_formats::anim::Script::parse(lb.data(&data, e)).ok()).map(|x| (x.hashcode, x)).collect()
}

#[test]
fn anim_effect_cmd_census() {
    let Some(iso_path) = std::env::var("NF_ISO").ok() else { eprintln!("NF_ISO not set - skipping"); return };
    let iso = nf_formats::iso9660::IsoImage::open(&iso_path).expect("open ISO");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&iso).expect("FILES.BIN");
    let read = |name: &str| -> Option<Vec<u8>> {
        let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case(name))?;
        iso.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).ok()
    };
    // The weapon archive rides along every level load (the port's lib.rs:1008 pulls its kind-5
    // scripts too) — the fire anims live there, so the census reads it once.
    let mut weapon_scripts = scripts_of(&iso, "0700002c.bin", &read);
    let mut per_level: BTreeMap<String, (u32, u32)> = BTreeMap::new();
    let mut total6 = 0u32;
    let mut total7 = 0u32;
    for code in 0x0700_0001u32..=0x0700_004a {
        let name = format!("{code:08x}.bin");
        if read(&name).is_none() { continue; }
        let _ = &name;
        let mut scripts = scripts_of(&iso, &name, &read);
        // The level's skin packs (lib.rs:1002's `078/079/07A{nn}.bin`) carry the guards' anims —
        // the fire anims' op-7s among them.
        let nn = &name[3..8];
        for pfx in ["078", "079", "07A"] {
            for (h, s) in scripts_of(&iso, &format!("{pfx}{nn}.bin"), &read) { scripts.entry(h).or_insert(s); }
        }
        for (h, s) in weapon_scripts.iter() { scripts.entry(*h).or_insert_with(|| s.clone()); }
        if code == 0x0700_0005 {
            let mut ops: BTreeMap<u8, u32> = BTreeMap::new();
            for s in scripts.values() { for c in &s.commands { *ops.entry(c.op).or_insert(0) += 1; } }
            println!("  07000005 op histogram: {:?} (scripts {})", ops, scripts.len());
        }
        let (mut c6, mut c7) = (0u32, 0u32);
        for s in scripts.values() {
            for c in &s.commands {
                match c.op { 6 => c6 += 1, 7 => c7 += 1, _ => {} }
            }
        }
        if c6 + c7 > 0 { per_level.insert(format!("{code:08x}"), (c6, c7)); }
        total6 += c6;
        total7 += c7;
    }
    let w6 = weapon_scripts.values().map(|s| s.commands.iter().filter(|c| c.op == 6).count() as u32).sum::<u32>();
    let w7 = weapon_scripts.values().map(|s| s.commands.iter().filter(|c| c.op == 7).count() as u32).sum::<u32>();
    println!("weapon archive 0700002c: cmd6 {w6} cmd7 {w7}");
    println!("merged per level: cmd6 total {total6}, cmd7 total {total7}");
    for (lv, (c6, c7)) in &per_level { println!("  {lv}: cmd6 {c6} cmd7 {c7}"); }
    // The verdict the audit-effect-calls row 8 needed: the interpreter's cases 6/7 exist in the
    // code [M 0x1c1b38] but NO anim script on the disc carries op 6 or op 7 — every swept script
    // holds only ops 0/1/4 (the 07000005 histogram above is representative). Row 8 is UNREACHED
    // on the disc — nothing to port (the same verdict as the M_TEXTBOX census).
    assert_eq!((total6, total7), (0, 0), "anim ops 6/7 appeared — the row-8 verdict changes");
    let _ = &mut weapon_scripts;
}
