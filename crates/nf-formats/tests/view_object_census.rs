//! Disc census [D]: the placements behind the three HUD views the health ring hides for
//! (`HUD_UpdateHealthPane` 0x19b9c8:127-138 — panes 10, 16, 18) and the Q-Worm PCs:
//!
//! - pane 10 = cam mode 0xd, set only by `Car_Activate` 0x131c48 (placement class 0xda →
//!   `Car_Create` 0x1305b0 [M parsemap 0x1d04f0 case 0xda]);
//! - pane 16 = the monitor view, `Monitor_Activate` 0x1b0058:47 (placement class 0x30);
//! - pane 18 = cam mode 0xf, set only by `GT_TakeControl` 0x181740 (a gun turret whose owner is
//!   the player, i.e. a turret that player deployed with weapon 0x53 [M 0x1a4bf0:120-133]);
//!   placed turrets are class 0x27 → `GT_Create` 0x17ffa0 with owner 0;
//! - class 0x35 → `PCQWorm_Create` 0x1b07a8 (param 0 = the KeyCodes index the install reveals).
//!
//! Prints one row per placement. Informational: asserts nothing about the data.
//!
//! Run: `NF_ISO=… cargo test -p nf-formats --test view_object_census -- --nocapture`

use nf_formats::{disc, iso9660::IsoImage, level::Level};

#[test]
fn census_view_objects() {
    let Ok(path) = std::env::var("NF_ISO") else { return };
    let im = IsoImage::open(&path).unwrap();
    let (entries, bin) = disc::files_bin_entries(&im).unwrap();
    for fe in entries.iter().filter(|e| e.name.to_ascii_lowercase().ends_with(".bin")) {
        let data = im.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).unwrap();
        let Ok(lv) = Level::parse(&data) else { continue };
        for (mi, m) in lv.maps.iter().enumerate() {
            for (pi, p) in m.placements.iter().enumerate() {
                let class = p.flags & 0xff;
                if ![0x35, 0x30, 0x27, 0xda].contains(&class) { continue; }
                let name = if p.by_hash() {
                    lv.by_hash.get(&p.hashcode).map(|&(a, b)| lv.maps[a].objects[b].name.clone())
                } else {
                    m.objects.get(p.index as usize).map(|o| o.name.clone())
                }.unwrap_or_default();
                println!("{}\tmap{}\t#{}\tclass {:#x}\t{}\t{:?}\t{:?}", fe.name, mi, pi, class, name,
                    p.translation, p.params);
            }
        }
    }
}
