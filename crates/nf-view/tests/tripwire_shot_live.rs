//! tripwires-shot-from-range: a player round from range detonates a tripwire mine (the tripbomb,
//! `Mine_Create` 0x17ce28, class 0x4B). The game chain for a round at a tripbomb:
//! - the pick adds every object whose world sphere the flight segment enters [M `Collide_PickObj`
//!   0x1eae80 — the obj+0x80 centre + obj+0x8C radius against `vecutil_calculate_closest_point_on_line`
//!   of the HITTEST segment; the mine's class 0x4B is not in the mask skips (0x400→3/0x12, 0x100→2/0x11,
//!   0x200→5) and `Bullet_update` 0x12e7b0 arms +0x9A = 0x20];
//! - the intersect tests the mine cel's own tris with the one-sided ray [M `Collide_Intersect`
//!   0x1ebaa8 → `Intersect_RayGeom` 0x10a8c8:168];
//! - the survivor mirrors onto the mine's `obj+0xD0` with `+0x58` = the bullet obj, `+0x51` |= 1
//!   [M `Collide_Update` 0x1ea398:160-234], and the walk stops at the first solid node
//!   [M 0x1ea398:216-229] — a nearer wall or guard takes the round instead;
//! - `Mine_Update` 0x17c930:31-45 explodes on ANY `+0x58 != 0` node (no filter — the bullet's
//!   weapon class is irrelevant), `Camera_Shake` at radius ≥ 0.5, the Explode object from the
//!   mine's params (damage = p0, radius/shake/scale = p1, blast script = p5), the mine deletes
//!   (`obj+0xFE` |= 1) [M 0x17cb7c-0x17cb88].
//!
//! The port gap this pins: the mines' explode arm tested only the player capsule and the drone
//! bodies — a round at the beam from range never registered. The fix lands the bullet contact on
//! the mine (the mirrored node's port stand-in) and the same `Mine_Update` arm explodes it.
//!
//! Opt-in: set `NF_ISO` to the disc image.

use nf_game::player::{Bond, Input};

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

/// Dev census (#[ignore]; run with `-- --ignored --nocapture`): the tripbomb placements (kind
/// 0xFE [M `Mine_Create` 0x17ce28's placement scan]) on the tripwire levels, with their world
/// bounds, blast script and tri count — the probe that chose the pin's level and mine. Also
/// every map's placement-kind histogram, so a mine on a non-primary map can't hide.
#[test]
#[ignore]
fn census_tripbomb_placements() {
    use nf_formats::{disc, iso9660::IsoImage, level::Level};
    let Some(iso_path) = iso() else { eprintln!("skip: NF_ISO not set"); return };
    let iso = IsoImage::open(&iso_path).unwrap();
    let (entries, bin) = disc::files_bin_entries(&iso).unwrap();
    for lvl in ["0700000a.bin", "0700000b.bin", "07000014.bin"] {
        let Some(fe) = entries.iter().find(|e| e.name.eq_ignore_ascii_case(lvl)) else { continue };
        let data = iso.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).unwrap();
        let level = Level::parse(&data).unwrap();
        println!("{lvl}: {} maps", level.maps.len());
        for (mi, m) in level.maps.iter().enumerate() {
            let mut kinds = std::collections::BTreeMap::<u32, usize>::new();
            for p in &m.placements { *kinds.entry(p.flags & 0xff).or_default() += 1; }
            let mines: Vec<_> = m.placements.iter().filter(|p| p.flags & 0xff == 0xfe).collect();
            println!("  map[{mi}]: {} placements; kinds {:?}", m.placements.len(),
                kinds.iter().map(|(k, v)| format!("{k:#04x}x{v}")).collect::<Vec<_>>().join(","));
            for p in &mines {
                println!("    0xFE placement idx {} hash {:#x} trans {:?} params {:?}",
                    p.index, p.hashcode, p.translation, p.params);
            }
        }
    }
}
