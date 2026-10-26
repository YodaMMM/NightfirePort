//! InverterTrigger (0xF9) on the real level: 07000014 #390 writes ch 1 from ch 0x87.
use nf_formats::level::Level;
use nf_game::triggers::{self, kind, Channels};

#[test]
fn the_07000014_inverter_clears_channel_1_from_channel_0x87() {
    let iso = std::env::var("NF_ISO").expect("NF_ISO");
    let image = nf_formats::iso9660::IsoImage::open(&iso).expect("iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&image).expect("entries");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("07000014.bin")).expect("14");
    let data = image.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read");
    let lvl = Level::parse(&data).expect("parse");
    let mut list = triggers::build(&lvl);
    let inv: Vec<usize> = list.iter().enumerate().filter(|(_, t)| t.kind == kind::INVERTER).map(|(i, _)| i).collect();
    eprintln!("[inv] inverter triggers: {}", inv.len());
    for &i in &inv {
        let t = &list[i];
        eprintln!("[inv] placement {} out {} input {:?}", t.placement, t.out, t.list);
    }
    assert_eq!(inv.len(), 18, "07000014 carries 18 type-0xF9 placements (measured)");
    let idx = inv.iter().copied().find(|&i| list[i].placement == 390).expect("the #390 inverter [D]");
    let t = &list[idx];
    assert_eq!(t.out, 1, "its output is p1 = 0x1 [D]");
    assert_eq!(t.list.first().copied(), Some(0x87), "its input is p0 = 0x87 [D]");

    let mut ch = Channels::default();
    ch.set(0x87, 1, 0);
    ch.set(1, 1, 0);
    triggers::calc(0x0700_0014, &mut list, &mut ch, &[], &[], 1);
    eprintln!("[inv] after cut-tick: ch87 {} ch1 {}", ch.on(0x87), ch.on(1));
    assert!(ch.on(0x87), "the input stays on");
    assert!(!ch.on(1), "the inverter clears its output while the input is on [M 0x1b30a8]");
}
