//! perf-240g: the palette build's fast paths (`AttachPlan`: the effect-pool slots walked once per
//! slot, the colour arms decided once per scene, the colour tail's index lists) against the
//! reference walk (`PALETTE_REFERENCE`: every attachment through `att_one`, every colour slot
//! through the full arm chain). Per level, 600 ticks of the perf_state_hash input (walk, turn,
//! strafe, fire from tick 200, so projectiles, decals and debris come and go); after every tick
//! the palette and colours are built both ways and must be bit-identical.
//!
//! Sister assertion: the comparison can fail — a palette with one entry's bits flipped differs.
//! `#[ignore]`, opt-in: `NF_ISO`; `NF_HASH_LEVELS` narrows the levels (default: all on-foot).

use nf_view::scene::{ViewOpts, PALETTE_REFERENCE};
use std::sync::atomic::Ordering;

fn input_at(t: u32) -> (nf_game::player::Input, bool) {
    let mut i = nf_game::player::Input::default();
    match (t / 90) % 6 {
        0 => { i.forward = 1.0; }
        1 => { i.forward = 1.0; i.turn = 0.6; }
        2 => { i.strafe = 1.0; i.forward = 0.5; }
        3 => { i.turn = -0.8; }
        4 => { i.forward = -1.0; i.strafe = -0.5; }
        _ => { i.forward = 1.0; i.turn = 0.3; }
    }
    (i, t % 37 < 4 && t > 200)
}

fn levels(iso: &str) -> Vec<String> {
    if let Ok(l) = std::env::var("NF_HASH_LEVELS") { return l.split(',').map(|s| format!("{}.bin", s.trim())).collect(); }
    let img = nf_formats::iso9660::IsoImage::open(iso).expect("iso");
    let (entries, _) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let mut names: Vec<String> = entries.iter().map(|e| e.name.to_ascii_lowercase())
        .filter(|n| n.starts_with("0700") && n.ends_with(".bin"))
        .filter(|n| { let code = u32::from_str_radix(&n[..8], 16).unwrap_or(0) & 0xff;
            !(0x23..=0x29).contains(&code) && code != 0x00 && n != "07000500.bin" && n != "07000999.bin" })
        .collect();
    names.sort(); names.dedup(); names
}

fn bits(p: &[[[f32; 4]; 4]], c: &[[f32; 4]]) -> (Vec<u32>, Vec<u32>) {
    (p.iter().flat_map(|m| m.iter().flatten().map(|f| f.to_bits())).collect(), c.iter().flatten().map(|f| f.to_bits()).collect())
}

#[test]
#[ignore]
fn palette_fast_paths_match_the_reference_walk() {
    let Some(iso) = std::env::var("NF_ISO").ok() else { eprintln!("NF_ISO not set"); return };
    let ticks: u32 = std::env::var("NF_HASH_TICKS").ok().and_then(|v| v.parse().ok()).unwrap_or(600);
    let mut compared = 0usize;
    let mut live_pool_ticks = 0usize;
    for name in levels(&iso) {
        let Ok(mut scene) = nf_view::load(&iso, &name, false) else { eprintln!("[pal-id] {name} not loadable"); continue };
        let (layout, _) = scene.skin_layout();
        let (mut pf, mut cf, mut pr, mut cr) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
        let mut first_diff = None;
        for t in 0..ticks {
            let (inp, fire) = input_at(t);
            let Some(sim) = scene.sim.as_mut() else { break };
            sim.step_bond(inp, fire, false, false);
            let s = scene.sim.as_ref().unwrap();
            if !s.drones.bullets.is_empty() || !s.drones.player_bullets.is_empty() || !s.drones.effects.decals.is_empty() || !s.drones.effects.debris.is_empty() { live_pool_ticks += 1; }
            let yaw = t as f32 * 0.05;
            let mut opts = ViewOpts::default();
            opts.right = [yaw.cos(), 0.0, -yaw.sin()];
            opts.up = [0.0, 1.0, 0.0];
            PALETTE_REFERENCE.store(false, Ordering::Relaxed);
            scene.skin_palette_into(&layout, None, t as f32, opts, None, &mut pf, &mut cf);
            PALETTE_REFERENCE.store(true, Ordering::Relaxed);
            scene.skin_palette_into(&layout, None, t as f32, opts, None, &mut pr, &mut cr);
            PALETTE_REFERENCE.store(false, Ordering::Relaxed);
            if first_diff.is_none() && bits(&pf, &cf) != bits(&pr, &cr) {
                let at = pf.iter().zip(&pr).position(|(a, b)| a.iter().flatten().map(|f| f.to_bits()).ne(b.iter().flatten().map(|f| f.to_bits())));
                first_diff = Some((t, at));
            }
            compared += 1;
        }
        eprintln!("[pal-id] {name} ticks {ticks} mats {} cols {} {}", pf.len(), cf.len(), match first_diff { None => "identical".to_string(), Some((t, at)) => format!("DIFFERS at tick {t} entry {at:?}") });
        assert!(first_diff.is_none(), "{name}: fast palette differs from the reference walk at {first_diff:?}");
        // Sister: one flipped bit is seen.
        if let Some(m) = pf.first_mut() { m[0][0] = f32::from_bits(m[0][0].to_bits() ^ 1); }
        assert_ne!(bits(&pf, &cf), bits(&pr, &cr), "the comparison must be able to fail");
    }
    eprintln!("[pal-id] {compared} builds compared, {live_pool_ticks} ticks with live pool objects");
    assert!(compared > 0);
}
