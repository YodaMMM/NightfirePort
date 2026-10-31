//! The rigid bodies (0xD8) draw at their body's pose. `RB_Create` 0x1d6bf0:53 points the body's
//! drawn matrix at obj+0x90 and `RB_EulerEvolve` 0x1d7810 rewrites it once the body is disturbed
//! [M], while the draw (`View_DrawObjectGlist` 0x1e6780) renders obj+0xD8 at obj+0x90. 07000004's
//! two hanging lamps (#39, #40 [D]) were baked into the static draw: a shot lamp's body swung but
//! the lamp on screen never moved.
//!
//! Opt-in: `NF_ISO`. NF_SILENT everywhere.

#[path = "common/walker.rs"]
#[allow(dead_code)]
mod walker;
use walker::Walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn opts() -> nf_view::scene::ViewOpts { nf_view::scene::ViewOpts::default() }

#[test]
fn the_07000004_lamp_draws_at_its_body_pose_after_a_shot() {
    let Ok(_) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set — skipping"); return };
    std::env::set_var("NF_SILENT", "1");
    let mut w = Walker::new(nf_view::load(&iso(), "07000004.bin", false).expect("load 07000004"));
    assert_eq!(w.scene.rbody_placements, vec![39, 40], "07000004's 0xD8 rows are the two lamps [D #39/#40]");
    // `lock_slot`'s chain, then one slot per rotor, then one per rigid body.
    let base = {
        let sim = w.scene.sim.as_ref().unwrap();
        sim.drones.doors.list.len() + sim.drones.sss.list.len() + sim.drones.breaks.list.len()
            + 2 * sim.drones.locks.len()
            + (1 + nf_game::gunimp::TURRET_ARTS.len()) * w.scene.gunimp_placements
            + 3 * sim.drones.sensors.list.len()
            + w.scene.dynobj_matrices.len()
            + w.scene.rotor_placements.len()
    };
    for bi in 0..2 {
        assert!(w.scene.char_verts.iter().any(|v| v.door == Some((base + bi) as u16)), "lamp {bi}: the mesh rides its slot");
    }
    // Sister: no static World draw sits at lamp #40 any more (the static copy would hide the swing).
    let at40 = |m: &nf_view::scene::Material| (m.centre[0] - 8.01).abs() < 0.5 && (m.centre[2] - 23.03).abs() < 0.5 && (m.centre[1] - 7.2).abs() < 1.0;
    assert!(!w.scene.materials.iter().any(|m| at40(m) && m.layer == nf_view::scene::Layer::World), "lamp #40 left the static draw");
    let (layout, _attrs) = w.scene.skin_layout();
    let pose = |scene: &nf_view::scene::Scene| {
        let (pal, _cols) = scene.skin_palette(&layout, None, 240.0, opts(), None);
        pal[layout.door_base as usize + base + 1]
    };
    let body = |scene: &nf_view::scene::Scene| {
        let st = &scene.sim.as_ref().unwrap().drones.rbodies.list.iter().find(|o| o.placement == 40).unwrap().st;
        let r = st.matrix;
        [[r[0][0], r[0][1], r[0][2], 0.0], [r[1][0], r[1][1], r[1][2], 0.0], [r[2][0], r[2][1], r[2][2], 0.0], [st.trans[0], st.trans[1], st.trans[2], 1.0]]
    };
    let close = |a: [[f32; 4]; 4], b: [[f32; 4]; 4]| (0..4).all(|r| (0..4).all(|c| (a[r][c] - b[r][c]).abs() < 1e-4));
    for _ in 0..6000u32 {
        w.tick(Default::default(), false);
        let sim = w.scene.sim.as_ref().unwrap();
        if sim.drones.npc_enabled && !sim.disabled && sim.drones.channels.on(0x53) { break; }
    }
    let rest = pose(&w.scene);
    assert!(close(rest, body(&w.scene)), "at rest the lamp draws at its body (= the placement) [M 0x1d6bf0:53]");
    w.walk_nav_to([8.0, 4.4, 26.5], "under the whitelamp", 1.2);
    let at = body(&w.scene)[3];
    w.shoot_point([at[0], at[1], at[2]], "the whitelamp", |sim| sim.drones.rbodies.list.iter().find(|o| o.placement == 40).unwrap().st.omega != [0.0; 3]);
    for _ in 0..120 { w.tick(Default::default(), false); }
    let swung = pose(&w.scene);
    assert!(close(swung, body(&w.scene)), "after the shot the drawn lamp follows the body [M RB_EulerEvolve 0x1d7810]");
    assert!(!close(swung, rest), "the shot lamp's drawn pose moved off its rest pose");
}
