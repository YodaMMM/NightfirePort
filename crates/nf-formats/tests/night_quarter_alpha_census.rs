//! night-alpha: which glists carry the obj+0xF0 bit 0x20000 that night sight quarters
//! [M View_SetupRenderModes_Tweak 0x1e5888:12-17: `if (obj+0xf0 & 0x20000) && viewer+0x236 == 1
//! then alpha = obj+0x106 >> 2`]. obj+0xF0 is loaded from the glist flags (+0x44) by
//! `Control_SetGList` 0x134960 / `Control_CreateObjEx` 0x1343c8 and the per-class creators; the
//! only code that ORs 0x20000 in is `Gas_Init` 0x1aacc8:83 (`|= 0x20020120`, every gas puff).
//! This census proves the data side [D]: no glist in any SP archive carries the bit, so the gas
//! puffs are the whole set. Opt-in via NF_ISO. Numbers and names only.

use nf_formats::iso9660::IsoImage;
use nf_formats::level::Level;

const SP_LEVELS: [&str; 21] = [
    "07000001", "07000002", "07000003", "07000004", "07000005", "07000006", "07000007",
    "07000008", "07000009", "0700000a", "0700000b", "0700000c", "07000011", "07000012",
    "07000013", "07000014", "07000015", "07000016", "07000041", "0700004a", "0700004b",
];

fn levels() -> Option<Vec<(&'static str, Level)>> {
    let iso = std::env::var("NF_ISO").ok()?;
    let img = IsoImage::open(&iso).ok()?;
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).ok()?;
    Some(SP_LEVELS.iter().filter_map(|&id| {
        let fe = entries.iter().find(|e| e.name == format!("{id}.bin"))?;
        let data = img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).ok()?;
        Some((id, Level::parse(&data).ok()?))
    }).collect())
}

/// Objects (every map of every SP archive, duplicates across maps counted) whose glist flags
/// carry `bit`.
fn count(levels: &[(&str, Level)], bit: u32) -> usize {
    levels.iter().flat_map(|(_, l)| l.maps.iter()).flat_map(|m| m.objects.iter()).filter(|o| o.flags & bit != 0).count()
}

#[test]
fn no_glist_carries_the_night_quarter_alpha_bit() {
    let Some(levels) = levels() else { eprintln!("NF_ISO not set - skipping"); return };
    assert_eq!(levels.len(), 21);
    assert_eq!(count(&levels, 0x2_0000), 0, "glists with flags 0x20000");
    // Sister (can fail): the neighbouring high bits ARE carried, so the flags word is read past
    // bit 16 and the zero above is the data, not a truncated parse.
    assert_eq!(count(&levels, 0x4_0000), 73, "glists with flags 0x40000");
    assert_eq!(count(&levels, 0x8_0000), 182, "glists with flags 0x80000");
    assert_eq!(count(&levels, 0x10_0000), 29, "glists with flags 0x100000");
    // The searchlight beams are not among them [D]: 07000005's two beam glists are flags 0x221.
    let l05 = &levels.iter().find(|(id, _)| *id == "07000005").unwrap().1;
    let beams: Vec<u32> = l05.maps.iter().flat_map(|m| m.objects.iter())
        .filter(|o| o.name.starts_with("OBJECT_SearchLightbeam")).map(|o| o.flags).collect();
    assert_eq!(beams, vec![0x221, 0x221], "07000005 searchlight beam glist flags");
}
