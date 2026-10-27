//! level-4a-ride-cutscene: the −52 landing cut-scene (SP #289, pack 0x060006cf) carries its objects.
//!
//! The ride cut-scene (ticks ~1611-2186 of the lift ride) shows the landed charge hand-off: a guard
//! holds the demo charge, Bond lands on the lift roof. In the port the scene played with pieces
//! missing — the satchel and the lift floor. This file pins what the pack's streams do [D] and what
//! the port must carry.
//!
//! The pack's object ops ([D 0780004a.bin 0x060006cf], 575 ticks, 28 streams, camera true):
//! - s1/s2/s3: op 0xA (`Script_AnimStart` 0x1d8f40) — the three cut-scene characters: skins
//!   0x0500006 (script 0x060006d2), 0x0500007 (0x060006d1), 0x0500004 (0x060006d0 — the level's
//!   guard skin [D 0700004a.bin §7 the garrison's oDrone skin]).
//! - s7: op 7 (`Script_EntityStart` 0x1d8d78) — glist 0x02000758 "OBJECT_DemoCharge" (kind 0x31,
//!   the plain-object create arm [M `Script_CreateEntity` 0x1d8968's fallthrough:
//!   `control_create_object` + `hashtable_set_object_to_entity_gfx`]), keys 0..461 — the satchel
//!   prop's keyed path. The map entry lives on 0700004a.bin map 64 [D gfx census].
//! - s10: glist 0x020006e1 "NIS_LIFTHATCH" (kind 0x31) [D s10].
//! - s12: glist 0x020006b1 "NIS_P99" (kind 0x31) [D s12].
//! - s13: glist 0x020006ee "NIS_lift" (kind 0x31) — the lift car/floor the scene shows under Bond
//!   [D s13; 0700004a.bin map 62].
//! - s4 t0: event 0xB param 0x18 [M `Script_EventHandler` 0x1d94c0 case 0xb — writes
//!   `sAnimObject+0x62 = 0x18` on the PREVIOUS stream's object (the handler's record base +0x80 is
//!   one 0x28 record below `Script_Run`'s +0xa8)] — the guard anim (s3, skin 0x0500004) carries
//!   weapon_data[0x18]'s third-person mesh on its datum bone (`AnimObjectDraw` 0x1c4490:256-265)
//!   — the satchel in the guard's hand.
//! - s26 t575: event 6 (PlayerEnable at the previous stream's object — s25's hash-0 kind-0x19
//!   marker, the walkreal-pinned landing spot) [M 0x1d94c0 case 6].
//! - s27 t575: event 0xD — switch channel 0x22 = 1 (the hand-back mark) [D s27].

#[path = "common/walker.rs"]
mod walker;

