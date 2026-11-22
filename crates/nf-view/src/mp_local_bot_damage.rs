//! mp-local-bot-damage: in a LOCAL (offline) bot match the player's own rounds hit and hurt the
//! bots through the same pain path the online combat and the bots' rounds use (`Sim::mp_hurt` →
//! the ported `BOT_handlePain` 0x125ed8), and a kill scores for the player.
//!
//! Real path only: boot, the real MP menus by `App::menu_input` presses (Multiplayer, Local,
//! Arena Deathmatch, map 0x07000024, one bot: Speed 0, Skill Poor, Hit points 50, Confirm), then
//! in the match the player's look through `mouse_look_add` (the MouseMotion arm), W through
//! `app_key`, the trigger through `app.trigger` (the left mouse button). NF_ISO-gated.
//!
//! mp-bot-hit-fx: the same hit shows the bot's flesh puff and plays its pain line
//! (`local_match_player_round_on_the_bot_puffs_and_barks`).
use super::*;
use crate::headless_app_harness::HeadlessApp;
use menu::{Action as MA, Page};
use winit::keyboard::KeyCode;

fn page(h: &HeadlessApp) -> Option<Page> { h.app.fe.as_ref().map(|f| f.page) }

fn press(h: &mut HeadlessApp, a: MA) {
    h.app.menu_input(a);
    h.frames(20);
    let _ = h.pump(3.0, |a| a.fe.as_ref().map_or(true, |f| !f.close_iris_in_flight()));
}

fn press_until(h: &mut HeadlessApp, a: MA, want: Page, n: usize) {
    for _ in 0..n {
        if page(h) == Some(want) { return; }
        press(h, a);
    }
    assert_eq!(page(h), Some(want), "never reached {want:?}");
}

/// Boot and walk the real menus into a local Arena Deathmatch on 0x07000024 with one bot.
fn local_bot_match(iso: &str) -> HeadlessApp {
    let mut h = HeadlessApp::boot(iso);
    assert!(h.pump(60.0, |a| a.fe.as_ref().is_some_and(|fe| fe.page == Page::Start && !fe.intro_playing)), "title");
    h.key(KeyCode::Enter);
    assert!(h.pump(10.0, |a| a.fe.as_ref().is_some_and(|fe| fe.page == Page::EstHero)));
    h.frames(210);
    h.key(KeyCode::Enter);
    assert!(h.pump(10.0, |a| a.fe.as_ref().is_some_and(|fe| fe.page == Page::Main && !fe.intro_playing)), "main");
    press(&mut h, MA::Down);
    press_until(&mut h, MA::Confirm, Page::MpMode, 5);
    press_until(&mut h, MA::Confirm, Page::MpJoin, 5);
    press_until(&mut h, MA::Confirm, Page::MpScenario, 8);
    press_until(&mut h, MA::Confirm, Page::MpMap, 3);
    assert_eq!(h.app.fe.as_ref().unwrap().mp_scenario, 0x1, "Arena = Deathmatch");
    press_until(&mut h, MA::Confirm, Page::MpSetup, 3);
    assert_eq!(h.app.fe.as_ref().unwrap().mp_level, 0x0700_0024);
    press_until(&mut h, MA::Confirm, Page::MpOptions, 6);
    press(&mut h, MA::Down);
    press_until(&mut h, MA::Confirm, Page::MpBots, 2);
    while h.app.fe.as_ref().unwrap().mp_bots_row < 1 { press(&mut h, MA::Down); }
    press_until(&mut h, MA::Confirm, Page::MpBotChoose, 2);
    press_until(&mut h, MA::Confirm, Page::MpBotSetup, 2);
    // The bot setup rows by real Down/Right presses: Speed 0, Skill Poor (8), Hit points 50.
    for (row, want) in [(1usize, 0i32), (4, 8), (5, 50)] {
        while h.app.fe.as_ref().unwrap().cursor < row { press(&mut h, MA::Down); }
        for _ in 0..10 {
            let b = &h.app.fe.as_ref().unwrap().mp_bots[0];
            let cur = match row { 1 => b.speed, 4 => b.skill, _ => b.hit_points };
            if cur == want { break; }
            press(&mut h, MA::Right);
        }
    }
    assert_eq!(h.app.fe.as_ref().unwrap().mp_bots[0].hit_points, 50);
    press_until(&mut h, MA::Confirm, Page::MpBots, 2);
    assert_eq!(h.app.fe.as_ref().unwrap().mp_bot_count(), 1);
    while h.app.fe.as_ref().unwrap().mp_bots_row > 0 { press(&mut h, MA::Up); }
    press_until(&mut h, MA::Confirm, Page::MpOptions, 2);
    while h.app.fe.as_ref().unwrap().mp_options_row > 0 { press(&mut h, MA::Up); }
    press_until(&mut h, MA::Confirm, Page::MpConfirm, 2);
    h.app.menu_input(MA::Confirm);
    assert!(h.pump(120.0, |a| a.async_load.is_none() && a.pending_level.is_none() && a.scene.sim.as_ref().is_some_and(|s| s.mp.is_some())), "the match never loaded");
    {
        let sim = h.app.scene.sim.as_ref().unwrap();
        assert!(sim.net_rng.is_none() && h.app.net.is_none(), "a local match, no session");
        assert_eq!(sim.mp.as_ref().unwrap().bots.len(), 1);
    }
    h.app.grabbed = true;
    h
}

