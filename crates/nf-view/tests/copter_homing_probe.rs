//! port-map-bullets probe 2: per level — real copter placements (0x2F with param 0 == 0, the
//! only kind `Copter_Create` 0x17d340:43-45 builds a copter object for) and any pickup (0xf0)
//! granting a homing weapon (0x2e/0x30/0x6d; 0x45 is the thrown hat). The reachability proof
//! for `Bullet_heatseekable` 0x12f248's copter arm: it can SELECT a copter only when a homing
//! round flies whose owner is not the copter. Opt-in; prints only.

#[test]
#[ignore = "assertion-free probe; run with --ignored"]
fn sweep_copters_vs_homing_pickups() {
    let repo = std::env::var("NF_REPO").unwrap_or_else(|_| "/home/z/NightfirePort".into());
    let iso = std::env::var("NF_ISO").expect("NF_ISO");
    let img = nf_formats::iso9660::IsoImage::open(&iso).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let mut archives: Vec<String> = entries.iter().map(|e| e.name.to_ascii_uppercase())
        .filter(|n| n.len() == 12 && n.starts_with("0700") && n.ends_with(".BIN")).collect();
    archives.sort(); archives.dedup();
    for a in &archives {
        let data = match std::panic::catch_unwind(|| {
            let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case(&a.to_ascii_lowercase())).unwrap();
            img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read")
        }) { Ok(d) => d, Err(_) => continue };
        let lv = match nf_formats::level::Level::parse(&data) { Ok(l) => l, Err(_) => continue };
        let mm = match lv.main_map() { Some(m) => m, None => continue };
        let param = |p: &nf_formats::placement::Placement, k: u32|
            p.params.iter().find(|(j, _)| *j == k).map_or(0, |&(_, v)| v);
        let copters: Vec<usize> = lv.maps[mm].placements.iter().enumerate()
            .filter(|(_, p)| p.flags & 0xff == 0x2f && param(p, 0) == 0).map(|(i, _)| i).collect();
        // 0xf0 pickups: the weapon id is param 0 (the pickup's id); grant-weapons also come as
        // 0xf1/0xf2 sets — print any 0xf* placement whose params mention a homing weapon id.
        let homing = [0x2eu32, 0x30, 0x6d];
        let mut pickups = Vec::new();
        for (i, p) in lv.maps[mm].placements.iter().enumerate() {
            let k = p.flags & 0xff;
            if !(0xf0..=0xf3).contains(&k) { continue; }
            for slot in 0..8u32 {
                let v = param(p, slot);
                if homing.contains(&v) || v == 0x45 { pickups.push((i, k, slot, v)); }
            }
        }
        if !copters.is_empty() || !pickups.is_empty() {
            println!("{a}: copters(0x2F kind0) {copters:02x?}  homing-weapon pickup params {pickups:02x?}");
        }
    }
}
