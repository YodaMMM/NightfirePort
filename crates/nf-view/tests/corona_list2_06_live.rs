//! corona-list2: a searchlight corona in clear sight draws in sort list 2, over the world.
//!
//! `Searchlight_Update` sets `obj+0xF0 |= 0x2000` (and scales ×1.5) when `Collide_LineOfSight`
//! from the viewer to the corona is clear, clears it when blocked, and leaves it alone while the
//! alpha byte is 0 [M 0x1cb900:337-347]. `View_AddObjects` 0x1e62d8:57-59 routes a 0x2000 object
//! to list 2, which `Game_Draw` 0x1e6f50:157-176 draws after `psiClearZ` — the Z clear runs while
//! `WipeZBuffer` 0x30c759 = 1 [D ACTION.ELF data; L ee_start_05.bin] — so nothing in the world
//! occludes it. The port gives each corona a second mesh copy in pass 6 (no depth test) and shows
//! exactly one of the two copies from `Part::sight`.
//!
//! The spot: Bond at (0, 41.5, −23.6) facing east under head #2 (7.13, 47.57, −23.60) with the
//! courtyard lit (ch 0x34, what the entry plates write [D trigger census]). The head tracks him and
//! its corona sits behind the wall's parapet; the depth-tested draw cut it off at the parapet
//! (audits/render-1031-corona/before-06_corona.png).
//!
//! Opt-in via NF_ISO.

use nf_game::player::Input;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

const SPOT: [f32; 3] = [0.0, 41.5, -23.6];

fn lit_scene(iso: &str) -> nf_view::scene::Scene {
    let mut scene = nf_view::load(iso, "07000006.bin", false).expect("load 07000006");
    let sim = scene.sim.as_mut().unwrap();
    for _ in 0..3000 { sim.step_bond(Input::default(), false, false, false); if !sim.disabled { break; } }
    assert!(!sim.disabled, "Bond is free after the intro");
    sim.drones.hold_npcs = true;
    sim.drones.channels.set(0x34, 1, 0);
    sim.bond = nf_game::player::Bond::new(SPOT, std::f32::consts::FRAC_PI_2);
    sim.bond.pitch = 0.3;
    for _ in 0..400 { sim.step_bond(Input::default(), false, false, false); }
    scene
}

/// The corona part linked to the head nearest (7.13, 47.57, −23.60) — head #2 [D census].
fn corona_of_head2(sim: &nf_view::scene::Sim) -> usize {
    let parts = &sim.drones.searchlights.parts;
    let head = parts.iter().position(|p| p.kind == 1 && (p.pos[0] - 7.13).abs() < 0.2 && (p.pos[2] + 23.6).abs() < 0.2)
        .expect("head #2");
    parts[head].links[4].expect("head #2 has a corona")
}

/// Every (pass, fog, searchlight slot) the scene's materials draw.
fn searchlight_draws(scene: &nf_view::scene::Scene) -> Vec<(u8, u8, u16)> {
    let mut out = Vec::new();
    for m in &scene.materials {
        let idx = &scene.indices[m.first_index as usize..(m.first_index + m.index_count) as usize];
        for &i in idx {
            let Some(k) = (i as usize).checked_sub(scene.char_vertex_start) else { continue };
            if let Some(sl) = scene.char_verts.get(k).and_then(|v| v.searchlight) {
                // One material can batch several parts sharing a texture: record every slot once.
                if !out.contains(&(m.draw.pass, m.draw.fog, sl)) { out.push((m.draw.pass, m.draw.fog, sl)); }
            }
        }
    }
    out
}

fn opts() -> nf_view::scene::ViewOpts {
    nf_view::scene::ViewOpts { right: [0.0, 0.0, 1.0], up: [0.0, 1.0, 0.0], ..Default::default() }
}

/// Max |y| offset test: a slot's vertices are drawn (near the corona) or collapsed FAR (y −10000).
fn slot_visible(scene: &nf_view::scene::Scene, pos: &[[f32; 3]], slot: u16) -> Option<bool> {
    let ys: Vec<f32> = scene.char_verts.iter().zip(pos).filter(|(v, _)| v.searchlight == Some(slot)).map(|(_, p)| p[1]).collect();
    if ys.is_empty() { return None; }
    Some(ys.iter().all(|&y| y > -9000.0))
}

#[test]
fn an_in_sight_corona_draws_from_the_list2_copy() {
    let Some(iso) = iso() else { eprintln!("skip: NF_ISO not set"); return };
    let scene = lit_scene(&iso);
    let sim = scene.sim.as_ref().unwrap();
    let c = corona_of_head2(sim);
    let p = &sim.drones.searchlights.parts[c];
    eprintln!("corona {c}: drawn {} sight {} alpha {} scale {} pos {:?}", p.drawn, p.sight, p.alpha, p.scale, p.pos);
    assert!(p.drawn && p.alpha > 0, "the tracking head's corona is drawn");
    assert!(p.sight, "the viewer sees the corona: the 0x2000 bit is set [M 0x1cb900:342-344]");

    // The copy's slot: after the parts and the LOD slots.
    let n_parts = sim.drones.searchlights.parts.len();
    let k = scene.sl_l2.iter().position(|&h| h as usize == c).expect("the corona has a list-2 copy");
    let copy = (n_parts + scene.sl_lod.len() + k) as u16;
    let draws = searchlight_draws(&scene);
    let own: Vec<_> = draws.iter().filter(|d| d.2 == c as u16).collect();
    let l2: Vec<_> = draws.iter().filter(|d| d.2 == copy).collect();
    assert!(!own.is_empty() && !l2.is_empty(), "both copies have draws");
    assert!(own.iter().all(|d| d.0 == 3), "the corona's own mesh is in the alpha list (glist flags 0x229: bit 0x1) {own:?}");
    assert!(l2.iter().all(|d| d.0 == 6 && d.1 == 0), "the copy is pass 6 (list 2, no depth test, no fog) {l2:?}");

    let pos = scene.animate(None, 0.0, opts());
    assert_eq!(slot_visible(&scene, &pos, copy), Some(true), "the list-2 copy shows while in sight");
    assert_eq!(slot_visible(&scene, &pos, c as u16), Some(false), "the alpha-list copy is hidden while in sight");
}

/// Sister: the same frame with the bit cleared (what a blocked sight test writes) shows the
/// alpha-list copy and hides the list-2 one — the visibility checks above can fail.
#[test]
fn sister_a_blocked_corona_stays_in_the_alpha_list() {
    let Some(iso) = iso() else { eprintln!("skip: NF_ISO not set"); return };
    let mut scene = lit_scene(&iso);
    let c = corona_of_head2(scene.sim.as_ref().unwrap());
    scene.sim.as_mut().unwrap().drones.searchlights.parts[c].sight = false;
    let n_parts = scene.sim.as_ref().unwrap().drones.searchlights.parts.len();
    let k = scene.sl_l2.iter().position(|&h| h as usize == c).expect("the corona has a list-2 copy");
    let copy = (n_parts + scene.sl_lod.len() + k) as u16;
    let pos = scene.animate(None, 0.0, opts());
    assert_eq!(slot_visible(&scene, &pos, copy), Some(false), "no list-2 draw without the bit");
    assert_eq!(slot_visible(&scene, &pos, c as u16), Some(true), "the alpha-list copy draws");
}
