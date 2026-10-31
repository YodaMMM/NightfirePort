//! ambient-sweep: every drawn object's TAmbient follows the game's 0x800 rule.
//!
//! [M] `View_SetupRenderModes_Tweak` 0x1e5888: an object whose obj+0xF0 carries 0x800 draws at
//! `tint × obj+0x100..0x102 >> 8`; without it at the plain tint. `control_init_object` 0x134630 seeds
//! obj+0x100..0x102 with the 0xff sentinel; `control_link_object_to_cel` 0x1346b8 / `build_LinkToRoom`
//! 0x1c9680 snap them to the linked cel's +0x9a/+0x9c/+0x9e bytes and `Lights_CalcAmbientLight`
//! 0x1cb178 eases them toward the cel each tick for the moving objects. obj+0xF0 is the glist's flags
//! only where the creator copies them (`Control_SetGList` 0x134960 or a direct `glist+0x44` store);
//! Lock_Create 0x1ae1f8, RB_Create 0x1d6bf0, OneSided_Create 0x1e7ce8 and Mine_Create 0x17ce28 never
//! write it, so those objects draw undimmed.
//!
//! Opt-in via NF_ISO.

use nf_game::player::Input;
use nf_view::scene::Scene;

/// The campaign's on-foot level archives (0x41+ are the non-campaign maps).
fn campaign_levels(iso: &str) -> Vec<String> {
    let img = nf_formats::iso9660::IsoImage::open(iso).unwrap();
    nf_view::matrix::on_foot_levels(&img).into_iter()
        .filter(|n| u32::from_str_radix(&n[..8], 16).map_or(false, |c| (c & 0xff) <= 0x1b))
        .collect()
}

/// Drawn characters whose main mesh carries the 0x800 glist flag and whose ambient is still the
/// 0xff sentinel (never linked to a cel, so drawn at full brightness).
fn sentinel_drawn(scene: &Scene) -> Vec<usize> {
    let s = scene.sim.as_ref().unwrap();
    let fs = scene.frame_state_for(None, 0.0, Default::default(), true);
    (0..scene.characters.len()).filter(|&ci| {
        !scene.char_hidden(ci, Default::default()) && !fs.hidden[ci]
            && scene.characters[ci].skin.main.iter().flatten().any(|h| scene.models_0x800.get(h).copied().unwrap_or(false))
            && s.char_ambient[ci] == [0xff; 3]
    }).collect()
}

fn tick(scene: &mut Scene) { scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false); }

/// Through each level's start (its intro cut-scene's actors included) and 150 ticks of play, no
/// drawn 0x800 character is left at the sentinel.
#[test]
fn no_dim_flagged_drawn_character_keeps_the_sentinel() {
    let Some(iso) = std::env::var("NF_ISO").ok() else { return };
    let levels = campaign_levels(&iso);
    assert!(levels.len() >= 19, "the campaign's on-foot archives: {levels:?}");
    let mut bad = Vec::new();
    for lvl in levels {
        let mut scene = nf_view::load(&iso, &lvl, false).unwrap();
        let mut t = 0;
        let mut after = 0;
        while after < 150 && t < 4000 {
            tick(&mut scene);
            t += 1;
            if !scene.sim.as_ref().unwrap().disabled { after += 1; }
            if t % 30 == 0 {
                for ci in sentinel_drawn(&scene) { bad.push(format!("{lvl} tick {t} character {ci} skin {:08x}", scene.characters[ci].skin.hashcode)); }
            }
        }
    }
    assert!(bad.is_empty(), "drawn 0x800 characters still at the 0xff sentinel: {bad:#?}");
}

/// Sister: a drawn guard cut off from every position source (no drone, no script) keeps the
/// sentinel, and the sweep's detector reports it — the comparison above can fail.
#[test]
fn sister_an_unlinked_guard_is_reported() {
    let Some(iso) = std::env::var("NF_ISO").ok() else { return };
    let mut scene = nf_view::load(&iso, "07000005.bin", false).unwrap();
    for _ in 0..60 { tick(&mut scene); }
    assert!(sentinel_drawn(&scene).is_empty(), "07000005 starts clean");
    let ci = {
        let s = scene.sim.as_ref().unwrap();
        (0..scene.characters.len()).find(|&ci| s.char_drone[ci].is_some() && !scene.char_hidden(ci, Default::default())
            && matches!(scene.characters[ci].role, nf_formats::drone::SpawnRole::Placed)).expect("a drawn placed guard")
    };
    let s = scene.sim.as_mut().unwrap();
    s.char_drone[ci] = None;
    s.char_ambient[ci] = [0xff; 3];
    for _ in 0..30 { tick(&mut scene); }
    assert!(sentinel_drawn(&scene).contains(&ci), "the unlinked guard stays at the sentinel and is reported");
}

