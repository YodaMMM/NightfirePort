//! port-map-bullets probe: sweep the weapon_data records (ee_start_05.bin @ 0x2bf150, 115 ×
//! 0x10C [L]) for the `+0x6C` homing bit 0x20000 (`Bullet_homing`'s gate) and print every
//! carrier with its kind — the reachability proof for `Bullet_heatseekable` 0x12f248's
//! non-drone arms. Opt-in; prints only.

#[test]
#[ignore = "assertion-free probe; run with --ignored"]
fn sweep_homing_weapon_records() {
    let repo = std::env::var("NF_REPO").unwrap_or_else(|_| "/home/z/NightfirePort".into());
    let bytes = std::fs::read(format!("{repo}/work/ee_start_05.bin")).expect("ee_start_05.bin");
    let base = 0x2bf150usize;
    let n = 115usize;
    for w in 0..n {
        let at = base + w * 0x10c;
        if at + 0x70 > bytes.len() { break; }
        let rec = &bytes[at..at + 0x10c];
        let flags = u32::from_le_bytes(rec[0x6c..0x70].try_into().unwrap());
        let kind = u16::from_le_bytes(rec[0x02..0x04].try_into().unwrap());
        let range = f32::from_le_bytes(rec[0x1c..0x20].try_into().unwrap());
        if flags & 0x2000_0 != 0 {
            println!("weapon {w:#04x}: +0x6C = {flags:#010x}  kind {kind:#06x}  range {range}  <<<< HOMING");
        }
    }
    println!("swept {n} records at {base:#x}, file {} bytes", bytes.len());
}
