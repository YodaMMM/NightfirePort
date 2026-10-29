//! walkreal probe: does a FRESH 07000003 load write ch 0x4D — the part-1 obj2 ("Find door to
//! servants quarters", watch 0x4D [L c07000002]) completion hand-off, by the arc's own pattern
//! (07000002's intro stream 7 writes 0x47, part-0's watch [D 07800002 0x06000084])? Ticks the
//! fresh load through the real script-player path. Diagnostic only — --nocapture.

#[path = "common/walker.rs"]
mod walker;

use walker::{Act, Walker};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
fn probe_07000003_fresh_0x4d() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000003.bin", false).expect("load 07000003"));
    w.skip_cutscenes = false;
    let mut got = None;
    for t in 0..4800u32 {
        if w.scene.sim.as_ref().unwrap().drones.channels.on(0x4d) { got = Some(t); break; }
        w.tick(Default::default(), false);
    }
    let sim = w.scene.sim.as_ref().unwrap();
    eprintln!("ch 0x4D on: {:?} at tick", got);
    eprintln!("ch 0x47 on: {}", sim.drones.channels.on(0x47));
    eprintln!("ch 0x96 on: {} (the 03 monitor channel [D])", sim.drones.channels.on(0x96));
    eprintln!("mission row 07000003 states: {:?}", sim.drones.mission.states[6]);
    let row = nf_game::mission::LEVELS.iter().position(|l| l.level == 0x0700_0002).unwrap();
    eprintln!("mission row 07000002 (carried) states: {:?}", sim.drones.mission.states[row]);
    let _ = Act::None;
}

#[test]
fn probe_07000003_doors_and_sps() {
    let iso = iso();
    let scene = nf_view::load(&iso, "07000003.bin", false).expect("load 07000003");
    let sim = scene.sim.as_ref().unwrap();
    for (i, d) in sim.drones.doors.list.iter().enumerate() {
        eprintln!(
            "door[{i}] placement={} at=({:.1},{:.1},{:.1}) kind={} ch_open={:#04x} ch_lock={:#04x} state={:#x} approach={}",
            d.placement, d.pos[0], d.pos[1], d.pos[2], d.kind, d.ch_open, d.ch_lock, d.state, d.approach
        );
    }
}

#[test]
fn probe_07000003_param77_and_scripts() {
    let iso = iso();
    let img = nf_formats::iso9660::IsoImage::open(&iso).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("07000003.bin")).expect("archive");
    let data = img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read");
    let level = nf_formats::level::Level::parse(&data).expect("parse 07000003");
    let m = level.main_map().expect("main map");
    for p in &level.maps[m].placements {
        for (k, v) in &p.params {
            if *v == 0x4d {
                eprintln!("param77: placement={} flags={:#x} hash={:#x} param[{}]={:#x} at=({:.1},{:.1},{:.1})",
                    p.index, p.flags, p.hashcode, k, v, p.translation[0], p.translation[1], p.translation[2]);
            }
        }
        if p.flags & 0xff == 0xd9 {
            eprintln!("SP: placement={} flags={:#x} params={:?} at=({:.1},{:.1},{:.1})",
                p.index, p.flags, p.params, p.translation[0], p.translation[1], p.translation[2]);
        }
        if p.flags & 0xff == 0xe8 {
            eprintln!("LOADLEVEL: placement={} params={:?} at=({:.1},{:.1},{:.1})",
                p.index, p.params, p.translation[0], p.translation[1], p.translation[2]);
        }
    }
    // every script the sim carries: any Event 0xD (channel write) touching 0x4D
    let scene = nf_view::load(&iso, "07000003.bin", false).expect("load 07000003");
    let sim = scene.sim.as_ref().unwrap();
    for (hash, sc) in &sim.scripts {
        let dump = format!("{:?}", sc);
        if dump.contains("0x4d") || dump.contains("77") {
            eprintln!("script {:#x}: mentions 0x4d/77 (inspect)", hash);
        }
    }
    eprintln!("done");
}
