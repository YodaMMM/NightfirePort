//! `nf_view::menu::CORNER_STAMP_PAGES` is exactly the set of front-end pages whose control list carries
//! the corner stamp 0x10000224 (TEX 0x0300014c) shown (msg 0x2b 6) [D 07000048.bin 08000002]. Join Game
//! and the MP Start summary carry none (PCSX2 shows no corner there). Opt-in via NF_ISO.

use nf_formats::menudef::MenuDef;

fn menu_def(iso_path: &str) -> MenuDef {
    let iso = nf_formats::iso9660::IsoImage::open(iso_path).expect("open ISO");
    let (files, bin) = nf_formats::disc::files_bin_entries(&iso).expect("FILES.BIN");
    let fe = files.iter().find(|f| f.name.eq_ignore_ascii_case("07000048.BIN")).expect("07000048.bin");
    let data = iso.read_bytes(bin.lba as u64 * nf_formats::iso9660::SECTOR_SIZE as u64 + fe.offset as u64, fe.size as u64).expect("read");
    let lb = nf_formats::levelbin::LevelBin::parse(&data).expect("parse 07000048.bin");
    let e = lb.entries.iter().find(|e| e.hash() == 0x0800_0002).expect("menu def 08000002");
    MenuDef::parse(lb.data(&data, e), 0x8000_0002).expect("parse the menu def")
}

#[test]
fn the_corner_stamp_pages_are_the_discs() {
    let Ok(iso) = std::env::var("NF_ISO") else { return };
    let md = menu_def(&iso);
    let mut disc: Vec<u32> = md.pages.iter()
        .filter(|p| p.controls.iter().any(|c| c.id == 0x1000_0224 && c.texture == Some(0x0300_014c)
            && c.msgs.iter().rev().find(|m| m.0 == 0x2b).is_some_and(|m| m.1 == 6)))
        .map(|p| p.id).collect();
    disc.sort();
    let mut ours = nf_view::menu::CORNER_STAMP_PAGES.to_vec();
    ours.sort();
    assert_eq!(ours, disc);
    // Sister: the pages PCSX2 shows without the corner are not in the set.
    for no in [nf_view::menu::PAGE_MP_JOIN, nf_view::menu::PAGE_MP_CONFIRM] {
        assert!(!disc.contains(&no), "{no:#x} carries no corner stamp [D]");
    }
}

/// Every front-end page the port draws takes the backdrop movie its PAGE record names [D aux]: the Dossier's
/// Records / Rewards / Gadgets / Weapons drew the dark plate (aux 0 in the port, 0x07350048 on the disc).
#[test]
fn the_dossier_pages_take_their_records_backdrop() {
    let Ok(iso) = std::env::var("NF_ISO") else { return };
    let md = menu_def(&iso);
    for id in [nf_view::menu::PAGE_RECORDS, nf_view::menu::PAGE_REWARDS, nf_view::menu::PAGE_GADGETS, nf_view::menu::PAGE_WEAPONS] {
        let aux = md.page(id).expect("the page record").aux;
        assert_eq!(nf_view::menu::aux_movie_for(id), aux, "{id:#x}'s backdrop");
    }
}
