//! copter-hits: the 07000012 office helicopter (#707, gate 0x25, flight script 0x060005BD [D the
//! placements dump]) — how many rounds take it down, and how its rotor loop is mixed.
//!
//! Damage [M]: `Copter_Update` 0x17dbb8:277-320 walks the 9 part slots, scaling each slot's
//! HITDATA chain by the Bits table (ACTION.ELF .data 0x2a37e8: 2, 3, 1, 2, 100, 1, 0, 0, 0) into
//! rec+0x64; the death is `hp < rec+0x64` [:358], hp = DiffMod · 299 [Copter_Create 0x17d340:94].
//! The slots are filled by the SP create-callback `Copter_Register` 0x17e530 with the script's
//! event-0xF parameter as the slot (`Script_EventHandler` 0x1d94c0:172-175), and every part but
//! slot 2 gets obj+0xF0 bit 0x20, which `Collide_PickObj` 0x1eae80:58-82 always skips. On
//! 07000012 the script registers stream 0 (Helicopter-Bodya 0x0200062A) as slot 2 and streams
//! 11/13/16/18 as slots 5-8 [D the script dump]; the main rotor (stream 4, 0x0200062B, a 7.1-unit
//! sphere) is no part. The port used to take the stream index as the slot, which put the rotor in
//! slot 4 (×100): one P2K round (contact damage 3.5 [D weapon record 2 +0x0C]) did 350 > 299.
//!
//! Sound [M]: the rotor loop is sfx 0x10B (`Script_SoundStart` 0x1d8820 → `Sound_Play3D(100, -1,
//! -1, id, obj+0xC0, …, custom 0)`, so the bank's radii: tracking 2, inner 10, outer 100, sub
//! volume 95 [D SB 0x10B]); `PS2_SFXCalculate3D` (SFX.IRX 0x1060c) splits it into the front pair
//! and the rear voice (`rear = (dot(−dir, u) + 1000)·att/2000`, `front = 100 − rear`, rear voice =
//! rear·75/100) and `psiSetSampleVolumeRear` 0xf018 writes the rear right as `0x7FFF − v`
//! (phase-inverted) in stereo mode 1 (`bootup` 0x1c7e78:85 `SFXSetMode(1)`).
//!
//! Opt-in: set `NF_ISO`.

use nf_game::player::Input;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

const GATE: u32 = 0x25;
const BODY: u32 = 0x0200_062a;
const ROTOR: u32 = 0x0200_062b;

fn round(weapon: u16, from: [f32; 3], at: [f32; 3]) -> nf_game::drone::PlayerBullet {
    let d = [at[0] - from[0], at[1] - from[1], at[2] - from[2]];
    let l = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt().max(1e-6);
    nf_game::drone::PlayerBullet {
        pos: from, dir: [d[0] / l, d[1] / l, d[2] / l],
        speed: 13.2, travelled: 0.0, range: 50.0, damage: 0.0, weapon,
        first: true, dead: false, bounces: 0, homing: false, target: None,
        // homing-copter-3: the copter-target flag (homing-copter-2 added the field; this
        // literal predates it) — a plain round carries no copter target.
        target_copter: false,
        gravity: false, guided_cam: false, snd: 0, swoosh: None, born: 0,
    }
}

fn step(scene: &mut nf_view::scene::Scene, n: usize) {
    for _ in 0..n { scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false); }
}

/// The 12 copter with its gate open and its script running (the registrations made).
fn copter_scene(iso: &str) -> nf_view::scene::Scene {
    let mut scene = nf_view::load(iso, "07000012.bin", false).expect("load 07000012");
    {
        let sim = scene.sim.as_mut().unwrap();
        assert_eq!(sim.drones.copters.len(), 1, "one copter object on 07000012 [D placement 707, param 0 = 0]");
        assert_eq!(sim.drones.copters[0].gate as u32, GATE);
        sim.drones.channels.set(GATE, 1, sim.drones.frame);
    }
    step(&mut scene, 30);
    scene
}

fn sphere_of(scene: &nf_view::scene::Scene, slot: usize) -> Option<([f32; 3], f32)> {
    let sim = scene.sim.as_ref().unwrap();
    let c = &sim.drones.copters[0];
    sim.drones.scripts.copter_part_targets(c.placement, &c.parts, &sim.data()).0[slot]
}

