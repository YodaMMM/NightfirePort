//! The MP match summary 0x40000049's tinted plates and title bars (`nf_view::fe_pages::SUMMARY_PLATES`)
//! are the page's own type-5 labels: same control id, layer, rect, colour (msg 0x1b), texture
//! 0x03000042 (msg 0x24) and texel (msg 0x2d through `msg2d_uv`) [D 07000048.bin 08000002].
//! Opt-in via NF_ISO.

use nf_formats::menudef::MenuDef;
use nf_view::fe_pages::{msg2d_uv, SUMMARY_PLATES};

fn menu_def(iso_path: &str) -> MenuDef {
    let iso = nf_formats::iso9660::IsoImage::open(iso_path).expect("open ISO");
    let (files, bin) = nf_formats::disc::files_bin_entries(&iso).expect("FILES.BIN");
    let fe = files.iter().find(|f| f.name.eq_ignore_ascii_case("07000048.BIN")).expect("07000048.bin");
    let data = iso.read_bytes(bin.lba as u64 * nf_formats::iso9660::SECTOR_SIZE as u64 + fe.offset as u64, fe.size as u64).expect("read");
    let lb = nf_formats::levelbin::LevelBin::parse(&data).expect("parse 07000048.bin");
    let e = lb.entries.iter().find(|e| e.hash() == 0x0800_0002).expect("menu def 08000002");
    MenuDef::parse(lb.data(&data, e), 0x8000_0002).expect("parse the menu def")
}

fn disc_plates(md: &MenuDef) -> Vec<(u32, u8, i32, i32, i32, i32, u16, u16, u32)> {
    let pg = md.page(0x4000_0049).expect("page 0x40000049");
    pg.controls.iter().filter(|c| c.ctype == 0 && c.texture == Some(0x0300_0042)).map(|c| {
        let col = c.msgs.iter().find(|m| m.0 == 0x1b).map(|m| m.1).unwrap();
        let uv = c.msgs.iter().find(|m| m.0 == 0x2d).map(|m| msg2d_uv(m.1, m.2)).unwrap();
        assert_eq!((uv[2], uv[3]), (1.0, 1.0), "one texel");
        (c.id, c.f1 as u8, c.x as i32, c.y as i32, c.w as i32, c.h as i32, uv[0] as u16, uv[1] as u16, col)
    }).collect()
}

#[test]
fn the_summary_plates_are_the_pages_own_labels() {
    let Ok(iso) = std::env::var("NF_ISO") else { return };
    let mut disc = disc_plates(&menu_def(&iso));
    let mut ours = SUMMARY_PLATES.to_vec();
    disc.sort(); ours.sort();
    assert_eq!(ours, disc);
}

/// Sister: the msg-0x2d words read low-half-first would name a different texel — the comparison can fail.
#[test]
fn a_swapped_uv_read_would_not_match() {
    let Ok(iso) = std::env::var("NF_ISO") else { return };
    let md = menu_def(&iso);
    let swapped: Vec<_> = disc_plates(&md).into_iter().map(|mut p| { std::mem::swap(&mut p.6, &mut p.7); p }).collect();
    let mut ours = SUMMARY_PLATES.to_vec();
    let mut sw = swapped; ours.sort(); sw.sort();
    assert_ne!(ours, sw);
}