/// The posed slots (doors, SimpleScripts, breakables, locks, gun emplacements, sensors, dynamic
/// objects, rotors, rigid bodies, searchlight parts): on every campaign level none of their meshes
/// carries a 0x800 glist [D the build's `door_dims`/`sl_dims` are all None], so the game draws them
/// at the plain 0xff tint and every palette colour slot of theirs is white.
#[test]
fn posed_slots_draw_white_where_their_glists_lack_0x800() {
    let Some(iso) = std::env::var("NF_ISO").ok() else { return };
    for lvl in campaign_levels(&iso) {
        let mut scene = nf_view::load(&iso, &lvl, false).unwrap();
        for _ in 0..30 { tick(&mut scene); }
        assert!(scene.door_dims.iter().chain(&scene.sl_dims).all(|d| d.is_none()), "{lvl}: a posed slot carries a 0x800 glist");
        let (l, _) = scene.skin_layout();
        let (_, cols) = scene.skin_palette(&l, None, 0.0, Default::default(), None);
        for k in l.col_door..l.col_one {
            assert_eq!(&cols[k as usize][..3], &[1.0; 3], "{lvl}: door-region colour {} is dimmed", k - l.col_door);
        }
    }
}

/// The 0x800 posed-slot path itself: a slot given a dim draws at its multiplier in the GPU palette
/// and in the CPU vertex path (the same `posed_dim_mult`); the undimmed slot next to it stays white.
#[test]
fn a_dim_flagged_posed_slot_takes_its_multiplier() {
    let Some(iso) = std::env::var("NF_ISO").ok() else { return };
    let mut scene = nf_view::load(&iso, "07000001.bin", false).unwrap();
    for _ in 0..30 { tick(&mut scene); }
    let (l, _) = scene.skin_layout();
    assert!(l.col_one - l.col_door >= 2, "07000001 has door-region slots");
    let want = [0.25f32, 0.5, 0.75];
    if scene.door_dims.is_empty() { scene.door_dims.resize(1, None); }
    scene.door_dims[0] = Some(nf_view::scene::PosedDim { centre: [0.0; 3], snap: want, live: false });
    let (_, cols) = scene.skin_palette(&l, None, 0.0, Default::default(), None);
    assert_eq!(&cols[l.col_door as usize][..3], &want, "the dimmed slot's palette colour");
    assert_eq!(&cols[l.col_door as usize + 1][..3], &[1.0; 3], "its undimmed neighbour stays white");
    // The CPU path: door-0 vertices scale by the same bytes.
    let verts = scene.posed_vertices(None, 0.0, Default::default());
    let base = &scene.vertices[scene.char_vertex_start..];
    let mut n = 0;
    for ((v, b), cv) in verts.iter().zip(base).zip(&scene.char_verts) {
        if cv.door != Some(0) { continue; }
        for k in 0..3 { assert_eq!(v.rgba[k], ((b.rgba[k] as f32 * want[k]).round() as i32).clamp(0, 255) as u8); }
        n += 1;
    }
    assert!(n > 0, "door 0 has vertices");
}

/// 07000015's tripbombs (kind 0xfe, `Mine_Create` 0x17ce28) draw undimmed: the mesh's glist carries
/// 0x800 [D placement 487 'Tripbomb_3rd_short'], but the creator never copies it into obj+0xF0, so the
/// red beam keeps its full 0xfe red vertex colour.
#[test]
fn the_tripbomb_beam_draws_undimmed() {
    let Some(iso) = std::env::var("NF_ISO").ok() else { return };
    let scene = nf_view::load(&iso, "07000015.bin", false).unwrap();
    let at = [2.869, 41.986, 59.459]; // placement 487's translation [D]
    let red: Vec<[u64; 3]> = scene.materials.iter().filter(|m| m.layer == nf_view::scene::Layer::World)
        .filter(|m| (0..3).map(|k| (m.centre[k] - at[k]).powi(2)).sum::<f32>() < 16.0)
        .map(|m| {
            let mut acc = [0u64; 3];
            let ix = &scene.indices[m.first_index as usize..(m.first_index + m.index_count) as usize];
            for i in ix { for k in 0..3 { acc[k] += scene.vertices[*i as usize].rgba[k] as u64; } }
            acc.map(|a| a / (ix.len() as u64).max(1))
        })
        .filter(|a| a[0] > 4 * (a[1] + a[2] + 1)).collect();
    assert!(!red.is_empty(), "the beam's material sits by placement 487");
    assert!(red.iter().all(|a| a[0] >= 250), "the beam is undimmed: {red:?}");
    // Sister: the cel ambient there would have dimmed it below half.
    let s = scene.sim.as_ref().unwrap();
    let amb = s.ambient_at(at).expect("the beam is inside a room cel");
    assert!(amb[0] < 128, "a 0x800 dim would cut the red to {}", 254 * amb[0] as u32 >> 8);
}
