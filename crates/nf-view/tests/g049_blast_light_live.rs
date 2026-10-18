//! G-049 (07000008 — the rocket blast does not light the end wall): the blast script's keyed light
//! must carry the game's radius and colour. A live PCSX2 read of the LightList during an 08 rocket
//! blast (2026-10-18, scratchpad blast08b.py: `LightList` 0x2c74cc, record +0x3C radius, +0x44..46
//! colour bytes) showed the impact light at radius 96.0, colour bytes (255, 197, 134), for 81 frames
//! [L]. The port drew radius 768 (the script scale applied twice — `keys` already returns key scale
//! × SCRIPTINFO+0x94 [M Script_GetInterp 0x1da158:177]) and (134, 197, 255) (the key bytes read in
//! order — `Script_GetInterp` packs +0x26 << 24 [M 0x1da158:174] and `Script_Interp` case 6 stores
//! bits 24..31 at +0x44 [M 0x1d9a08:244-246]).

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

#[test]
fn the_rocket_blast_light_has_the_games_radius_and_colour() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = nf_view::load(&iso, "07000008.bin", false).expect("load 07000008");
    // The launcher's blast: `Explode_Create` plays 0x06000052 at scale = weapon_data[0x2d]+0x08
    // (the 0x4 arm's f12 [M disasm 0x12ce1c-0x12ce2c]).
    let scale = {
        let sim = scene.sim.as_ref().unwrap();
        let r = sim.weapons.records.get(0x2d).expect("the launcher record");
        f32::from_le_bytes(r[0x08..0x0c].try_into().unwrap())
    };
    {
        let sim = scene.sim.as_mut().unwrap();
        assert!(sim.drones.scripts.lib.contains_key(&0x0600_0052), "the blast script is in 08's lib");
        let m = [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [139.55, 50.77, -96.15, 1.0]];
        sim.drones.scripts.spawn_once_scaled(usize::MAX, 0x0600_0052, m, scale);
    }
    let light = |scene: &nf_view::scene::Scene| scene.sim.as_ref().unwrap().drones.scripts.objects.list.iter()
        .find_map(|o| match o { Some(nf_game::nis::Obj::Light { radius, rgb, live: true, .. }) => Some((*radius, *rgb)), _ => None });
    let mut seen = None;
    for _ in 0..20 {
        scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false);
        if let Some(l) = light(&scene) { seen = Some(l); break; }
    }
    let (radius, rgb) = seen.expect("the blast's keyed light is live within 20 ticks");
    eprintln!("blast light: scale {scale}, radius {radius}, rgb {rgb:?}");
    assert!((radius - 96.0).abs() < 0.5, "radius {radius}, PCSX2 reads 96.0 [L]");
    assert_eq!(rgb, [255, 197, 134], "the warm colour, R first [L + M]");
    // Sister: the pre-fix values are what this rules out.
    assert_ne!(rgb, [134, 197, 255]);
    assert!((radius - 768.0).abs() > 1.0);
}
