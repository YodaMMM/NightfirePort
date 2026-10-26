//! 07000014's channel inverters: the game's InverterTrigger is the kind-0x41 placement —
//! `parsemap_create_dynamic_objects` 0x1d04f0 case 0x41 → `InverterTrigger_Create` 0x1b3040
//! (class 0x53; p0 → obj+0xF4, p1 → obj+0xF6 [M 0x1b3040:13-18]), and `InverterTrigger_Update`
//! 0x1b30a8 is `if (switch_channels[*(short*)(+0xF4)] != 0 && *(short*)(+0xF6) != 0)
//! switch_channels[*(short*)(+0xF6)] = 0;` — the second gate is the raw OUT FIELD, not the
//! channel's value [M 0x1b30a8:8-11]. The kind-0xF9 placements are NOT inverters: case 0xf9 calls
//! `SoundTrigger_Create` 0x1b2628 (class 0x40) and `SoundTrigger_Update` 0x1b2790 stores NO
//! switch_channels at all — p0 is the ON gate, p1 the one-shot flag, p2 the sound id
//! [M 0x1b2628:41-48; 0x1b2790]. (A previous cut built INVERTER triggers from the 0xF9 rows,
//! reading p0/p1 as the in/out channel pair; both reads are wrong on this disc, where every 0xF9
//! row is a voice line — the pair at the generator room gates on the fusebox-cut channels
//! 0x87/0x88 with sound 0xf1 [D]. The port's worldobj inverters (0x41) own the real behaviour;
//! nf-view's worldobj_live::inverter_07000006_forces_the_out_channel_low pins the tick on level 6.)
use nf_formats::iso9660::IsoImage;
use nf_formats::level::Level;
use nf_formats::{disc, placement::Placement};

fn level_14() -> Level {
    let iso = std::env::var("NF_ISO").expect("NF_ISO");
    let image = IsoImage::open(&iso).expect("iso");
    let (entries, bin) = disc::files_bin_entries(&image).expect("entries");
    let fe = entries
        .iter()
        .find(|e| e.name.eq_ignore_ascii_case("07000014.bin"))
        .expect("14");
    let data = image
        .read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64)
        .expect("read");
    Level::parse(&data).expect("parse")
}

