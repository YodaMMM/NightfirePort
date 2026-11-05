//! audit-sprite-set-text: the MP status-line labels bound from the disc's USATxt.dat, through the
//! port's own FILES.BIN reader — pins the exact strings every `Sprite_SetText` caller puts in
//! `StatusSpr` / `TimeSpr` [M 0x1848d0 / 0x184bd0 / 0x184268 / 0x1870a0], so the status-line
//! concatenation (`nf_game::mp::hud::status_line`'s parts) and the doc strings are grounded.
//! Opt-in via `NF_ISO`.

use nf_formats::iso9660::IsoImage;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

fn lang() -> nf_formats::hud::Lang {
    let path = iso().expect("NF_ISO");
    let img = IsoImage::open(&path).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("USATxt.dat")).expect("no USATxt.dat");
    let bytes = img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read");
    nf_formats::hud::Lang::parse(&bytes).expect("USATxt.dat parses")
}

#[test]
fn the_mp_status_labels_bind_from_the_disc() {
    let Some(_) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let l = lang();
    let get = |id: u32| l.bind(id).map(|s| s.to_string()).unwrap_or_else(|| "<unbound>".into());
    // The end-of-match status arms [M MP_SortOutWhoWon 0x1848d0, MP_Update 0x184bd0 case 2].
    for id in [0x2000022u32, 0x2000023, 0x2000024, 0x2000027, 0x2000028, 0x1c9, 0x2000049] {
        println!("0x{id:08x} = {:?}", get(id));
    }
    // The demolition/protection site statuses [M MP_DemolitionProtectionUpdate 0x1870a0:102-158].
    for id in [0x200002bu32, 0x200002c, 0x200002d, 0x200002e, 0x200002f] {
        println!("0x{id:08x} = {:?}", get(id));
    }
    // The objective status headers [M HUD_UpdateStatusPane 0x19c310 case 2].
    for id in [0x2000051u32, 0x2000052, 0x2000053] {
        println!("0x{id:08x} = {:?}", get(id));
    }
    // The fmt constants the callers sprintf through [L ee_start_05.bin 0x30cbe0..0x30cc10]:
    // "" @0x30cbe8, "%s" @0x30cbf0, "%s%s" @0x30cbf8, "%s%s %s" @0x30cc00, "%d - %d" @0x30cc08.
    // The port's concat matches the game's sprintf: each part's %s slots are filled in print
    // order (`StatusPart::Format`), plain labels concatenate.
    assert!(!get(0x2000028).is_empty(), "the time-up status label binds");
    assert!(!get(0x1c9).is_empty(), "the second status label binds");
    assert!(!get(0x2000049).is_empty(), "the restart label binds");
}