#[test]
fn local_match_player_rounds_hurt_and_kill_the_bot() {
    let Ok(iso) = std::env::var("NF_ISO") else { return };
    let mut h = local_bot_match(&iso);
    let local = h.app.scene.sim.as_ref().unwrap().mp_local_slot;
    let bot_slot = h.app.scene.sim.as_ref().unwrap().mp.as_ref().unwrap().bots[0].slot;
    let mut walking = false;
    let mut shots_on_body = 0u32; // ticks the trigger was down with the view ray on the bot's body, no wall nearer
    let mut first_hurt: Option<(u32, f32)> = None;
    let mut player_hurt = false;
    let mut bot_was_dead = false;
    let mut kill_seen: Option<(u32, i32, u32)> = None;
    let kills0 = h.app.scene.sim.as_ref().unwrap().mp.as_ref().unwrap().game.slots[local].kills;
    let hp0 = h.app.scene.sim.as_ref().unwrap().drones.health.health;
    for t in 0..6000u32 {
        h.frame(1.0 / 60.0);
        let sim = h.app.scene.sim.as_ref().unwrap();
        let mp = sim.mp.as_ref().unwrap();
        let bot = &mp.bots[0];
        if sim.drones.health.health < hp0 { player_hurt = true; }
        if !bot.dead && bot.health < 50.0 && first_hurt.is_none() {
            first_hurt = Some((t, bot.health));
            eprintln!("[bot-dmg] t{t} the bot is hurt: health {} after {shots_on_body} trigger ticks on its body", bot.health);
        }
        if bot.dead && !bot_was_dead {
            let kills = mp.game.slots[local].kills;
            eprintln!("[bot-dmg] t{t} the bot died; player kills {kills0} -> {kills}, bot deaths {}", bot.deaths);
            kill_seen = Some((t, kills, bot.deaths));
            break;
        }
        bot_was_dead = bot.dead;
        // Hunt the bot: real look deltas toward its body, fire when lined up.
        let bp = sim.bond.pos;
        let eye = sim.bond.head();
        let bpos = bot.brain.pos;
        let d = ((bpos[0] - bp[0]).powi(2) + (bpos[2] - bp[2]).powi(2)).sqrt();
        let want = (bpos[0] - bp[0]).atan2(bpos[2] - bp[2]);
        let mut err = want - sim.bond.yaw;
        while err > std::f32::consts::PI { err -= std::f32::consts::TAU; }
        while err < -std::f32::consts::PI { err += std::f32::consts::TAU; }
        let wantp = (bpos[1] - 0.3 - eye[1]).atan2(d);
        let perr = wantp - sim.bond.pitch * std::f32::consts::FRAC_PI_2;
        let lk = sim.bond.look();
        let far = [eye[0] + lk[0] * 50.0, eye[1] + lk[1] * 50.0, eye[2] + lk[2] * 50.0];
        let on_body = sim.mp_combat_bodies(local).iter().filter(|b| b.alive && b.slot == bot_slot)
            .find_map(|b| b.segment_hit(eye, far)).map(|t| t * 50.0);
        let wall = sim.world.segment(eye, far).map(|t| t * 50.0);
        let lined = on_body.is_some_and(|b| wall.is_none_or(|w| b < w));
        let fire = !bot.dead && err.abs() < 0.06 && t % 10 < 5; // semi-auto: press and release
        if fire && lined { shots_on_body += 1; }
        let zoom = sim.zoom;
        let walk = d > 6.0 && err.abs() < 0.5 && !bot.dead;
        if t % 240 == 0 { eprintln!("[bot-dmg] t{t} dist {d:.1} body {on_body:?} wall {wall:?} bot health {} gun w{:#x} clip {}", bot.health, sim.gun.weapon, sim.gun.clip); }
        mouse_look_add(&mut h.app.turn, &mut h.app.look, -(err.clamp(-0.15, 0.15) / MOUSE_SENSITIVITY) as f64, -(perr.clamp(-0.1, 0.1) / MOUSE_SENSITIVITY) as f64, zoom);
        h.app.trigger = fire;
        if walk != walking {
            let _ = h.app.app_key(KeyCode::KeyW, if walk { winit::event::ElementState::Pressed } else { winit::event::ElementState::Released }, false);
            walking = walk;
        }
    }
    eprintln!("[bot-dmg] trigger ticks on the body {shots_on_body}; first hurt {first_hurt:?}; kill {kill_seen:?}; the bot hurt the player: {player_hurt}");
    assert!(shots_on_body > 10, "the player never lined up on the bot ({shots_on_body} ticks)");
    assert!(first_hurt.is_some(), "the player's rounds never hurt the bot ({shots_on_body} trigger ticks on its body)");
    let (_, kills, deaths) = kill_seen.expect("the player never killed the bot");
    assert_eq!(deaths, 1);
    assert_eq!(kills, kills0 + 1, "the bot kill scores for the player");
}

