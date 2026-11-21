//! The driving view's engine sound: the track's Engine bank (banks.ini slot 9) names the ten layer
//! loops (the vanquish has no midload pair [D]), decoded from EA-XA with their loop points; the
//! game's layer logic (nf_game::driving::audio::EngineSound) drives their volume and pitch each
//! tick. Tests never open an audio device (NF_SILENT; `start_audio` is the window app's). Opt-in: NF_ISO.

#[test]
fn the_vanquish_engine_layers_load_and_follow_the_throttle() {
    let Ok(iso) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set — skipping"); return };
    let (_, mut d) = nf_view::driving::load_driving_scene(&iso, "MIS4").expect("load MIS4");
    let e = d.engine.as_ref().expect("the engine bank");
    let loaded = e.voices.iter().filter(|v| v.is_some()).count();
    assert_eq!(loaded, 8, "ten layers minus the missing midload pair");
    assert!(e.voices.iter().flatten().all(|(p, r)| !p.data.is_empty() && *r > 8000), "decoded loops with their rates");
    assert!(e.output.is_none(), "no device in tests");
    for _ in 0..60 { d.step(0.0, 0.0, 0.0, false); }
    let idle = d.engine.as_ref().unwrap().snd.load;
    for _ in 0..120 { d.step(1.0, 0.0, 0.0, false); }
    let busy = d.engine.as_ref().unwrap().snd.load;
    assert!(busy > idle + 0.3, "the load follows the throttle: {idle} -> {busy}");
}
