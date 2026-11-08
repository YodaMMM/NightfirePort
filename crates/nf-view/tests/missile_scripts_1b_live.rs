//! 0700001b: the SpaceMissile and SpaceLaser scripts PLAY through the real script runtime.
//!
//! `Create_SpaceMissile` 0x1a9ef0 loads its launch script with `Script_Load(p0, pos, rot, …)`,
//! plays it once and runs one `Script_Update` at create [M 0x1a9ef0:46-73]; states 1-3 of
//! `SpaceMissile_Update` 0x1aa118 update it, free it, load the p1/p2 branch script and update
//! that [M disasm 0x1aa23c-0x1aa350]. `SpaceLaser_Update` 0x1aa3f8 does the same for its
//! tracking/dead scripts [M :87-126, :176]. These scripts are creator-owned (no ScriptPlayer
//! placement), and the port used to replace them with tick counters holding only their channel
//! writes, so their entity, light and sound streams never ran: no missile on any deck, no
//! launch sounds, no laser body. Each test below fails on that tree.
//!
//! Disc data [D 0700001b kind-7 NIS decode]: the p0 scripts' t0 entities (streams 0-4, 12, 13:
//! glists 0x20007e3..e6, 0x20007f9, 0x2000b00/b02; missile 4's 06000a30 carries none), the p0
//! sound 0x515 on stream 9 at t2; the p1 060007e1's flame 0x20007e1 (stream 9, obj type 0xE4,
//! parented to stream 1); the tracking pair 0x060007c0/c1's type-0x38 laser 0x20007ce (stream 4,
//! t0) and its beam 0x20007cc (stream 1, t120..t359), the sound 0x513 at t2.

#[path = "common/walker.rs"]
mod walker;

use nf_game::nis::Obj;
use walker::Walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn load() -> Walker {
    let mut w = Walker::new(nf_view::load(&iso(), "0700001b.bin", false).unwrap());
    w.wait_enabled(3000, "the shuttle arrival");
    w
}

/// The live script entities of `script` (the stream objects' glist hash and world position).
fn entities(w: &Walker, script: u32) -> Vec<(u32, [f32; 3])> {
    w.scene.sim.as_ref().unwrap().drones.scripts.objects.list.iter().flatten().filter_map(|o| match o {
        Obj::Entity { hash, matrix, src, .. } if src.0 == script => Some((*hash, [matrix[3][0], matrix[3][1], matrix[3][2]])),
        _ => None,
    }).collect()
}

fn sound_live(w: &Walker, script: u32, stream: u8) -> bool {
    w.scene.sim.as_ref().unwrap().drones.script_sounds.iter().any(|s| s.0 == (script, stream))
}

/// The missiles stand on the decks from the load: each p0 played its create-time update, so its
/// t0 entities exist, keyed from the missile's own placement base [M 0x1a9ef0:46-73]. Sister:
/// missile 4's 06000a30 has no entity stream [D: 2 streams], so it shows nothing.
#[test]
fn the_launch_scripts_stand_the_missiles_on_the_decks() {
    let w = load();
    // (p0 hash, placement translation) [D the kind-0x39 dump: #129/#131-137].
    let missiles: [(u32, [f32; 3]); 8] = [
        (0x0600092a, [12.691, -5.935, 11.382]), (0x0600092b, [12.691, -5.935, -11.382]),
        (0x06000929, [25.72, -8.236, 11.382]), (0x06000a30, [25.72, -8.236, -11.382]),
        (0x06000927, [38.741, -10.537, 11.382]), (0x0600092c, [38.741, -10.537, -11.382]),
        (0x0600092e, [51.779, -12.838, 11.382]), (0x0600092d, [51.779, -12.838, -11.382]),
    ];
    for (h, at) in missiles {
        let e = entities(&w, h);
        if h == 0x06000a30 {
            assert!(e.is_empty(), "sister: 06000a30 carries no entity stream [D]");
            continue;
        }
        let glists: Vec<u32> = e.iter().map(|x| x.0).collect();
        for g in [0x020007e4u32, 0x020007e5, 0x020007e6] {
            assert!(glists.contains(&g), "{h:#x}: the t0 entity {g:#x} is live [D], got {glists:x?}");
        }
        // 0x20007e4 keys at the stream origin, so it stands exactly on the placement base
        // [M 0x1a9ef0:46 — Script_Load(p0, obj+0x30, obj+0x50)].
        let at0 = e.iter().find(|x| x.0 == 0x020007e4).unwrap().1;
        let d = (0..3).map(|k| (at0[k] - at[k]).abs()).fold(0.0f32, f32::max);
        assert!(d < 1e-3, "{h:#x}: 0x20007e4 at {at0:?} stands on its deck {at:?}");
    }
}