/// One tick of the hunt: real look deltas toward the bot's body, W while far, the trigger pressed
/// and released while lined up. Returns whether the trigger was down with the view ray on the body.
fn hunt_tick(h: &mut HeadlessApp, t: u32, walking: &mut bool) -> bool {
    let sim = h.app.scene.sim.as_ref().unwrap();
    let local = sim.mp_local_slot;
    let bot = &sim.mp.as_ref().unwrap().bots[0];
    let bot_slot = bot.slot;
    let bp = sim.bond.pos;
    let eye = sim.bond.head();
    let bpos = bot.brain.pos;
    let d = ((bpos[0] - bp[0]).powi(2) + (bpos[2] - bp[2]).powi(2)).sqrt();
    let want = (bpos[0] - bp[0]).atan2(bpos[2] - bp[2]);
    let mut err = want - sim.bond.yaw;
    while err > std::f32::consts::PI { err -= std::f32::consts::TAU; }
    while err < -std::f32::consts::PI { err += std::f32::consts::TAU; }
    let wantp = (bpos[1] - 0.3 - eye[1]).atan2(d);
    let perr = wantp - sim.bond.pitch * std::f32::consts::FRAC_PI_2;
    let lk = sim.bond.look();
    let far = [eye[0] + lk[0] * 50.0, eye[1] + lk[1] * 50.0, eye[2] + lk[2] * 50.0];
    let on_body = sim.mp_combat_bodies(local).iter().filter(|b| b.alive && b.slot == bot_slot)
        .find_map(|b| b.segment_hit(eye, far)).map(|t| t * 50.0);
    let wall = sim.world.segment(eye, far).map(|t| t * 50.0);
    let lined = on_body.is_some_and(|b| wall.is_none_or(|w| b < w));
    let fire = !bot.dead && err.abs() < 0.06 && t % 10 < 5;
    let zoom = sim.zoom;
    let walk = d > 6.0 && err.abs() < 0.5 && !bot.dead;
    mouse_look_add(&mut h.app.turn, &mut h.app.look, -(err.clamp(-0.15, 0.15) / MOUSE_SENSITIVITY) as f64, -(perr.clamp(-0.1, 0.1) / MOUSE_SENSITIVITY) as f64, zoom);
    h.app.trigger = fire;
    if walk != *walking {
        let _ = h.app.app_key(KeyCode::KeyW, if walk { winit::event::ElementState::Pressed } else { winit::event::ElementState::Released }, false);
        *walking = walk;
    }
    fire && lined
}

