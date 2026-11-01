//! The hanging rigid bodies are solid to Bond's walk.
//!
//! `RB_Create` 0x1d6bf0 links each kind-0xD8 body into its room's cel chain
//! (`build_LinkToRoom` 0x1c9680, :81). The player's movement query is a kind-0x800 cylinder with
//! HITTEST+0x98 = 0 [M `Player_Collision` 0x194df0:88,205 — BL+0x7D4/+0x7D8]. `Collide_Pick`
//! 0x1eb320:314-329 walks each candidate cel's linked objects into `Collide_PickObj` 0x1eae80
//! (skipped only for +0x98 & 0x10 or obj+0xF0 & 0x30000000). PickObj admits the body: its
//! obj+0xF0 carries none of the mask bits and its glist has collision geometry (obj+0xD8 → +8 ≠
//! 0). `Collide_Intersect` 0x1ebaa8 copies the body's live obj+0x90 (:106), runs
//! `Intersect_CylGeom` 0x109da0 over the glist (:296) and pushes the cylinder out by the contact
//! (:354-373). So Bond is stopped by the body's own triangles at its live pose.
//!
//! The bodies are anonymous objects (hashcode 0xFFFFFFFF); their triangles sit under the
//! synthetic `0x8000_0000 | map << 16 | object` key (`RbObj::shape_key`). Registering them under
//! the raw hashcode left them with no collision.
//!
//! 0700000d's three bodies (placements 89/90/91) hang beside the mounted gun: a slab 2.38 wide,
//! 0.2 deep and 4.03 tall (y 4.22..8.25), its foot 0.23 above the deck (3.985) [D the posed
//! shape: x 132.59..134.97, z −304.15..−303.95 for #91; #89 starts at x 139.73]. Bond walks north
//! (−z) into #91 and stops at its south face; the sister walks the same line through the 4.8-wide
//! gap between #91 and #89 and carries on.
//!
//! Not modelled: the contact's mirror onto the body's own obj+0xD0 list (Collide_Update
//! 0x1ea398:162-215), so the port's body does not swing from the walk-in.
//!
//! Opt-in via `NF_ISO`.

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

/// Load 0700000d and release Bond from the intro.
fn load() -> Option<nf_view::scene::Scene> {
    unsafe { std::env::set_var("NF_SILENT", "1"); }
    let iso = iso()?;
    let mut scene = nf_view::load(&iso, "0700000d.bin", false).expect("load 0700000d");
    {
        let sim = scene.sim.as_mut().unwrap();
        for _ in 0..2000 {
            sim.step_bond(Default::default(), false, false, false);
            if !sim.disabled { break; }
        }
        assert!(!sim.disabled, "the intro releases Bond");
    }
    Some(scene)
}

/// Bond walks north (−z) from (x, 4.5, −300) for `ticks`; returns his final position.
fn walk_north(scene: &mut nf_view::scene::Scene, x: f32, ticks: u32) -> [f32; 3] {
    let sim = scene.sim.as_mut().unwrap();
    sim.bond.pos = [x, 4.5, -300.0];
    sim.bond.vel = [0.0; 3];
    sim.bond.yaw = std::f32::consts::PI; // the port's forward is (sin, cos): π faces −z
    sim.bond.pitch = 0.0;
    for _ in 0..ticks {
        sim.step_bond(nf_game::player::Input { forward: 1.0, ..Default::default() }, false, false, false);
    }
    sim.bond.pos
}

/// The slab's south face (the posed shape's max z) and its x span, from the live pose.
fn slab(scene: &nf_view::scene::Scene, placement: u16) -> (f32, f32, f32) {
    let sim = scene.sim.as_ref().unwrap();
    let o = sim.drones.rbodies.list.iter().find(|o| o.placement == placement).expect("the body exists");
    let tris = sim.world.shape(o.shape_key).expect("the body's shape key resolves to triangles");
    let m = o.matrix();
    let (mut zmax, mut xlo, mut xhi) = (f32::MIN, f32::MAX, f32::MIN);
    for t in tris {
        for p in t {
            let w = nf_formats::placement::transform(&m, *p);
            zmax = zmax.max(w[2]);
            xlo = xlo.min(w[0]);
            xhi = xhi.max(w[0]);
        }
    }
    (zmax, xlo, xhi)
}

#[test]
fn bond_walking_into_the_hanging_slab_is_stopped_at_its_face() {
    let Some(mut scene) = load() else { eprintln!("skip: NF_ISO not set"); return };
    {
        let sim = scene.sim.as_ref().unwrap();
        let o = sim.drones.rbodies.list.iter().find(|o| o.placement == 91).expect("body #91");
        assert_eq!(o.glist, u32::MAX, "#91 is an anonymous object");
        assert_ne!(o.shape_key, o.glist, "the shape key is the synthetic (map, object) key");
    }
    let (face, xlo, xhi) = slab(&scene, 91);
    let x = (xlo + xhi) * 0.5;
    let end = walk_north(&mut scene, x, 150);
    eprintln!("into #91 at x {x}: end {end:?}, face z {face}");
    // Stopped south of the face: never through it, and right up against it (the walk covers
    // ~4 units of open deck first).
    assert!(end[2] > face, "Bond walked through the slab: z {} past the face {face}", end[2]);
    assert!(end[2] < face + 1.0, "Bond stopped short of the slab: z {} vs the face {face}", end[2]);
    assert!((end[0] - x).abs() < 0.5, "a head-on walk-in stays on its line (x {} vs {x})", end[0]);
}

/// Sister: the same walk through the gap between #91 and #89 crosses the bodies' line.
#[test]
fn the_same_walk_through_the_gap_beside_the_slab_carries_on() {
    let Some(mut scene) = load() else { eprintln!("skip: NF_ISO not set"); return };
    let (face, _, hi91) = slab(&scene, 91);
    let (_, lo89, _) = slab(&scene, 89);
    let x = (hi91 + lo89) * 0.5;
    assert!(lo89 - hi91 > 3.0, "the gap is wide enough for Bond ({hi91}..{lo89})");
    let end = walk_north(&mut scene, x, 150);
    eprintln!("gap at x {x}: end {end:?}, face z {face}");
    assert!(end[2] < face - 1.0, "the gap walk must pass the slabs' line: z {} vs the face {face}", end[2]);
}
