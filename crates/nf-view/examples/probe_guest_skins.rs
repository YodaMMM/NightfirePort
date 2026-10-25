//! One-shot probe: are the 07000007 guest skins morph-bearing (Skin +0x12 facial count > 0)?
//! The guests seen talking in probe_guest_face: skins 0x05000024 (vt 13) / 0x05000019 (vt 14).
//! Opt-in: NF_ISO.
use nf_game::player::Input;

fn main() {
    let Some(iso) = std::env::var("NF_ISO").ok() else { eprintln!("NF_ISO not set"); return };
    let scene = nf_view::load(&iso, "07000007.bin", false).expect("load 07000007");
    let sim = scene.sim.as_ref().unwrap();
    let mut skins: Vec<u32> = sim.drones.list.iter()
        .filter(|d| d.present && d.voice_type >= 9 && d.voice_type <= 20)
        .map(|d| d.skin).collect();
    skins.sort(); skins.dedup();
    for sk in &skins {
        let f = sim.skins.get(sk).map(|s| s.facial).unwrap_or(255);
        let morph = scene.characters.iter().find(|c| c.skin.hashcode == *sk)
            .map(|c| c.morph_targets.is_some()).unwrap_or(false);
        println!("skin {sk:08x} facial {f} char_morph_targets {morph}");
    }
    let mut cis: Vec<_> = (0..sim.char_drone.len())
        .filter_map(|ci| sim.char_drone[ci].map(|di| (ci, di)))
        .collect();
    cis.truncate(6);
    for (ci, di) in cis {
        let d = &sim.drones.list[di];
        println!("char {ci} <- drone {di} skin {:08x} vt {} present {}", d.skin, d.voice_type, d.present);
    }
}
