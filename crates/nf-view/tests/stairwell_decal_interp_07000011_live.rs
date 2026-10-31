//! Bullet-hole decals drawn between two ticks (07000011's stairwell).
//!
//! The decal pool is the game's 200-node DList [M `Effect_Bullet` 0x1ce8a8 0x1cea20..0x1cea58:
//! `DList_MoveFromFree2InUse(0x3496e0)`; with no free node the oldest in-use node is unlinked
//! (`Effect_UnLink` + `DList_RemoveFromInUse2Free`) and taken again]. A node never moves: a live
//! hole keeps its node until that node is reused for a new hole, and the reused node is written
//! whole (`Mat_Align2Dir` + the `hit + normal·0.01` translation, node+0x20/+0x50) before it draws.
//!
//! The port draws each pool node through one palette slot, and the window blends every palette
//! entry between the previous and the latest tick (G-043, `lerp_palette_moving`, `SNAP` 4.0).
//! Two ways that blend can put a hole in mid-air while a full pool keeps taking new holes:
//!  - a slot that is handed to a DIFFERENT hole between the two ticks (the Vec-shift mapping
//!    moved every live hole one slot down per eviction — every slot changed hole every tick);
//!  - a reused node (the evicted hole's slot) sliding from the old hole to the new one.
//!
//! The check fires a real sprayed burst through the real input path from the stairwell spot
//! (the M32 on full auto, the aim swept across the well) until the pool is full and evicting,
//! then for every evicting tick builds both ticks' palettes, blends them exactly as the window
//! does (`nf_view::moving_entries` + `lerp_palette_moving`, fraction 0.5) and requires every
//! drawn decal slot to sit on a live hole of either tick (a hole that exists in the sim) — never
//! between two of them.
//!
//! Opt-in: `NF_ISO`. NF_SILENT everywhere.

use nf_formats::anim::Mat4;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

/// The palette entries of the decal draw slots (kind 0 effect attachments).
fn decal_entries(scene: &nf_view::scene::Scene, l: &nf_view::scene::SkinLayout) -> Vec<usize> {
    scene.attachments.iter().enumerate()
        .filter(|(_, a)| matches!(a.effect, Some((0, _, _))))
        .map(|(k, _)| l.attach_base as usize + k)
        .collect()
}

fn is_far(m: &Mat4) -> bool { m[3][1] < -1000.0 }