/// The launch order runs the p0: missile 5 (seed 0 [D]) leaves state 0 on the first gated tick,
/// its p0 06000927 plays the 0x515 sound at t2 [D stream 9] and its in-stream channel writes land
/// (ch 185 @t10, ch 35 @t359 [D stream 5]). Sister: before the order no missile sound plays.
#[test]
fn the_launch_order_plays_the_missile_sounds_and_writes() {
    let mut w = load();
    assert!(!sound_live(&w, 0x06000927, 9), "sister: no launch sound before the order");
    let mut first_sound = None;
    let mut ch185 = None;
    let mut ch35 = None;
    for t in 0..400u32 {
        {
            // The launch order ch 19 is the AND(193,194) mux's out [D PL#109] — held through its
            // inputs (the interactables test's drive).
            let sim = w.scene.sim.as_mut().unwrap();
            sim.drones.channels.set(193, 1, sim.drones.frame);
            sim.drones.channels.set(194, 1, sim.drones.frame);
        }
        w.tick(Default::default(), false);
        let sim = w.scene.sim.as_ref().unwrap();
        if first_sound.is_none() && sound_live(&w, 0x06000927, 9) { first_sound = Some(t); }
        if ch185.is_none() && sim.drones.channels.on(185) { ch185 = Some(t); }
        if ch35.is_none() && sim.drones.channels.on(35) { ch35 = Some(t); }
    }
    let s = first_sound.expect("missile 5's p0 plays its 0x515 sound [D 06000927 stream 9]");
    let (a, b) = (ch185.expect("ch 185 @t10 [D]"), ch35.expect("ch 35 @t359 [D]"));
    assert_eq!(a - s, 8, "the sound at t2, ch 185 at t10 on the same stream clock");
    assert_eq!(b - s, 357, "ch 35 at t359");
    let m5 = w.scene.sim.as_ref().unwrap().drones.space_missiles.iter().find(|m| m.index == 5).unwrap().state;
    assert_eq!(m5, 1, "missile 5 is tracking its p0 [M 0x1aa23c]");
}

/// The launch branch plays the p1: missile 4 (seed 20, console 4 unshot) takes the launch branch
/// at launch+~3000 and its 060007e1 creates the flame 0x20007e1 (obj type 0xE4) on its first
/// update. `Script_CreateEntity` 0x1d8968 has no 0xE4 arm (its type compares are 0xea/0x22/0x20/
/// 0x29/0xe1/0xf0/0xeb/0xf2/0xf5 [M disasm 0x1d8990-0x1d8a18]), so the flame is a plain class-0
/// object: `Player_DealWithObjHit` 0x194458 hurts only through class 0x27 — no hurt volume
/// joins the level with it.
#[test]
fn the_launch_branch_plays_the_flame_without_a_hurt_volume() {
    let mut w = load();
    let hurts = w.scene.sim.as_ref().unwrap().hurts.len();
    let mut launched = false;
    for _ in 0..3300u32 {
        {
            let sim = w.scene.sim.as_mut().unwrap();
            sim.drones.channels.set(193, 1, sim.drones.frame);
            sim.drones.channels.set(194, 1, sim.drones.frame);
        }
        w.tick(Default::default(), false);
        let m4 = w.scene.sim.as_ref().unwrap().drones.space_missiles.iter().find(|m| m.index == 4).unwrap().clone();
        if m4.state == 3 && m4.launch_flag == 1 { launched = true; break; }
    }
    assert!(launched, "missile 4 takes the launch branch [M 0x1aa31c]");
    let e = entities(&w, 0x060007e1);
    assert!(e.iter().any(|x| x.0 == 0x020007e1), "the p1's flame entity is live [D 060007e1 stream 9], got {e:x?}");
    assert!(e.iter().any(|x| x.0 == 0x020007e5), "the p1's missile body 0x20007e5 [D stream 1]");
    assert!(sound_live(&w, 0x060007e1, 11) || {
        // The 0x466 line starts at t2 — two more updates.
        w.wait(3);
        sound_live(&w, 0x060007e1, 11)
    }, "the p1's 0x466 launch sound [D stream 11 t2]");
    assert_eq!(w.scene.sim.as_ref().unwrap().hurts.len(), hurts, "the flame is no hurt volume [M 0x1d8990-0x1d8a18]");
}

/// The laser's tracking script plays: the type-0x38 laser body 0x20007ce from t0, the beam
/// 0x20007cc from t120, the 0x513 charge sound at t2 [D 0x060007c0/c1].
#[test]
fn the_laser_tracking_script_plays_its_body_beam_and_sound() {
    let mut w = load();
    {
        let sim = w.scene.sim.as_mut().unwrap();
        for m in sim.drones.space_missiles.iter_mut() { m.state = 4; } // the missiles' inert epilogue [M 0x1aa3e0]
    }
    let both = |w: &Walker| [0x060007c0u32, 0x060007c1].map(|h| entities(w, h)).concat();
    assert!(both(&w).is_empty(), "sister: nothing of the tracking pair before the first shot");
    let (mut body, mut beam, mut sound) = (None, None, None);
    for t in 0..1200u32 {
        {
            let sim = w.scene.sim.as_mut().unwrap();
            sim.drones.channels.set(19, 1, sim.drones.frame); // the arm gate [D p7]
        }
        w.tick(Default::default(), false);
        let e = both(&w);
        if body.is_none() {
            if let Some(x) = e.iter().find(|x| x.0 == 0x020007ce) {
                body = Some(t);
                let o = w.scene.sim.as_ref().unwrap().drones.space_lasers[0].fire_origin;
                assert_eq!(o, x.1, "the fire origin is the registered laser body [M SpaceLaser_Register 0x1aa9e8]");
            }
        }
        if beam.is_none() && e.iter().any(|x| x.0 == 0x020007cc) { beam = Some(t); }
        if sound.is_none() && (sound_live(&w, 0x060007c0, 12) || sound_live(&w, 0x060007c1, 13)) { sound = Some(t); }
        if beam.is_some() && sound.is_some() { break; }
    }
    let b = body.expect("the laser body 0x20007ce [D stream 4 t0]");
    assert_eq!(sound.expect("the 0x513 charge sound [D t2]") - b, 2, "the sound at t2");
    assert_eq!(beam.expect("the beam 0x20007cc [D stream 1 t120]") - b, 120, "the beam at t120");
}