/// Fires `n` rounds of `weapon` through the body sphere, one per tick, then lets the walk
/// consume them.
fn fire_at_body(scene: &mut nf_view::scene::Scene, weapon: u16, n: u32) {
    for _ in 0..n {
        let (c, _) = sphere_of(scene, 2).expect("the body part is live");
        let from = [c[0], c[1] + 1.0, c[2]];
        scene.sim.as_mut().unwrap().drones.player_bullets.push(round(weapon, from, c));
        step(scene, 1);
    }
    step(scene, 3);
}

/// The registration table and the P2K / SG5 rounds-to-kill: the body takes the rounds at
/// ×1.0, so d1's 299 hp needs 86 P2K rounds (3.5) and 109 SG5 rounds (2.75); one round fewer
/// leaves it flying, the last one sends it to state 6 (the explode level's death).
#[test]
fn the_12_copter_takes_86_p2k_or_109_rifle_rounds() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    unsafe { std::env::set_var("NF_SILENT", "1"); }
    let scene = copter_scene(&iso);
    {
        let sim = scene.sim.as_ref().unwrap();
        let c = &sim.drones.copters[0];
        let mask: u16 = c.parts.iter().enumerate().filter(|(_, p)| p.is_some()).map(|(i, _)| 1u16 << i).sum();
        assert_eq!(mask, 0b1_1110_0100, "slots 2, 5, 6, 7, 8 registered [D 0x060005BD's event-0xF params], got {mask:#b}");
        assert_eq!(c.parts[2].map(|p| p.1), Some(BODY), "slot 2 = Helicopter-Bodya");
        assert!(c.parts.iter().all(|p| p.is_none_or(|p| p.1 != ROTOR)), "the rotor is no part");
        assert_eq!(sim.data().weapons.bullet_contact_damage(2), 3.5, "P2K contact damage [D record 2 +0x0C]");
        assert_eq!(sim.data().weapons.bullet_contact_damage(0x16), 2.75, "SG5 contact damage [D record 0x16 +0x0C]");
        assert_eq!(c.hp, 299.0, "the default difficulty's hp [M 0x17d340:94, DiffMod 1]");
    }
    for (weapon, dmg, n) in [(2u16, 3.5f32, 86u32), (0x16, 2.75, 109)] {
        let mut scene = copter_scene(&iso);
        assert_eq!(nf_game::copter::rounds_to_kill(299.0, dmg), n);
        fire_at_body(&mut scene, weapon, n - 1);
        {
            let c = &scene.sim.as_ref().unwrap().drones.copters[0];
            eprintln!("[copter-hits] weapon {weapon:#x}: {} rounds → damage {} state {}", n - 1, c.damage, c.state);
            assert_eq!(c.damage, (n - 1) as f32 * dmg, "every round lands once on the body at ×1.0");
            assert!(c.damage <= c.hp && c.state == 1, "{} rounds leave it flying (damage {} hp {})", n - 1, c.damage, c.hp);
        }
        // Sister: the next round is past hp → state 6 (07000012 is an explode level [M :360-363]).
        fire_at_body(&mut scene, weapon, 1);
        let sim = scene.sim.as_ref().unwrap();
        let gone = sim.drones.copters.is_empty();
        assert!(gone || sim.drones.copters[0].state == 6, "round {n} kills it (state {:?})", sim.drones.copters.first().map(|c| c.state));
    }
}

/// A round that passes outside the body sphere scores nothing, even through the rotor's
/// stream-index slot the old mapping counted ×100 (the rotor sits inside the 7.4-unit body
/// sphere, so the miss runs beside the whole craft). Sister: the same round through the body
/// adds exactly its 3.5 once — the round dies on the body.
#[test]
fn a_miss_scores_nothing_and_a_body_round_counts_once() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    unsafe { std::env::set_var("NF_SILENT", "1"); }
    let mut scene = copter_scene(&iso);
    let (bc, br) = sphere_of(&scene, 2).expect("the body part");
    // A vertical ray 1.5 units outside the body sphere's rim.
    let x = [bc[0] + br + 1.5, bc[1] + 3.0, bc[2]];
    scene.sim.as_mut().unwrap().drones.player_bullets.push(round(2, x, [x[0], x[1] - 6.0, x[2]]));
    step(&mut scene, 4);
    assert_eq!(scene.sim.as_ref().unwrap().drones.copters[0].damage, 0.0, "a miss adds nothing");
    fire_at_body(&mut scene, 2, 1);
    assert_eq!(scene.sim.as_ref().unwrap().drones.copters[0].damage, 3.5, "sister: one body round adds its 3.5 once");
}