/// The drawn decal slots of `blended` that sit on no live hole of either tick (> `tol` from all).
fn holes_between(blended: &[Mat4], entries: &[usize], live: &[[f32; 3]], tol: f32) -> Vec<[f32; 3]> {
    let mut out = Vec::new();
    for &i in entries {
        let m = &blended[i];
        if is_far(m) { continue; }
        let p = [m[3][0], m[3][1], m[3][2]];
        let near = live.iter().any(|q| {
            let d = [p[0] - q[0], p[1] - q[1], p[2] - q[2]];
            (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt() <= tol
        });
        if !near { out.push(p); }
    }
    out
}

/// Bond at the stairwell spot (the cluster-on-a-plane view: the second landing, looking across
/// the well), the Storm M32 drawn (the auto rifle), full clips.
fn setup() -> nf_view::scene::Scene {
    let mut scene = nf_view::load_with_carry_difficulty(&iso(), "07000011.bin", false, None, 3).expect("load 07000011 d3");
    let sim = scene.sim.as_mut().unwrap();
    sim.bond = nf_game::player::Bond::new([16.03, 32.17, -2.39], 0.340);
    sim.bond.pitch = -0.101;
    let wd = sim.weapons.clone();
    let rounds = wd.clip_size(0x12).max(1);
    assert!(sim.drones.inventory.equip_weapon(&wd, 0x12, rounds, true), "the M32 equips");
    sim.drones.inventory.equip_ammo(&wd, 0x12, rounds * 3);
    sim.step_bond(nf_game::player::Input { switch_to: Some(0x12), ..Default::default() }, false, false, false);
    scene
}

/// One held-trigger tick with the aim swept across the well (yaw ±0.45, pitch ±0.2 around the
/// spot's view) — the spray that leaves holes on the far wall, the side walls and the flights.
fn spray_tick(scene: &mut nf_view::scene::Scene, t: u32) {
    let sim = scene.sim.as_mut().unwrap();
    sim.bond.pos = [16.03, 32.17, -2.39];
    sim.bond.vel = [0.0; 3];
    let k = t as f32;
    sim.bond.yaw = 0.340 + 0.45 * (k * 0.131).sin();
    sim.bond.pitch = -0.101 + 0.2 * (k * 0.057).sin();
    for c in sim.drones.inventory.clip.iter_mut() { if *c == 0 { *c = 30; } }
    for c in sim.drones.inventory.carried.iter_mut() { *c = 90; }
    let fire = true;
    sim.step_bond(nf_game::player::Input { fire, ..Default::default() }, fire, false, false);
}

fn live_holes(scene: &nf_view::scene::Scene) -> Vec<[f32; 3]> {
    scene.sim.as_ref().unwrap().drones.effects.decals.iter().map(|d| d.pos).collect()
}

#[test]
fn sprayed_holes_never_draw_between_two_holes_while_the_pool_evicts() {
    let Ok(_) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set — skipping"); return };
    std::env::set_var("NF_SILENT", "1");
    let mut scene = setup();
    // Fill the pool: spray until the oldest holes are being evicted.
    let mut t = 0u32;
    while scene.sim.as_ref().unwrap().drones.effects.evicted < 5 {
        spray_tick(&mut scene, t);
        t += 1;
        assert!(t < 6000, "the spray filled the 200-hole pool (evicted {} after {t} ticks, {} holes)",
            scene.sim.as_ref().unwrap().drones.effects.evicted, scene.sim.as_ref().unwrap().drones.effects.decals.len());
    }
    let (layout, _) = scene.skin_layout();
    let entries = decal_entries(&scene, &layout);
    assert!(!entries.is_empty(), "the level has decal draw slots");
    let opts = nf_view::scene::ViewOpts::default();
    let mut checked = 0u32;
    let mut bad: Vec<(u32, [f32; 3])> = Vec::new();
    for _ in 0..240 {
        let ev0 = scene.sim.as_ref().unwrap().drones.effects.evicted;
        let holes0 = live_holes(&scene);
        let (p0, _) = scene.skin_palette(&layout, None, 240.0, opts, None);
        spray_tick(&mut scene, t);
        t += 1;
        let ev1 = scene.sim.as_ref().unwrap().drones.effects.evicted;
        if ev1 == ev0 { continue; }
        let (p1, _) = scene.skin_palette(&layout, None, 240.0, opts, None);
        // The window's blend, exactly: the moving entries (minus the reused decal nodes) blended
        // at the fraction, every other entry at the newer tick.
        let snap = scene.decal_reused_entries(&layout, ev0);
        let moving = nf_view::moving_entries(&p0, &p1, &snap);
        let mut blended = p1.clone();
        nf_view::lerp_palette_moving(&mut blended, &p0, &p1, &moving, &[], 0.5);
        let mut live = holes0;
        live.extend(live_holes(&scene));
        for p in holes_between(&blended, &entries, &live, 0.02) { bad.push((t, p)); }
        checked += 1;
    }
    assert!(checked >= 10, "at least ten evicting ticks were blended (got {checked})");
    assert!(
        bad.is_empty(),
        "{} drawn decal slots sat between two holes on {checked} evicting ticks (first: {:?}) — a hole drawn in mid-air",
        bad.len(), bad.first()
    );
}

/// The sister: the same measure flags a decal slot that blends from one live hole to another
/// (the mid-air draw the check exists for), so the zero above is not a blind measure.
#[test]
fn the_between_holes_measure_flags_a_blended_hole() {
    let Ok(_) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set — skipping"); return };
    std::env::set_var("NF_SILENT", "1");
    let mut scene = setup();
    let mut t = 0u32;
    while scene.sim.as_ref().unwrap().drones.effects.decals.len() < 2 {
        spray_tick(&mut scene, t);
        t += 1;
        assert!(t < 600, "two holes made");
    }
    let (layout, _) = scene.skin_layout();
    let entries = decal_entries(&scene, &layout);
    let (p1, _) = scene.skin_palette(&layout, None, 240.0, nf_view::scene::ViewOpts::default(), None);
    let live = live_holes(&scene);
    // Find two drawn holes far enough apart that their midpoint is off both.
    let drawn: Vec<usize> = entries.iter().copied().filter(|&i| !is_far(&p1[i])).collect();
    assert!(drawn.len() >= 2, "two decal slots draw");
    let (a, b) = (drawn[0], drawn[1]);
    let mut p0 = p1.clone();
    p0[a] = p1[b];
    assert!(holes_between(&p1, &entries, &live, 0.02).is_empty(), "the unblended palette draws every slot on a live hole");
    let blended = nf_view::lerp_palette(&p0, &p1, 0.5);
    let d = (0..3).map(|k| (p1[a][3][k] - p1[b][3][k]).powi(2)).sum::<f32>().sqrt();
    if d > 0.05 && d <= 4.0 {
        assert_eq!(holes_between(&blended, &entries, &live, 0.02).len(), 1, "a slot blended from one hole to another is flagged");
    } else {
        // The first two holes coincide or sit a snap apart: move the second slot's source 1 unit.
        let mut p0 = p1.clone();
        p0[a][3][0] += 1.0;
        let blended = nf_view::lerp_palette(&p0, &p1, 0.5);
        assert_eq!(holes_between(&blended, &entries, &live, 0.02).len(), 1, "a slot blended off its hole is flagged");
    }
}