use nf_game::player::Input;
use walker::Walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
fn the_landing_cutscene_carries_the_satchel_the_guard_anim_and_the_lift_floor() {
    let mut w = Walker::new(nf_view::load(&iso(), "0700004a.bin", false).unwrap());
    // Board the lift (the walkreal_0700004a boarding walk — the reveal plate under the first steps).
    for k in 0..84 {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        let target = if k < 42 { [3.2, 0.0, -2.2] } else { [3.2, 0.0, -3.8] };
        let desired = (target[0] - p[0]).atan2(target[2] - p[2]);
        let err = walker::wrap_pi(desired - sim.bond.yaw);
        w.tick(nf_game::player::Input { turn: err.clamp(-0.08, 0.08), forward: 1.0, ..Default::default() }, false);
    }
    assert!(w.scene.sim.as_ref().unwrap().drones.channels.on(0x4d), "the reveal plate fired [D #277]");

    // The ride: no input; the landing cut-scene disables Bond (event 4 [D 0x060006cf s9 t0]).
    w.skip_cutscenes = false;
    let mut scene_ticks = 0u32;
    let mut satchel_at: Option<(u32, [f32; 3])> = None;
    let mut lift_at: Option<(u32, [f32; 3])> = None;
    let mut guard_weapon: Option<(u32, u8)> = None;
    let mut guard_anim: Option<(u32, u32, f32)> = None;
    let mut bond_y_before = 0.0f32;
    let mut bond_y_after = 0.0f32;
    let mut poses: Vec<(u8, u32, bool)> = Vec::new();
    let mut satchel_mid: Option<[f32; 3]> = None;
    let mut lift_mid: Option<[f32; 3]> = None;
    let mut guard_mid: Option<[f32; 3]> = None;
    let mut hatch_mid: Option<[f32; 3]> = None;
    let mut guard_drawn_y: Option<f32> = None;
    let mut guard_drawn: Option<[f32; 3]> = None;
    let mut hand_world: Option<[f32; 3]> = None;
    let mut cam_mid: Option<[f32; 3]> = None;
    for k in 0..6800u32 {
        w.tick(Input::default(), false);
        let sim = w.scene.sim.as_ref().unwrap();
        if sim.disabled {
            if scene_ticks == 0 { bond_y_before = sim.bond.pos[1]; }
            scene_ticks += 1;
            // The scene's objects (src = (0x060006cf, stream) [D the stream table]).
            if let Some(m) = sim.script_entity((0x0600_06cf, 7), 0x0200_0758) {
                if satchel_at.is_none() { satchel_at = Some((scene_ticks, [m[3][0], m[3][1], m[3][2]])); }
            }
            if let Some(m) = sim.script_entity((0x0600_06cf, 13), 0x0200_06ee) {
                if lift_at.is_none() { lift_at = Some((scene_ticks, [m[3][0], m[3][1], m[3][2]])); }
            }
            let objs = &sim.drones.scripts.objects.list;
            for o in objs.iter().flatten() {
                if let nf_game::nis::Obj::Anim { skin: 0x0500_0004, weapon, src: (h, 3), .. } = o {
                    if *h == 0x0600_06cf && guard_weapon.is_none() { guard_weapon = Some((scene_ticks, *weapon)); }
                }
            }
            if guard_anim.is_none() {
                if let Some((script, frame)) = sim.script_anim((0x0600_06cf, 3), 0x0500_0004) {
                    guard_anim = Some((scene_ticks, script, frame));
                }
            }
            // The character POSE at mid-scene (once): a None pose collapses the guard — the satchel
            // on his hand vanishes with him.
            if scene_ticks == 300 {
                for c in w.scene.characters.iter() {
                    if let Some((h, si)) = c.script {
                        if h != 0x0600_06cf { continue; }
                        let root = sim.script_anim((h, si), c.skin.hashcode)
                            .and_then(|(script, frame)| nf_game::anim::script_pose_together(&sim.scripts, &sim.seqs, &c.skeleton, &c.skin, script, frame))
                            .map(|pose| pose.first().map(|(t, _)| *t).unwrap_or([0.0; 3]));
                        let ok = root.is_some();
                        poses.push((si, c.skin.hashcode, ok));
                        if let Some(t) = root { eprintln!("char s{si} root at ({:.2},{:.2},{:.2})", t[0], t[1], t[2]); }
                        if si == 3 {
                            // The anim script's own windows and the pack sequences' frame-0 roots.
                            let sc = sim.scripts.get(&0x0600_06d0u32);
                            if let Some(sc) = sc {
                                for (a, _b, q) in sc.sequences() {
                                    if let Some(seq) = sim.seqs.get(&q) {
                                        let v = |ch: usize, fr: u32| seq.value(ch, fr);
                                        eprintln!("  s3 win@{a} seq {q:08x} flags {:x} chans {} | ch0 f1 {:.2} f100 {:.2} | ch1 f1 {:.2} f100 {:.2} | ch2 f1 {:.2} | ch219 {:.2} ch220 {:.2} ch221 {:.2}",
                                            seq.flags, seq.channels.len(), v(0,1), v(0,100), v(1,1), v(1,100), v(2,1), v(219,1), v(220,1), v(221,1));
                                    } else { eprintln!("  s3 window@{a} seq {:08x} MISSING", q); }
                                }
                            }
                        }
                    }
                }
                // ride-cutscene-base: where the guard DRAWS — the posed object matrix applied to his world
                // root. The pack keys its whole scene car-local [D 0780004a decode]: the guard root
                // (2.35,0.85,-0.64) sits on the pack's own lift (s13 keys (0,0.99,0)), the charge (s7
                // (3.43,1.49,-0.84)) is the held-up prop — and the pack's entities compose the SP base
                // (0,-53.53,-3.38) [D placement] (this file pins the satchel/lift there). The guard draws
                // at the same compose: base ∘ root ≈ (2.46,-52.97,-4.27) mid-scene — on the car deck
                // (the lift entity composed ≈ (0,-55.8,-3.4), its deck ≈ -52.7); PCSX2 0700004a ride
                // frame 700: the red-beret guard holds the charge up in the car's hatch, the officer
                // beside him [L 2026-10-26]. The compose site is // [S] unverified (see scene.rs
                // pose_one) — the position itself is [D root ∘ base].
                let fs = w.scene.frame_state_for(None, 0.0, nf_view::scene::ViewOpts::default(), false);
                for (ci, c) in w.scene.characters.iter().enumerate() {
                    if c.script != Some((0x0600_06cf, 3)) { continue; }
                    if let Some(Some((_, world, m))) = fs.posed.get(ci) {
                        let t = [world[0][3][0], world[0][3][1], world[0][3][2]];
                        let p = |k: usize| m[0][k] * t[0] + m[1][k] * t[1] + m[2][k] * t[2] + m[3][k];
                        let wy = p(1);
                        eprintln!("guard drawn root ({:.2},{:.2},{:.2}) (object base y {:.2})", p(0), wy, p(2), m[3][1]);
                        guard_drawn_y = Some(wy);
                        guard_drawn = Some([p(0), wy, p(2)]);
                        // The held weapon (event 0xB → weapon byte 0x18 [M 0x1d94c0 case 0xb; D s4]) draws
                        // on the weapon datum's bone [M AnimObjectDraw 0x1c4490:256-265 + AnimDatumSetEntity
                        // 0x1c7848: the skin+0x20 datum row's quat/vec ∘ the bone's world palette]. Its draw
                        // slot: the attachment this scene build created for the guard character.
                        for (ai, a) in w.scene.attachments.iter().enumerate() {
                            if a.character as usize == ci && a.bone != 0 && fs.attach.get(ai).is_some() {
                                let aw = fs.attach[ai];
                                eprintln!("guard attachment a{ai} (bone {}) world ({:.2},{:.2},{:.2})", a.bone, aw[3][0], aw[3][1], aw[3][2]);
                                hand_world = Some([aw[3][0], aw[3][1], aw[3][2]]);
                            }
                        }
                    }
                }
                for o in sim.drones.scripts.objects.list.iter().flatten() {
                    if let nf_game::nis::Obj::Camera { matrix, src: (h, 0), .. } = o {
                        if *h == 0x0600_06cf { cam_mid = Some([matrix[3][0], matrix[3][1], matrix[3][2]]); }
                    }
                }
                satchel_mid = sim.script_entity((0x0600_06cf, 7), 0x0200_0758).map(|m| [m[3][0], m[3][1], m[3][2]]);
                lift_mid = sim.script_entity((0x0600_06cf, 13), 0x0200_06ee).map(|m| [m[3][0], m[3][1], m[3][2]]);
                guard_mid = sim.script_entity((0x0600_06cf, 12), 0x0200_06b1).map(|m| [m[3][0], m[3][1], m[3][2]]);
                hatch_mid = sim.script_entity((0x0600_06cf, 10), 0x0200_06e1).map(|m| [m[3][0], m[3][1], m[3][2]]);
            }
        } else if scene_ticks > 0 {
            bond_y_after = sim.bond.pos[1];
            break;
        }
    }
    eprintln!("scene ticks {scene_ticks}");
    eprintln!("satchel entity 0x02000758 (s7): {satchel_at:?}");
    eprintln!("lift floor 0x020006ee (s13): {lift_at:?}");
    eprintln!("guard anim weapon byte (s3): {guard_weapon:?}");
    eprintln!("guard anim (s3): {guard_anim:?}");
    eprintln!("bond y before {bond_y_before} after {bond_y_after}");

    // The scene's DRAW SLOTS for the pack script (the render side): one entity slot pair per
    // entity stream (two SPs run the pack — #273 and #289 [D census]), one character per anim
    // stream, and the mid-scene pose check (the pack's sequences are car-local [D root probe]).
    let mut ent_slots: Vec<(u8, u32, u8)> = Vec::new();
    for a in w.scene.attachments.iter() {
        if let Some((h, si, hash, k)) = a.script {
            if h == 0x0600_06cf { ent_slots.push((si, hash, k)); }
        }
    }
    let mut anim_slots: Vec<(u8, u32)> = Vec::new();
    for c in w.scene.characters.iter() {
        if let Some((h, si)) = c.script {
            if h == 0x0600_06cf { anim_slots.push((si, c.skin.hashcode)); }
        }
    }
    for (si, hash, _k) in ent_slots.iter() {
        assert!([0x0200_0758u32, 0x0200_06e1, 0x0200_06b1, 0x0200_06ee].contains(hash),
            "the pack's entity slots are exactly the four glists [D s7/s10/s12/s13], got {hash:08x}");
    }
    for si in [7u8, 10, 12, 13] {
        assert!(ent_slots.iter().any(|(s, _, _)| s == &si), "entity stream s{si} has a draw slot [D]");
    }
    assert_eq!(anim_slots.len(), 3, "the three cut-scene anims have characters [D s1/s2/s3]");

    // The mid-scene poses resolve (the pack's sequences loaded — facial 0x040006e6 + body
    // 0x040005fd [D 0780004a.bin]).
    assert_eq!(poses.len(), 3, "all three anim characters posed mid-scene");
    assert!(poses.iter().all(|&(_, _, ok)| ok), "every cut-scene character's pose resolves mid-scene, got {poses:?}");

    // THE CAR-LOCAL ROOTS, pinned [D 0x040005fd's root channels — read through the port's own
    // parse at mid-scene]: the body sequence's root translation ch0/1/2 reads (2.35, 0.85, -0.64)
    // with flags 0x0 and 222 channels; the facial 0x040006e6 carries flags 0x8, 22 channels and a
    // zero root. The game draws the cast IN the car (parked y -54.2 [L ride frame 700]) on these
    // roots — the base compose mechanism stays open (see the report: the anim object's obj+0x30
    // IS the raw root track [M AnimGetFirstTrans 0x1c7348 + AnimFrameResolve 0x1c35f8's delta
    // add], Script_Interp case 2 composes no base [M 0x1d9a08:140-199], and Script_AnimStart's
    // GetInterp call carries the SP-base flag but its output is unread [M disasm 0x1d90b4-
    // 0x1d9118]) — the PCSX2 obj+0x30/obj+0x90 read decides where it enters.
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let sc = sim.scripts.get(&0x0600_06d0u32).expect("the guard's script 0x060006d0 [D s3]");
        let seqs: Vec<u32> = sc.sequences().map(|(_a, _b, q)| q).collect();
        assert!(seqs.contains(&0x0400_05fd), "the guard's body sequence 0x040005fd loads [D], got {seqs:08x?}");
        let body = sim.seqs.get(&0x0400_05fd).expect("the body sequence parsed");
        let v = |ch: usize, fr: u32| body.value(ch, fr);
        assert_eq!(body.flags, 0x0, "the body sequence's header flags byte [D]");
        assert!((v(0, 1) - 2.35).abs() < 0.02 && (v(0, 100) - 2.35).abs() < 0.02,
            "the body root ch0 (x) is the car-local 2.35 [D], got {} / {}", v(0, 1), v(0, 100));
        assert!((v(1, 1) - 0.85).abs() < 0.02,
            "the body root ch1 (y) is the car-local 0.85 [D] (the car parked at y -54.2 [L]), got {}", v(1, 1));
        assert!((v(2, 1) - -0.64).abs() < 0.02,
            "the body root ch2 (z) is the car-local -0.64 [D], got {}", v(2, 1));
    }
    // The port's own draw state mid-scene: the guard now draws at the SP base ∘ root — the same
    // frame his props occupy (the compose site is // [S] unverified, see scene.rs pose_one; the
    // position is [D root (2.35,0.85,-0.64) ∘ SP base (0,-53.53,-3.38), mid-scene pose (2.46,0.56,-0.89)]).
    if let Some(gy) = guard_drawn_y {
        eprintln!("guard drawn root y {gy:.2} (the game: in the car, deck ≈ -52.7 = SP base (0,-53.53,-3.38) ∘ root (2.35,0.85,-0.64) [D+L])");
    }
    if let Some(cam) = cam_mid {
        // // [S] unverified (nis.rs case 4's scoped compose): printed, not asserted — the camera
        // rides the same base so the composed cast/props are in its frame.
        eprintln!("camera mid-scene ({:.2},{:.2},{:.2}) (raw keys ≈ (2.9,-8.3,-4.6); composed lands at the car)", cam[0], cam[1], cam[2]);
    }

    // The scene carried everything through the hand-back: the satchel and the lift floor were
    // still live near the end (their keyed paths run the whole 575 [D]).
    assert!(scene_ticks >= 500, "the landing cut-scene played (~575 ticks [D 0x060006cf]), got {scene_ticks}");
    assert!(satchel_at.is_some(), "the satchel entity 0x02000758 (s7 [D]) exists in the sim during the scene");
    assert!(lift_at.is_some(), "the lift floor 0x020006ee (s13 [D]) exists in the sim during the scene");
    assert!(satchel_mid.is_some() && lift_mid.is_some(), "the scene entities stay live mid-scene");
    // The guard's hand: event 0xB wrote the weapon byte 0x18 on the s3 anim [M 0x1d94c0 case 0xb;
    // D s4] — the held-charge mesh on the hand datum (AnimObjectDraw 0x1c4490:256-265).
    assert_eq!(guard_weapon.map(|(_, w)| w), Some(0x18), "the guard anim's weapon byte is 0x18 (event 0xB [M 0x1d94c0 case 0xb; D s4])");
    // The guard DRAWS on the car deck (base ∘ root ≈ (2.46,-52.97,-4.27) mid-scene): the pack keys
    // its whole scene car-local [D 0780004a decode], its entities compose the SP base [D placement],
    // and the guard's root sits on the pack's own lift (s13 keys (0,0.99,0) [D]) — the raw roots
    // put the guards (and their held weapons) ~54 units above the car, out of the scene.
    let gd = guard_drawn.expect("the guard's drawn root resolved mid-scene");
    assert!((gd[1] - -52.97).abs() < 0.8 && (gd[0] - 2.46).abs() < 0.8 && (gd[2] - -4.27).abs() < 0.8,
        "the guard draws at base ∘ root on the car deck [D root ∘ base], got {gd:?}");
    // The held weapon's draw slot follows its datum bone [M AnimObjectDraw 0x1c4490:256-265 +
    // AnimDatumSetEntity 0x1c7848] through the same base — and the pack's own keyed charge (s7
    // 0x02000758 [D]) is authored to that hand mid-scene: the hand-off reads as one pose.
    let hw = hand_world.expect("the guard's weapon attachment resolved mid-scene");
    let ch = satchel_mid.expect("the charge entity live mid-scene");
    let d2 = (hw[0] - ch[0]).powi(2) + (hw[1] - ch[1]).powi(2) + (hw[2] - ch[2]).powi(2);
    assert!(d2 < 9.0, "the guard's held weapon follows its parent bone to the keyed charge [D s7 keys + M 0x1c4490], hand {hw:?} vs charge {ch:?} (d² {d2:.2})");
}
