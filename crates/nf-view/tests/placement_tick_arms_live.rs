//! The delete and latch arms of three small `control_funcs` classes, on the real levels: each is
//! a tick the port had, missing the part where the object removes itself or stays latched.
//! - class 0x44 `MusicTrigger_Update` 0x1b31a0: p3 = 1 deletes the object after one post.
//! - class 0x43 `Hint_Update` 0x1b3380: a set off-channel deletes the marker unfired.
//! - class 0x42 `ThirdIcon_Update` 0x196e80 (the collide slot): `obj+0xF4` latches.
//! The channels are driven by hand: these tests pin what each object does with a channel, not
//! which script raises it. Opt-in via NF_ISO.

#[path = "common/walker_b2.rs"]
mod walker;

use walker::Walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn load(level: &str) -> Walker {
    let mut w = Walker::new(nf_view::load(&iso(), level, false).expect("load"));
    w.wait_enabled(3000, "the intro");
    w.wait(60);
    w
}

fn pulse(w: &mut Walker, ch: u32) {
    w.scene.sim.as_mut().unwrap().drones.channels.set(ch, 1, 0);
    w.wait(2);
    w.scene.sim.as_mut().unwrap().drones.channels.set(ch, 0, 0);
    w.wait(2);
}

fn teleport(w: &mut Walker, pos: [f32; 3]) {
    let sim = w.scene.sim.as_mut().unwrap();
    sim.bond.pos = pos;
    sim.bond.vel = [0.0; 3];
    w.wait(40);
}

/// 07000001 #313, params (13, 0, 2, 1) [D]: with the channel held the alarm slot posts once
/// [M 0x1b31a0:26-27, 0x1336f8:86-87].
#[test]
fn the_07000001_music_trigger_posts_once() {
    let mut w = load("07000001.bin");
    assert!(w.scene.sim.as_ref().unwrap().drones.worldobjs.music_triggers.iter().any(|t| t.ch_on == 13 && t.event_id == 2),
        "sister: the object exists until its channel fires [D #313]");
    // The level's own object list, run alone: the guards write the same slot (5 while attacking
    // [M the SeenAndAttacking arm]), so the live board cannot count this object's posts.
    let mut objects = std::mem::take(&mut w.scene.sim.as_mut().unwrap().drones.worldobjs);
    let mut ch = nf_game::triggers::Channels::default();
    ch.set(13, 1, 0);
    let (mut music, mut rng) = (nf_game::music::Music::default(), nf_game::drone::Rng::new(1));
    let (weapons, mut bullets) = (nf_formats::weapon::WeaponData::default(), Vec::new());
    let mut posts = 0;
    for f in 0..120 {
        objects.tick(&mut ch, [0.0; 3], &weapons, &mut rng, &mut bullets, f, &mut music);
        if music.events[2] == 2 { posts += 1; }
        music.update(0);
    }
    assert_eq!(posts, 1, "one post with the channel held 120 ticks");
    assert!(objects.music_triggers.iter().all(|t| t.ch_on != 13), "the one-shot object is gone");
}

/// 07000014's sabotage marker, label 0x100020E, off-channel 0x8C [D params]: once the channel has
/// been seen set the marker is gone, so walking in after it drops posts nothing [M 0x1b3380:15,
/// 42-44]. The fresh-load sister (the marker fires) is the_sabotage_hints_gate_on_the_8c_channel.
#[test]
fn a_07000014_hint_deleted_by_its_off_channel_stays_gone() {
    let mut w = load("07000014.bin");
    pulse(&mut w, 0x8c);
    teleport(&mut w, [-89.29, 1.0, 32.6]);
    let sim = w.scene.sim.as_ref().unwrap();
    assert!(!sim.hud.info_msgs.iter().any(|m| m.label == 0x100_020e), "the deleted marker posts nothing");
}

/// 07000012 #680, a type-2 icon on channel 0x3C [D params 0:2 1:0x3c]: dead before the channel,
/// and still armed after the channel drops [M 0x196e80:18-20, the only write of obj+0xF4 past
/// the create].
#[test]
fn the_07000012_icon_latches_on_its_channel() {
    let mut w = load("07000012.bin");
    let c = {
        let sim = w.scene.sim.as_ref().unwrap();
        let i = sim.icons.iter().find(|i| i.on_channel == 0x3c).expect("the channel icon [D #680]");
        assert!(!i.enabled, "sister: off at load");
        assert!(!sim.drones.channels.on(0x3c), "sister: the channel is off at load");
        i.centre
    };
    teleport(&mut w, [c[0], c[1] + 0.5, c[2]]);
    assert_ne!(w.scene.sim.as_ref().unwrap().bond.icon_ctx, 2, "sister: no context before the channel");
    pulse(&mut w, 0x3c);
    w.wait(2);
    let sim = w.scene.sim.as_ref().unwrap();
    assert!(!sim.drones.channels.on(0x3c));
    assert_eq!(sim.bond.icon_ctx, 2, "the latched icon writes its context with the channel off");
}

/// 07000041's type-2 icon carries the only kill channel on the disc, 0x6E [D params 1:1 2:0x6e]:
/// the icon writes on the tick the channel reads set, then is gone [M 0x196e80:21-26].
#[test]
fn the_07000041_icon_is_deleted_by_its_kill_channel() {
    let scene = nf_view::load(&iso(), "07000041.bin", false).expect("load");
    let mut w = Walker::new(scene);
    w.wait(2);
    let ii = w.scene.sim.as_ref().unwrap().icons.iter().position(|i| i.kill_channel == 0x6e).expect("the icon [D #105]");
    pulse(&mut w, 0x6e);
    let sim = w.scene.sim.as_mut().unwrap();
    assert!(sim.icons[ii].dead, "the kill channel removes the icon");
    assert!(!sim.icons[ii].gate(&sim.drones.channels), "a removed icon stays out even with the channel off");
}