#[test]
fn the_14_inverters_are_the_0x41_placements_and_the_f9_rows_are_voice_lines() {
    let level = level_14();
    let m = level.main_map().expect("main map");
    let placements = &level.maps[m].placements;
    let param = |p: &Placement, k: u32| {
        p.params.iter().find(|(j, _)| *j == k).map(|&(_, v)| v).unwrap_or(0)
    };

    // [D] The disc carries 18 kind-0xF9 rows and 4 kind-0x41 rows on the main map.
    let f9: Vec<&Placement> =
        placements.iter().filter(|p| p.flags & 0xff == 0xf9 && p.flags & 0xa000 == 0).collect();
    let k41: Vec<&Placement> =
        placements.iter().filter(|p| p.flags & 0xff == 0x41 && p.flags & 0xa000 == 0).collect();
    assert_eq!(f9.len(), 18, "07000014's kind-0xF9 rows (measured on the disc)");
    assert_eq!(k41.len(), 4, "07000014's kind-0x41 rows (measured on the disc)");

    // [M 0x1d04f0 case 0xf9 → SoundTrigger_Create] the port builds NO trigger from any 0xF9 row:
    // the trigger scan maps only 0xEA/0xEB/0xEC/0xED/0xEE/0xEF/0xE8/0xF4/0xDC (and every built
    // trigger's placement is that scan's row).
    let triggers = nf_game::triggers::build(&level);
    let f9_rows: std::collections::HashSet<usize> = placements
        .iter()
        .enumerate()
        .filter(|(_, p)| p.flags & 0xff == 0xf9)
        .map(|(i, _)| i)
        .collect();
    assert!(
        triggers.iter().all(|t| !f9_rows.contains(&t.placement)),
        "no trigger is built from a kind-0xF9 placement [M 0x1d04f0 case 0xf9 = SoundTrigger_Create]"
    );

    // [D] The four 0x41 rows' measured param pairs — the game's real inverters on this level.
    let mut pairs: Vec<(u32, u32)> =
        k41.iter().map(|p| (param(p, 0), param(p, 1))).collect();
    pairs.sort_unstable();
    assert_eq!(
        pairs,
        vec![(0x0e, 0xb8), (0x85, 0x22), (0x85, 0x6f), (0x85, 0xd1)],
        "the 0x41 rows gate the hatch lights and one corridor line off ch 0x85 / ch 0x0e [D]"
    );

    // The port builds exactly these through the game's creator path, in placement order.
    let w = nf_game::worldobj::WorldObjects::create(placements, &[], |_| (0, [0.0; 3]));
    let got: Vec<(u16, u16)> = w.inverters.iter().map(|i| (i.ch_in, i.ch_out)).collect();
    assert_eq!(
        got,
        vec![(0x0e, 0xb8), (0x85, 0x6f), (0x85, 0x22), (0x85, 0xd1)],
        "InverterTrigger_Create 0x1b3040: p0 → +0xF4, p1 → +0xF6 [M 0x1b3040:14-16]"
    );

    // [M 0x1b30a8] the update rule on the level's own inverter: ch 0x0e on + ch 0xb8 on → 0xb8
    // cleared, the input stays on; and the cleared output stays low while the input is on.
    let mut ch = nf_game::triggers::Channels::default();
    ch.set(0x0e, 1, 0);
    ch.set(0xb8, 1, 0);
    let inv = w.inverters.iter().find(|i| i.ch_in == 0x0e).expect("the 0x0e inverter");
    inv.tick(&mut ch, 1);
    assert!(!ch.on(0xb8), "the inverter holds its output low while its input is on [M 0x1b30a8]");
    assert!(ch.on(0x0e), "the input channel is untouched [M 0x1b30a8]");
    ch.set(0xb8, 1, 2);
    inv.tick(&mut ch, 3);
    assert!(!ch.on(0xb8), "a re-raised output is cleared again on the next tick [M 0x1b30a8]");

    // [D] The two generator-room voice rows gate on the fusebox-cut channels 0x87/0x88 (p0), one
    // shot (p1 = 1), sound 0xf1 (p2). [M 0x1b2628:41-48] reads p0 → [0], p1 → [2], p2 → [3];
    // [M 0x1b2790] stores no switch_channels, so raising the cut channel must NEVER clear ch 1 —
    // the regression the 0xF9→inverter cut introduced (it cleared ch 1 from p1 = the one-shot flag).
    let mut voices: Vec<(u32, u32, u32)> = f9
        .iter()
        .filter(|p| param(p, 2) == 0xf1)
        .map(|p| (param(p, 0), param(p, 1), param(p, 2)))
        .collect();
    voices.sort_unstable();
    assert_eq!(
        voices,
        vec![(0x22, 1, 0xf1), (0x87, 1, 0xf1), (0x88, 1, 0xf1), (0xd1, 1, 0xf1)],
        "the sound-0xf1 voice rows — 0x87/0x88 the fusebox-cut gates, 0x22/0xd1 the 0x85-inverter \
         outputs [D]"
    );
    let st = nf_game::sound::SoundTriggers::create(placements, 1);
    let v87 = st.list.iter().find(|t| t.on == 0x87 && t.id == 0xf1).expect("the 0x87 voice row");
    assert_eq!(v87.one_shot, 1, "p1 is the one-shot flag, not a channel [M 0x1b2790's puVar2[2]]");
    let mut audio = nf_game::sound::Sounds::default();
    let mut ch = nf_game::triggers::Channels::default();
    ch.set(0x87, 1, 0);
    ch.set(1, 1, 0);
    let mut st = st;
    for _ in 0..4 {
        st.tick(&mut audio, &ch);
    }
    assert!(st.playing(0xf1), "the cut-channel voice line arms [M 0x1b2790's play arm]");
    assert!(ch.on(1), "the voice line never writes switch_channels — ch 1 stays on [M 0x1b2790]");
    assert!(ch.on(0x87), "the gate channel is untouched [M 0x1b2790]");
}
