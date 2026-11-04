//! level-12-copter-sound-distance probe: the copter's sound chain on disc. The kind-0x2f
//! placements and their SP scripts' Sound lines (id, flat=2-D flag, object ref), then for every
//! id seen (plus the gun 0x5df): the EE SFXOutputData row [D ACTION.ELF] and the bank Effect
//! parameter block [D the level's .SFX] — the two tables the game's inner/outer come from.
//! Assertion-free diagnostic, run with --ignored --nocapture.

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

use nf_formats::nis::Cmd;

#[test]
#[ignore = "assertion-free dump; run with --ignored"]
fn dump_07000012_copter_sound_chain() {
    let iso = iso();
    let scene = nf_view::load(&iso, "07000012.bin", false).expect("load 07000012");
    let sim = scene.sim.as_ref().unwrap();
    let img = nf_formats::iso9660::IsoImage::open(&iso).unwrap();
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).unwrap();
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("07000012.bin")).unwrap();
    let data = img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).unwrap();
    let lv = nf_formats::level::Level::parse(&data).unwrap();
    println!("=== kind-0x2f placements (copters + their paths) ===");
    let mut script_hashes = Vec::new();
    for (i, p) in lv.maps.iter().flat_map(|m| &m.placements).enumerate() {
        if p.flags & 0xff != 0x2f { continue; }
        let param = |k: u32| p.params.iter().find(|(j, _)| *j == k).map_or(0, |&(_, v)| v);
        println!("plc #{i} pos=({:.2},{:.2},{:.2}) p0={} p2={} p3={} p4(script)={:08x} p5(script)={:08x} p6(gate)={} p7(out)={} p8(arr)={} p9(cd)={}",
            p.translation[0], p.translation[1], p.translation[2], param(0), param(2), param(3), param(4), param(5), param(6), param(7), param(8), param(9));
        for k in [4u32, 5] { let h = param(k); if h != 0 { script_hashes.push(h); } }
    }
    println!("\n=== live copter SP players ===");
    for p in &sim.drones.scripts.players {
        if p.scripts.iter().any(|&h| script_hashes.contains(&h)) {
            println!("player placement #{} scripts [{:08x}, {:08x}] modes [{}, {}] gate {:#x} out {:#x}",
                p.placement, p.scripts[0], p.scripts[1], p.modes[0], p.modes[1], p.gate, p.out);
        }
    }
    println!("\n=== Sound lines in those scripts (clock = summed Waits) ===");
    let mut ids = vec![0x5dfu32, 0x5de];
    for h in &script_hashes {
        let Some(nis) = sim.drones.scripts.lib.get(h) else { println!("script {h:08x}: NOT in lib"); continue };
        println!("script {h:08x}: length {} camera {} loop_start {}", nis.length, nis.camera, nis.loop_start);
        for (si, s) in nis.streams.iter().enumerate() {
            let mut clock = 0u32;
            for c in s.decode() {
                match c {
                    Cmd::Wait(t) => clock += t as u32,
                    Cmd::Sound { id, flat, has_obj, stream } => {
                        println!("  script {h:08x} stream {si} t={clock}: SOUND id={id:#x} flat={flat} has_obj={has_obj} obj_stream={stream}");
                        ids.push(id & 0xfffff);
                    }
                    Cmd::SoundEnd => println!("  script {h:08x} stream {si} t={clock}: SOUNDEND"),
                    Cmd::SubScript { hash, stream, mode } => println!("  script {h:08x} stream {si} t={clock}: SUB {hash:08x} stream {stream} mode {mode}"),
                    Cmd::Entity { hash, parent, spline, obj_type, params } => println!("  script {h:08x} stream {si} t={clock}: ENTITY {hash:08x} parent={parent:?} spline={spline} type={obj_type} params={params:?}"),
                    Cmd::EntityEnd => println!("  script {h:08x} stream {si} t={clock}: ENTITYEND"),
                    Cmd::Anim { script, skin, start } => println!("  script {h:08x} stream {si} t={clock}: ANIM {script:08x} skin {skin:08x} start {start}"),
                    Cmd::End(t) => println!("  script {h:08x} stream {si} t={clock}: END {t}"),
                    Cmd::Marker(t) => println!("  script {h:08x} stream {si} t={clock}: MARKER {t}"),
                    Cmd::Unread(op) => println!("  script {h:08x} stream {si} t={clock}: UNREAD {op:#x}"),
                    Cmd::Skip { op } => println!("  script {h:08x} stream {si} t={clock}: SKIP {op:#x}"),
                    _ => {}
                }
            }
        }
    }
    ids.sort(); ids.dedup();
    println!("\n=== the ids in both tables ===");
    for id in ids {
        let (inn, out, w, lp, cull) = sim.sfx.as_ref()
            .map(|t| t.get(id)).map(|h| (h.inner, h.outer, h.weight, h.looped, h.cull))
            .unwrap_or((f32::NAN, f32::NAN, f32::NAN, false, false));
        print!("id {id:#05x}: SFXOutputData inner={inn} outer={out} weight={w} looped={lp} cull={cull}");
        if let Some((slot, bank, _)) = scene.sound_bank.as_ref() {
            if let Some(block) = bank.effect(id) {
                if let Ok(e) = nf_formats::soundbank::Effect::parse(block) {
                    print!(" | bank(slot {slot}) tracking={} inner={} outer={} max_voices={} priority={} master_vol={} looped={} outdoors={} ducker={}",
                        e.tracking, e.inner, e.outer, e.max_voices, e.priority, e.master_volume, e.looped, e.outdoors, e.ducker);
                    for (k, s) in e.subs.iter().enumerate() {
                        print!(" | sub{k}: file={} vol={} vol_rnd={} pan={} pan_rnd={}", s.file, s.volume, s.volume_rnd, s.pan, s.pan_rnd);
                    }
                } else { print!(" | bank(slot {slot}): param block UNPARSEABLE"); }
            } else { print!(" | bank(slot {slot}): id absent"); }
        } else { print!(" | no level bank"); }
        println!();
    }
}
