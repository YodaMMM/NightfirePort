//! level-07-guest-faces: the talking guest's face animates with the line.
//!
//! The game's path: a party guest's BondTalk line rides `NDrone2_BondTalk` 0x1446b0 →
//! `NDrone2_SpeechSFX` 0x141b00's FACIAL arm — `AnimScriptPeekSfx` 0x1c6ca8 finds the line's
//! facial script, `AnimFacialAppend` 0x1c6490 appends it (marker 0xe0000002 at script+0x80,
//! stop-at-end +0xb5 = 3) and stores it at DCVars+0x53c; `AnimFacialBlend` 0x1be648 then
//! blends the three marker scripts' +0x54 buffers over the morph weight array (animobj+0xb0,
//! skin+0x2e floats — 22 on every face skin) into `AnimObjectDraw` 0x1c4490's Morph_Weight
//! marshalling. In gameplay only marker scripts feed it [M AnimScriptInit 0x1c1600:58-61 —
//! new scripts get marker 0], so the talk script IS the mouth: a guest whose body anim carries
//! no facial window (the 07000007 census: the guests' idle/walk anims carry none) must still
//! get the talk script's weights over the zero base. Pre-fix the port returned None there and
//! the slot-2 consumer was hardwired None — the face never moved.
//!
//! Through the real path (`nf_view::load` + `Sim::step_bond`, silent). Opt-in: NF_ISO.
use nf_game::player::Input;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

fn step(scene: &mut nf_view::scene::Scene) {
    scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false);
}

/// The mouth-flap threshold: a live weight (AnimFacialBlend's |w| > 0.01 override gate
/// [M 0x1be648], with margin against the lerp's float noise).
const LIVE: f32 = 0.05;

