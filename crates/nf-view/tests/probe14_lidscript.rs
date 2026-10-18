//! level-14 probe: the lid script — decode 07000014's script 0x06000A3B (every Vent_StoreRoom_
//! Graphic lid's ScriptPlayer) and the vent-area SP scripts, stream by stream, to learn what moves
//! the lids and on which channels. Prints only.

use nf_formats::iso9660::IsoImage;
use nf_formats::level::Level;
use nf_formats::nis::Cmd;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn dump_nis(level: &Level, hash: u32, label: &str) {
    let Some(n) = level.nis.get(&hash) else { eprintln!("{label}: script {hash:08x} NOT in the level's nis map"); return };
    eprintln!("== {label}: script {hash:08x} camera={} len={} loop={} streams {} ==", n.camera, n.length, n.loop_start, n.streams.len());
    for (si, s) in n.streams.iter().enumerate() {
        eprintln!(" stream {si}: first_key {} last_key {}", s.first_key, s.last_key);
        for c in s.decode() {
            match c {
                Cmd::Entity { hash, parent, obj_type, params, .. } => eprintln!("   Entity {hash:08x} parent{parent:?} type{obj_type} params {:02x?}", params.iter().map(|p| *p as u32).collect::<Vec<_>>()),
                Cmd::Event { kind, params } => eprintln!("   Event kind {kind} params {:08x?}", params),
                Cmd::Anim { script, skin, start } => eprintln!("   Anim script {script:08x} skin {skin:08x} start {start}"),
                Cmd::Wait(t) => eprintln!("   Wait {t}"),
                Cmd::Marker(t) => eprintln!("   Marker {t:#x}"),
                Cmd::End(t) => eprintln!("   End {t:#x}"),
                Cmd::Sound { id, stream, .. } => eprintln!("   Sound {id:08x} stream {stream}"),
                Cmd::SubScript { hash, stream, mode } => eprintln!("   SubScript {hash:08x} stream {stream} mode {mode:#x}"),
                Cmd::Camera { spline, first, last } => eprintln!("   Camera spline {spline} keys {first}..{last}"),
                Cmd::Text { label, time } => eprintln!("   Text {label:08x} time {time}"),
                other => eprintln!("   {other:?}"),
            }
        }
    }
}

#[test]
fn probe_lid_script() {
    let iso = IsoImage::open(&iso()).unwrap();
    let (entries, bin) = nf_formats::disc::files_bin_entries(&iso).unwrap();
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("07000014.bin")).expect("level");
    let data = iso.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).unwrap();
    let level = Level::parse(&data).unwrap();

    dump_nis(&level, 0x0600_0A3B, "the lid script (Vent_StoreRoom_Graphic)");
    // The vent-area cut-scene + any other script referenced near the route, for context.
    dump_nis(&level, 0x0600_0833, "SP #276 the vent-drop cut-scene");
}
