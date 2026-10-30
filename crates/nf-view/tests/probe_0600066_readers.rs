//! script-channel-events: who READS 07000006's intro channels (0x3d/0x3e/0x3f/0x40) [D] —
//! triggers (out/ins), doors (ch_open), sensors, SS, locks, mission objectives, and the SP
//! placements that gate on a channel (the game's SP_Hit path [M 0x1dc470:17-19]: a mask bit-1
//! SP fires when its OUT channel reads nonzero — the 0700004a ch-0x40 → SP #289 precedent).
//! Prints only.
use nf_game::triggers::kind;

#[test]
#[ignore = "assertion-free dump; run with --ignored"]
fn dump_0600066_channel_readers() {
    let iso = std::env::var("NF_ISO").expect("NF_ISO");
    let scene = nf_view::load(&iso, "07000006.bin", false).expect("load");
    let sim = scene.sim.as_ref().unwrap();
    let d = &sim.drones;
    let want = [0x3du8, 0x3e, 0x3f, 0x40, 0x02];
    for t in &d.triggers {
        let mut hits = Vec::new();
        if want.contains(&t.out) { hits.push(format!("OUT 0x{:02x}", t.out)); }
        for &c in &t.list { if want.contains(&(c as u8)) { hits.push(format!("IN 0x{c:02x}")); } }
        if !hits.is_empty() { println!("trigger pl {} kind {}: {}", t.placement,
            match t.kind { kind::TOUCH_ONCE => "TOUCH_ONCE", kind::TOUCH => "TOUCH", kind::AND => "AND", kind::OR => "OR", kind::SEQ_AND => "SEQ_AND", kind::OUT => "OUT", _ => "other" }, hits.join(" ")); }
    }
    for dr in &d.doors.list {
        if want.contains(&(dr.ch_open as u8)) { println!("door ch_open 0x{:02x}", dr.ch_open); }
    }
    for s in &d.sensors.list { if want.contains(&(s.out as u8)) { println!("sensor out 0x{:02x}", s.out); } }
    for s in &d.sss.list { if want.contains(&(s.channel as u8)) { println!("SS ch 0x{:02x} mask {:04x}", s.channel, s.mask); } }
    for l in &d.locks { if want.contains(&(l.out as u8)) { println!("lock out 0x{:02x}", l.out); } }
    for f in &d.fx.flickers { if want.contains(&(f.signal_channel as u8)) { println!("flicker pl {} signal ch 0x{:02x}", f.placement, f.signal_channel); } }
    for e in &d.fx.emitters { if want.contains(&(e.ch_out as u8)) { println!("emitter ch_out 0x{:02x}", e.ch_out); } }
    for sl in &d.searchlights.parts {
        for (nm, c) in [("on", sl.on), ("off", sl.off), ("alarm", sl.alarm)] {
            if want.contains(&c) { println!("searchlight pl {} {nm} ch 0x{c:02x}", sl.placement); }
        }
    }
    for part in nf_game::mission::LEVELS.iter().filter(|m| m.level == 0x0700_0006) {
        for o in part.objectives {
            if want.contains(&o.channel) { println!("objective mission {:08x} obj {:08x} watches ch 0x{:02x}", part.mission, o.text, o.channel); }
        }
    }
}

/// The SP placements on 07000006 whose params reference the intro channels [D].
#[test]
#[ignore = "assertion-free dump; run with --ignored"]
fn dump_0600066_sp_placements() {
    use nf_formats::iso9660::IsoImage;
    let iso = std::env::var("NF_ISO").expect("NF_ISO");
    let img = IsoImage::open(&iso).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("07000006.bin")).expect("archive");
    let data = img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read");
    let want = [0x3du32, 0x3e, 0x3f, 0x40];
    let level = nf_formats::level::Level::parse(&data).expect("level");
    for (mi, m) in level.maps.iter().enumerate() {
        for (i, pl) in m.placements.iter().enumerate() {
            if pl.flags & 0xff != 0xd9 { continue; }
            let hit: Vec<String> = pl.params.iter()
                .filter(|(_, p)| want.contains(p))
                .map(|(k, p)| format!("p{k}=0x{p:02x}"))
                .collect();
            if !hit.is_empty() {
                println!("SP map{mi} idx {i}: hash {:08x} flags {:08x} params {:x?} → {}", pl.hashcode, pl.flags, pl.params.iter().map(|(k, v)| format!("{k}:{v:x}")).collect::<Vec<_>>().join(","), hit.join(" "));
            }
        }
    }
}
