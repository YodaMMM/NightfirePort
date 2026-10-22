//! standins-drone-920: the two stand-in markers whose rule changed behaviour, through the real
//! path (`nf_view::load` + `Sim::step_bond`).
//!
//! (The attack-hold gate's real-path test — the crosshair ON a 40-unit guard, past his sight
//! range — is left work: the port's aim ray is look + tx·right + ty·up through the crosshair
//! offsets and the 40-unit cylinder pick needs the ray geometry chased further; the CODE change
//! itself is [M]-cited in drone.rs and covered by the decompile chain 0x1a7810:148-151 →
//! 0x148db0:24-34.)
//! - The MP bots' pickup-take test reads each body's world sphere centre obj+0x80 [M
//!   `Pickup_Update` 0x18e148:212-214]; the bots' `NetBody.sph` is the [L]-backed standing local
//!   offset through the object matrix [M `Control_BuildWorldSph` 0x133070:33-44; the LOCAL read
//!   is `ee_start_05.bin` M_CONTROL+0x80], not the feet position.
//!
//! Opt-in via `NF_ISO`.

use nf_game::player::Input;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

/// The bots' pickup-take bodies carry the real world sphere centre: `NetBody.sph` is the
/// [L]-backed local offset through the object matrix (the bot's own yaw), not the feet position —
/// and the human slot's body keeps `Bond::sphere_centre()` (the two arms agree).
#[test]
fn mp_bot_bodies_carry_the_world_sphere() {
    let Some(path) = iso() else { eprintln!("skip: NF_ISO not set"); return };
    let mut s = nf_game::mp::MpSettings::deathmatch(0x0700_0023);
    s.human_count = 1;
    s.player_count = 1;
    s.total_players = 1;
    s.bot_count = 1;
    s.seed = 0x0bed_0bad;
    let arena = format!("{:08x}.bin", s.level_code);
    let mut scene = nf_view::load_mp_carry(&path, &arena, false, None, s).expect("arena loads");
    for _ in 0..5 { scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false); }
    let bodies = scene.sim.as_ref().unwrap().net_bodies_now();
    let bot = bodies.iter().find(|b| b.bot).expect("the bot body exists");
    assert_ne!(bot.sph, bot.pos, "the bot's sph is the world sphere centre, not the feet [M 0x133070:33-44]");
    // The formula identity: the centre's y is pos.y + LOCAL[1] whatever the yaw [L M_CONTROL+0x80].
    assert!((bot.sph[1] - (bot.pos[1] - 0.1267)).abs() < 1e-4,
        "the centre's y rides the standing local offset [L ee_start_05 M_CONTROL+0x80]");
    // Sister: the human slot's body is the real `Bond::sphere_centre()`.
    let human = bodies.iter().find(|b| !b.bot).expect("the human body exists");
    let want_human = scene.sim.as_ref().unwrap().bond.sphere_centre();
    assert!((human.sph[0] - want_human[0]).abs() < 1e-4
        && (human.sph[1] - want_human[1]).abs() < 1e-4
        && (human.sph[2] - want_human[2]).abs() < 1e-4,
        "the human body's sph is Bond::sphere_centre()");
}
