//! The driving view clears to the mission's fog colour (the Render/Fog/<track>.tun text, FogColour =
//! a,b,c,d → GS FOGCOL R,G,B = b,c,d [M FUN_00144740 / FUN_001ac978 / FUN_0028cf98, nf_formats::rendertun]):
//! paris (MIS01) is 255,22,15,20 → (22, 15, 20), so the open sky above the sky bands is that dusk colour,
//! not black. Opt-in: NF_ISO.

#[test]
fn the_paris_drive_clears_to_its_fog_colour() {
    let Ok(iso) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set — skipping"); return };
    let (scene, _) = nf_view::driving::load_driving_scene(&iso, "MIS01").expect("load MIS01");
    let (c, end) = scene.fog.expect("the driving scene carries the mission fog");
    assert_eq!(c, [22, 15, 20], "FogColour 255,22,15,20 → R,G,B = fields 2,3,4");
    assert!((end - 1450.21).abs() < 0.01, "fogEND = 1450.21: {end}");
}