#[test]
fn a_talking_guests_face_weights_follow_the_line() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    unsafe { std::env::set_var("NF_SILENT", "1"); }
    let mut scene = nf_view::load(&iso, "07000007.bin", false).expect("load 07000007");
    // Free Bond through the intro, then stop the scripted-talk system (the speech_facial_fallback
    // harness) so the only talk lines are the ones Bond's approach provokes.
    for _ in 0..3000 { step(&mut scene); if !scene.sim.as_ref().unwrap().disabled { break; } }
    scene.sim.as_mut().unwrap().drones.scripts.skip_request = true;
    for _ in 0..300 { step(&mut scene); }
    assert!(!scene.sim.as_ref().unwrap().disabled, "Bond is free after the intro");

    // Approach the nearest awake type-13 guest and aim at him until his BondTalk takes the FACIAL
    // arm (a present-script attach: `facial_talk` — the port's +0x53c [M 0x141b00:18-23]).
    let mut target: Option<usize> = None;
    let mut attached: Option<u32> = None;
    for t in 0..4500u32 {
        {
            let sim = scene.sim.as_ref().unwrap();
            let i = target.unwrap_or_else(|| {
                let bp = sim.bond.pos;
                sim.drones.list.iter().enumerate()
                    .filter(|(_, d)| d.present && d.enabled && !d.corpse && d.voice_type == 13
                        && d.health > 0.0 && d.skin & 0xffff != 0x22)
                    .min_by_key(|(_, d)| ((d.pos[0] - bp[0]).powi(2) + (d.pos[2] - bp[2]).powi(2)) as u32)
                    .map(|(x, _)| x).expect("an awake type-13 guest [D census]")
            });
            let d = &sim.drones.list[i];
            let (npos, nyaw) = (d.pos, d.yaw);
            let sim = scene.sim.as_mut().unwrap();
            let off = [0.0f32, 0.7, -0.7, 1.4, -1.4][(t / 90) as usize % 5]; // dodge crowd blockers
            let a = nyaw + off;
            let (s2, c2) = (a.sin(), a.cos());
            sim.bond.pos = [npos[0] + s2 * 2.2, npos[1], npos[2] + c2 * 2.2];
            sim.bond.yaw = (npos[0] - sim.bond.pos[0]).atan2(npos[2] - sim.bond.pos[2]);
            let dy = 0.4f32 - 1.6; // aim mid-body [M 0x1a7810]
            sim.bond.pitch = (dy / 2.2f32.hypot(1.2)).asin();
        }
        step(&mut scene);
        let sim = scene.sim.as_ref().unwrap();
        if target.is_none() {
            if let Some(ai) = sim.drones.list.iter().position(|x| x.aimed_at) {
                let ad = &sim.drones.list[ai];
                if ad.voice_type == 13 && ad.skin & 0xffff != 0x22 { target = Some(ai); }
            }
        }
        if let Some(i) = target {
            if let Some(h) = sim.drones.list[i].facial_talk {
                attached = Some(h);
                break;
            }
        }
    }
    let di = target.expect("the crosshair reached a type-13 guest");
    let hash = attached.expect("a present-script BondTalk line attached (the FACIAL arm [M 0x141b00])");
    {
        let sim = scene.sim.as_ref().unwrap();
        let d = &sim.drones.list[di];
        assert!(sim.scripts.contains_key(&hash), "the attached script is a level script");
        assert!(sim.voices.as_deref().unwrap().iter().any(|l| l.facial == hash),
            "the attached script is a voice line's facial script [M 0x141b00 appends the entry's facialAnimId]");
    }

    // The guest's character link; the skin is morph-bearing (the census: facial 22).
    let sim = scene.sim.as_ref().unwrap();
    let ci = sim.char_drone.iter().position(|x| *x == Some(di))
        .expect("the guest is a rendered character");
    drop(sim);
    let count = scene.characters[ci].skin.facial as usize;
    assert_eq!(count, 22, "the guest skin's facial count [D census: every face skin is 22]");

    // Sample the weights every tick while the talk script plays; the mouth flaps — the weights
    // carry live values and CHANGE across the line (the talk seq's keyframes, e.g. 040002ed's
    // 64 frames [D probe_talk_script]).
    let mut samples: Vec<Vec<f32>> = Vec::new();
    for _ in 0..200u32 {
        {
            let sim = scene.sim.as_ref().unwrap();
            if sim.drones.list[di].facial_talk.is_none() { break; }
        }
        let w = {
            let sim = scene.sim.as_ref().unwrap();
            let c = &scene.characters[ci];
            scene.facial_weights_at(ci, c)
        };
        if let Some(w) = w {
            assert_eq!(w.len(), count, "the weight array is the skin's facial count");
            samples.push(w);
        }
        step(&mut scene);
    }
    assert!(!samples.is_empty(), "the talking guest is a morph-weight source while the line plays");
    eprintln!("[guest-faces] attach {hash:08x} samples {}", samples.len());
    // The mouth flap: the largest per-channel range across the line (a talk track also holds
    // constant channels — the census: 040002ed's neighbour pegs one at 1.0 — so the metric is
    // per channel, not the array peak).
    let flap = (0..count).map(|ch| {
        let hi = samples.iter().map(|w| w[ch]).fold(f32::MIN, f32::max);
        let lo = samples.iter().map(|w| w[ch]).fold(f32::MAX, f32::min);
        hi - lo
    }).fold(0.0f32, f32::max);
    assert!(samples.iter().any(|w| w.iter().any(|v| *v > LIVE)),
        "the talk weights carry live values during the line (the mouth opens)");
    assert!(flap > 0.3, "the weights CHANGE across the line — the mouth flaps (best channel range {flap})");

    // The stop-at-end detach ([M +0xb5 = 3]: the script ends → +0x53c clears): the face returns
    // to the flat base — no live talk weights once the line is over (the port's idle/blink slots
    // are unported, so the base alone is zero — the 07000007 guests' body anims carry no face
    // window; the ambient machine is the remaining work, see the report).
    let mut flat_seen = false;
    for _ in 0..900u32 {
        {
            let sim = scene.sim.as_ref().unwrap();
            if sim.drones.list[di].facial_talk.is_some() { continue; } // wait out any re-attach window
        }
        let w = {
            let sim = scene.sim.as_ref().unwrap();
            let c = &scene.characters[ci];
            scene.facial_weights_at(ci, c)
        };
        if let Some(w) = w {
            if w.iter().all(|v| v.abs() <= LIVE) {
                flat_seen = true;
                break;
            }
        }
        step(&mut scene);
    }
    assert!(flat_seen, "after the line ends the talk weights leave the face (the mouth stops)");
}