/// The rotor loop's 3-D mix on the real bank: at 20 units in front the front pair carries
/// it (left = right, no rear); at 20 units behind the front pair drops to 9 and the rear
/// voice takes 57 (×vol) with its right phase-inverted. Sister: at 50 units the front level
/// falls to 14 (the quadratic falloff), and at the same 50 behind the rear voice is 20.
/// (The direction is normalised by `PS2_SquareRoot`'s overshoot: √40000 → 201, √250000 → 518,
/// so `dot(−dir, u)` is 995 / 965, not 1000.)
#[test]
fn the_rotor_loop_rear_voice_behind_vs_the_front_pair_in_front() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    unsafe { std::env::set_var("NF_SILENT", "1"); }
    let mut scene = nf_view::load(&iso, "07000012.bin", false).expect("load 07000012");
    let (slot, bank, sbf) = scene.sound_bank.take().expect("the level bank");
    {
        let fx = nf_formats::soundbank::Effect::parse(bank.effect(0x10b).expect("0x10B in the bank")).unwrap();
        assert_eq!((fx.tracking, fx.inner, fx.outer, fx.subs[0].volume), (2, 10, 100, 95), "[D SB 0x10B]");
    }
    let levels = |z: f32| -> ((f32, f32), (f32, f32)) {
        let mut d = nf_audio::sfx::Driver::new();
        d.load_bank(slot, bank.clone(), sbf.clone());
        // Facing +z; +x is the screen-left axis (up × forward).
        d.set_listener([0.0; 3], [0.0, 0.0, 1.0], [1.0, 0.0, 0.0]);
        let mut out = Vec::new();
        assert_eq!(d.start(&mut out, 0x10b, Some([0.0, 0.0, z]), 1, None), 0, "0x10B starts at z {z}");
        // The key-on lands with the first `SFXUpdate` [M sound.md §3.2].
        if out.is_empty() { d.update(&mut out, &[]); }
        let plays: Vec<(f32, f32)> = out.iter().filter_map(|c| match c { nf_audio::Cmd::Play { left, right, .. } => Some((*left, *right)), _ => None }).collect();
        assert_eq!(plays.len(), 2, "a 3-D voice pair (front, rear): {out:?}");
        (plays[0], plays[1])
    };
    let pct = |x: f32| (x * 100.0).round() as i32;
    // The combined volume: sub volume 95 → `(v·v)/100` = 90 [M ES_CombineVolumes 0x1300].
    let v = |lvl: i32| (lvl * 90) / 100;
    let ((fl, fr), (bl, _)) = levels(20.0);
    eprintln!("[copter-snd] front 20: front ({fl}, {fr}) rear {bl}");
    assert_eq!((pct(fl), pct(fr), pct(bl)), (v(38), v(38), 0), "in front: the pair at 38, no rear");
    let ((fl2, fr2), (bl2, br2)) = levels(-20.0);
    eprintln!("[copter-snd] behind 20: front ({fl2}, {fr2}) rear ({bl2}, {br2})");
    assert_eq!((pct(fl2), pct(fr2)), (v(9), v(9)), "behind: rear = 1995·77/2000 = 76, the pair 38·24/100 = 9");
    assert_eq!(pct(bl2), v(57), "behind: the rear voice at 76·75/100 = 57");
    assert_eq!(br2, -bl2, "the rear right is phase-inverted (0x7FFF − v) [M 0xf018]");
    assert!(pct(bl2) > pct(fl), "behind, the rear voice is LOUDER than the front pair in front (the IOP's split)");
    let ((fl3, _), _) = levels(50.0);
    let (_, (bl4, _)) = levels(-50.0);
    assert_eq!((pct(fl3), pct(bl4)), (v(14), v(20)), "the 50-unit falloff: att 28 → front 14 / rear 1965·28/2000·75/100 = 20");
}
