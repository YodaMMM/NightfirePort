//! One-shot probe for level-07-guest-faces: (1) a present BondTalk facial script's sequence
//! shape (flags, channels, frames, live weights), (2) whether the guests' body anims carry
//! facial windows (flags 0x8 sequences) while standing near Bond, (3) the drone/character link
//! + `facial_weights_at` behavior during a talk. Opt-in: NF_ISO.
use nf_game::player::Input;

fn main() {
    let Some(iso) = std::env::var("NF_ISO").ok() else { eprintln!("NF_ISO not set"); return };
    let mut scene = nf_view::load(&iso, "07000007.bin", false).expect("load 07000007");
    // free bond through the intro
    for _ in 0..3000 {
        scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false);
        if !scene.sim.as_ref().unwrap().disabled { break; }
    }
    let sim = scene.sim.as_mut().unwrap();
    sim.drones.scripts.skip_request = true;
    drop(sim);
    for _ in 0..300 { scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false); }

    // 1. the type-14 talk scripts' sequence shapes (all present per the census)
    {
        let sim = scene.sim.as_ref().unwrap();
        let voices = sim.voices.as_deref().unwrap();
        for i in 0x84..0x89 {
            let h = voices[i].facial;
            let Some(sc) = sim.scripts.get(&h) else { println!("line 0x{i:02x}: script absent"); continue };
            let seqs: Vec<_> = sc.sequences().collect();
            let (first, _second, q) = seqs[0];
            match sim.seqs.get(&q) {
                Some(sq) => {
                    let w1: Vec<f32> = (0..sq.channels.len().min(22)).map(|c| sq.value(c, 1)).collect();
                    let live = w1.iter().filter(|v| v.abs() > 0.01).count();
                    println!("line 0x{i:02x} hash {h:08x} len {} seqs {} first {first} seq {q:08x} flags {:x} ch {} frames {} live@f1 {live}",
                        sc.length, seqs.len(), sq.flags, sq.channels.len(), sq.frames);
                }
                None => println!("line 0x{i:02x} hash {h:08x}: seq {q:08x} MISSING"),
            }
        }
    }

    // 2. stand near a type-14 guest and watch body anims / face windows / the drone link
    let mut target: Option<usize> = None;
    for t in 0..2700u32 {
        {
            let sim = scene.sim.as_ref().unwrap();
            let i = target.unwrap_or_else(|| {
                let bp = sim.bond.pos;
                sim.drones.list.iter().enumerate()
                    .filter(|(_, d)| d.present && d.enabled && !d.corpse && d.voice_type == 13
                        && d.health > 0.0 && d.skin & 0xffff != 0x22)
                    .min_by_key(|(_, d)| ((d.pos[0] - bp[0]).powi(2) + (d.pos[2] - bp[2]).powi(2)) as u32)
                    .map(|(x, _)| x).unwrap_or(usize::MAX)
            });
            if i == usize::MAX { break; }
            let d = &sim.drones.list[i];
            let (npos, nyaw) = (d.pos, d.yaw);
            let sim = scene.sim.as_mut().unwrap();
            let off = [0.0f32, 0.7, -0.7, 1.4, -1.4][(t / 90) as usize % 5];
            let a = nyaw + off;
            let (s2, c2) = (a.sin(), a.cos());
            sim.bond.pos = [npos[0] + s2 * 2.2, npos[1], npos[2] + c2 * 2.2];
            sim.bond.yaw = (npos[0] - sim.bond.pos[0]).atan2(npos[2] - sim.bond.pos[2]);
            let dy = 0.4f32 - 1.6;
            sim.bond.pitch = (dy / 2.2f32.hypot(1.2)).asin();
        }
        scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false);
        let sim = scene.sim.as_ref().unwrap();
        if let Some(ai) = sim.drones.list.iter().position(|x| x.aimed_at) {
            let ad = &sim.drones.list[ai];
            if ad.voice_type == 14 || ad.voice_type == 13 { target = Some(ai); }
        }
        if let Some(i) = target {
            let d = &sim.drones.list[i];
            if d.facial_talk.is_some() || d.facial_played.len() >= 2 {
                // report the body anim + face windows + the character link
                let bp = sim.bond.pos;
                let _ = bp;
                let script_h = d.anim.script;
                let (face_win, ch, flags) = match script_h.and_then(|h| sim.scripts.get(&h)) {
                    Some(sc) => {
                        let wins: Vec<_> = sc.sequences().filter_map(|(a, _b, q)| {
                            sim.seqs.get(&q).filter(|sq| sq.flags & 0x8 != 0).map(|sq| (a, q, sq.channels.len(), sq.flags))
                        }).collect();
                        (wins.len(), wins.first().map(|w| w.1), wins.first().map(|w| w.3))
                    }
                    None => (0, None, None),
                };
                let ci = sim.char_drone.iter().position(|x| *x == Some(i));
                let ft = d.facial_talk.map(|h| format!("{h:08x}")).unwrap_or_else(|| "-".into());
                let chw = ch.map(|c| format!("{c:08x}")).unwrap_or_else(|| "-".into());
                let fl = flags.map(|f| format!("{f:x}")).unwrap_or_else(|| "-".into());
                let played = format!("{:?}", d.facial_played);
                println!("[t{t}] guest d{i} skin {:08x} vt {} body {:?} face_wins {} seq {chw} flags {fl} talk {ft} start {} played {played} char {ci:?} ftime {:.2}",
                    d.skin, d.voice_type, script_h, face_win, d.facial_talk_start, d.anim.ftime);
                if let Some(ci) = ci {
                    let c = &scene.characters[ci];
                    let w = scene.facial_weights_at(ci, c);
                    match &w {
                        Some(v) => println!("    weights_at: Some len {} live {} first5 {:?}",
                            v.len(), v.iter().filter(|x| x.abs() > 0.01).count(), &v[..v.len().min(5)]),
                        None => println!("    weights_at: None"),
                    }
                }
                if d.facial_played.len() >= 2 { break; }
            }
        }
    }
}
