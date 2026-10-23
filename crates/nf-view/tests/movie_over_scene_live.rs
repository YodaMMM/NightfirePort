//! movie-over-scene (movies played their audio but showed no picture): since doublecross-transitions chained `Player_Init`'s entry fade (45 ticks,
//! [M 0x1910f8:45]) into the scene's script overlay, the idle scene behind the boot/briefing movies
//! held it at full black — it only counts down while the sim ticks — and the HUD pass drew it OVER
//! the movie plane. The movie owns the screen: no scene overlay while a movie frame is up.

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
fn a_movie_frame_carries_no_scene_fade() {
    let scene = nf_view::load(&iso(), "07000001.bin", false).expect("load 07000001");
    // Fresh load, never ticked: the entry fade is at its full 45/45.
    let fade = scene.sim.as_ref().unwrap().drones.spawn_fade_view().expect("the entry fade is armed at load");
    assert!(fade.alpha >= 0.99, "the untouched entry fade is opaque (alpha {})", fade.alpha);
    let with_movie = nf_view::shot_hud(&scene, 16.0 / 9.0, None, true);
    let without = nf_view::shot_hud(&scene, 16.0 / 9.0, None, false);
    // Sister: with no movie the fade quad is in the frame.
    assert!(without.v.len() > with_movie.v.len(), "the scene frame carries the fade ({} vs {} vertices)", without.v.len(), with_movie.v.len());
    let script_only = scene.script_overlay_hud(16.0 / 9.0);
    assert!(!script_only.v.is_empty(), "the script overlay has the fade quad");
    assert_eq!(with_movie.v.len() + script_only.v.len(), without.v.len(), "the movie frame drops exactly the scene's script overlay");
}

/// The front end: `ResetMap_Load` runs `Player_Start` on 0x07000048 too [M 0x1d6360:154], so the
/// front-end scene carries the armed entry fade — the port's front-end scene never ticks it away,
/// and it drew black over the title page (art 0x03000074 + the prompt). The front-end level draws
/// no scene overlay; sister: a gameplay level (07000001, untouched entry fade) still does.
#[test]
fn the_front_end_level_draws_no_scene_fade() {
    let fe = nf_view::load(&iso(), "07000048.bin", false).expect("load the front-end level");
    assert!(fe.sim.as_ref().unwrap().drones.spawn_fade_view().is_some(), "the front end's entry fade is armed (Player_Start runs there)");
    assert!(!nf_view::scene_overlay_up(&fe, false), "no scene overlay on the front-end level");
    let game = nf_view::load(&iso(), "07000001.bin", false).expect("load 07000001");
    assert!(nf_view::scene_overlay_up(&game, false), "sister: a gameplay level draws its overlay");
    assert!(!nf_view::scene_overlay_up(&game, true), "sister: not over a movie frame");
}