/// mp-bot-hit-fx: the player's round on a bot shows the flesh puff at the contact
/// (`NDrone2_DealWithObjHit` 0x13fff0 case 5's `Gas_Init(3.0, 6.0, ..., 0x20008bc, 5, ...)`
/// [M :243-249], the bots being drones in the game) and plays the bot's pain line from the
/// 0x14d pool (`BOT_handlePain` 0x125ed8's trigger-0x1f arm, `0x14d + Rand(5)` [M :97-131]).
/// Before the fix the local match's hits reached the pain path only: no puff.
#[test]
fn local_match_player_round_on_the_bot_puffs_and_barks() {
    let Ok(iso) = std::env::var("NF_ISO") else { return };
    let mut h = local_bot_match(&iso);
    let bot_slot = h.app.scene.sim.as_ref().unwrap().mp.as_ref().unwrap().bots[0].slot;
    const BLOOD: u32 = 0x0200_08bc;
    let blood = |h: &HeadlessApp| h.app.scene.sim.as_ref().unwrap().drones.gas.puffs.iter().filter(|g| g.mesh == BLOOD).count();
    // The pain pool's lines, resolved through the loaded voice table (the bark's own lookup).
    let pain_sfx: Vec<u32> = {
        let sim = h.app.scene.sim.as_ref().unwrap();
        let v = sim.voices.as_deref().expect("the voice table loads");
        (0x14d..0x152).filter_map(|i| v.get(i).map(|l| l.sfx)).collect()
    };
    assert!(!pain_sfx.is_empty(), "the bot pain pool resolves");
    let mut walking = false;
    let mut lined_ticks = 0u32;
    let mut before = None;
    for t in 0..6000u32 {
        let hp = h.app.scene.sim.as_ref().unwrap().mp.as_ref().unwrap().bots[0].health;
        let puffs = blood(&h);
        let started = h.app.scene.sim.as_ref().unwrap().mp_fx.started.len();
        if hunt_tick(&mut h, t, &mut walking) { lined_ticks += 1; }
        h.frame(1.0 / 60.0);
        let sim = h.app.scene.sim.as_ref().unwrap();
        let bot = &sim.mp.as_ref().unwrap().bots[0];
        if !bot.dead && bot.health < hp {
            before = Some((t, hp, bot.health, puffs, started));
            break;
        }
    }
    let (t, hp0, hp1, puffs0, started0) = before.unwrap_or_else(|| panic!("the player's rounds never hurt the bot ({lined_ticks} lined trigger ticks)"));
    let sim = h.app.scene.sim.as_ref().unwrap();
    let bot = &sim.mp.as_ref().unwrap().bots[0];
    let body = bot.brain.pos;
    let new_blood: Vec<[f32; 3]> = sim.drones.gas.puffs.iter().filter(|g| g.mesh == BLOOD).map(|g| g.pos).collect();
    let near = new_blood.iter().filter(|p| ((p[0] - body[0]).powi(2) + (p[2] - body[2]).powi(2)).sqrt() < 1.5
        && p[1] < body[1] + 0.5 && p[1] > body[1] - 2.2).count();
    let barks: Vec<u32> = sim.mp_fx.started[started0.min(sim.mp_fx.started.len())..].iter()
        .filter(|e| e.3 == bot_slot).map(|e| e.1).collect();
    eprintln!("[bot-fx] t{t} hit: health {hp0} -> {hp1}; blood puffs {puffs0} -> {}, {near} at the bot; sounds at the bot slot {barks:?} (pain pool {pain_sfx:?})", new_blood.len());
    assert!(new_blood.len() > puffs0, "the hit spawned no flesh puff");
    assert!(near > 0, "the flesh puff is not at the bot's body");
    assert!(sim.drones.gas.puffs.iter().filter(|g| g.mesh == BLOOD).all(|g| g.billboard), "the blood puff faces the viewer (+0xF0 |= 8)");
    assert!(barks.iter().any(|id| pain_sfx.contains(id)), "the hit played no pain line at the bot");
    // Render aid: `NF_BOTFX_SHOT=<out.png>` saves what the window shows a few ticks after the hit
    // (the app's own capture, the headless `--shot` renderer).
    if let Some(out) = std::env::var_os("NF_BOTFX_SHOT") {
        h.app.trigger = false;
        let n: u32 = std::env::var("NF_BOTFX_SHOT_TICKS").ok().and_then(|v| v.parse().ok()).unwrap_or(4);
        for _ in 0..n { h.frame(1.0 / 60.0); }
        h.app.capture_png(std::path::Path::new(&out)).expect("capture");
        eprintln!("[bot-fx] saved {}", std::path::Path::new(&out).display());
    }
}
