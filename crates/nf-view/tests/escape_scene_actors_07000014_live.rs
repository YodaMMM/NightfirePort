//! level-14-rook-cutscene: 07000014's escape scene 0x06000848 (SP #733, 950 ticks) — the
//! post-cascade cut-scene at the Delta Sector door — must draw Rook.
//!
//! Evidence base:
//! - The scene's actors are its two `Script_AnimStart` streams [D decode, host-float:
//!   `07A00014.bin!06000848`]: stream 1 = skin 0x0500008C (the masked guard, anim
//!   0x0600085b → body seq 0x0400072c, 222 ch) and stream 9 = skin 0x05000066 — **Rook's own
//!   skin** [D docs/levels/07000014.md: Rook's drone skin 0x05000066], anim 0x0600085a whose
//!   windows are the 22-channel facial seq 0x04000745 [1..979] and the 222-channel body seq
//!   0x0400072b [1..951] [D 07000014.bin kind-5 + 07A00014.bin kind-4]. The game binds a FRESH
//!   anim object with that skin, not Rook's drone: `Script_AnimStart` 0x1d8f40 runs
//!   `control_create_object(0, 0x2d6200, 0x2d6200, 0)` (0x2d6200 = the ORIGIN vector) +
//!   `AnimObjectNew(obj, skin)` + `AnimScriptAdd(obj, script)` [M disasm 0x1d8f40:69-74].
//! - The camera sits at (27.82, 0.97, −20.38) until t750 looking back at the exit door
//!   #944 (27.79, 1.28, −25.32) [D camera keys]; the guard's body seq roots at
//!   (27.89, 1.17, −24.93) — in front of the door — so the game shows the guard AND Rook.
//! - The port drew the guard only. These tests pin the whole chain that puts Rook in the
//!   frame: the pre-built character slot (skin 0x05000066, script (0x06000848, 9)), the live
//!   `Obj::Anim`, the posed root on the body track, and the render-path palette at the
//!   script camera's eye cell. The sister pins stream 9's skin: a port that bound the
//!   guard's skin 0x0500008C into stream 9 fails it.
//!
//! Opt-in via `NF_ISO`.

use nf_game::player::Input;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

/// The escape scene 0x06000848's actors: (stream, skin, anim script) [D 07A00014.bin decode].
const ACTORS: [(u8, u32, u32); 2] = [(1, 0x0500_008c, 0x0600_085b), (9, 0x0500_0066, 0x0600_085a)];
/// The scene's own NIS hash (the pack 07A00014.bin kind-7 entry) [D].
const SCENE: u32 = 0x0600_0848;
/// The guard's body seq roots here at its frame 1 [D 0400072c ch 0..2 @1] — in front of the
/// Delta Sector exit door #944 (27.79, 1.28, −25.32 [D placement]).
const GUARD_ROOT: [f32; 3] = [27.8919, 1.1667, -24.9266];
/// Rook's body seq 0x040072b roots here at its frame 1 [D 0400072b ch 0..2 @1] — beside the
/// guard, in front of the door. The game draws the root UNSCALED: `AnimFrameCopy` 0x1c5668
/// unscales the buffer's first translation record (`Vec_Mul(param_2, 0x319d90)` [M :536]) even
/// though the skin's scale is 1.1 [D the 0x05000066 record] — the port's pre-fix 1.1× root
/// (29.97, 1.39, −26.49) put him behind the door plane, out of the shot.
const ROOK_ROOT: [f32; 3] = [27.2446, 1.2597, -24.0801];

fn scene_char(scene: &nf_view::scene::Scene, st: u8, skin: u32) -> Option<usize> {
    scene.characters.iter().position(|c| c.script == Some((SCENE, st)) && c.skin.hashcode == skin)
}

/// The real start path: the sabotage channel 0x8C armed ([D] AND #717 [0x87, 0x88] → 0x8C),
/// Bond crosses the escape touch #456 (out 0x0C) → SEQ_AND #771 [0x8C, 0x0C] → 0x09 → SP #733.
fn start_the_scene(iso: &str) -> nf_view::scene::Scene {
    let mut scene = nf_view::load(iso, "07000014.bin", false).expect("07000014.bin loads");
    {
        let sim = scene.sim.as_mut().unwrap();
        sim.drones.channels.set(0x8c, 1, sim.drones.frame);
        sim.bond.pos = [-33.5, 0.9, 26.5];
        let mut fired = false;
        for _ in 0..600u32 {
            sim.step_bond(Input::default(), false, false, false);
            if sim.script_camera().is_some() { fired = true; break; }
        }
        assert!(fired, "the escape scene 0x06000848 never started (no script camera) — the touch #456 + SEQ_AND #771 path [D]");
    }
    assert!(
        scene.sim.as_ref().unwrap().drones.scripts.players.iter()
            .any(|p| p.scripts.contains(&SCENE) && p.state == 2),
        "SP #733 is not running the escape scene from the touch path"
    );
    scene
}

/// The census at the scene's start: both actors are (a) created — live `Obj::Anim`s bound to
/// their own skins and anim scripts, (b) pre-built characters with those (script, stream) pairs,
/// (c) posed — `script_pose_together` finite with the root on the body track, not the origin.
#[test]
fn the_escape_scene_actors_live_and_posed_from_the_touch() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = start_the_scene(&iso);
    for _ in 0..30u32 { scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false); }
    let sim = scene.sim.as_ref().unwrap();
    for &(st, skin, asc) in &ACTORS {
        // (b) the pre-built character slot: the seeding walk built a Character for this
        // (scene, stream) pair and bound the stream's own skin.
        let ci = scene_char(&scene, st, skin);
        assert!(ci.is_some(), "stream {st}: the scene's skin {skin:08x} built no character (unresolved skin or a missed Cmd::Anim)");
        assert!(sim.skins.contains_key(&skin), "skin {skin:08x} is not in the level's skins [D 07000014.bin kind-3]");
        // (a) the live Obj::Anim, bound to the stream's anim script.
        let (script, frame) = sim.script_anim((SCENE, st), skin)
            .unwrap_or_else(|| panic!("stream {st} skin {skin:08x}: no live Obj::Anim at the scene's start"));
        assert_eq!(script, asc, "stream {st}: the live Obj::Anim binds the wrong anim script [D decode]");
        // Rook's actor must carry ROOK's skin — a port that bound the guard's 0x0500008C here
        // is the reported symptom (the masked guard in front of the door, Rook missing).
        if st == 9 {
            assert_eq!(skin, 0x0500_0066, "stream 9 must be Rook's skin 0x05000066 [D decode; docs/levels/07000014.md]");
            let ci = ci.unwrap();
            assert_eq!(scene.characters[ci].skin.hashcode, 0x0500_0066,
                "stream 9's character carries skin {:08x}, not Rook's 0x05000066",
                scene.characters[ci].skin.hashcode);
        }
        // (c) posed on the body track, finite, not collapsed to the origin.
        let pose = nf_game::anim::script_pose_together(&sim.scripts, &sim.seqs, &scene.characters[ci.unwrap()].skeleton, &scene.characters[ci.unwrap()].skin, script, frame)
            .unwrap_or_else(|| panic!("stream {st}: script_pose_together None for {script:08x} @ {frame}"));
        assert!(!pose.is_empty() && pose.iter().all(|(t, q)| t.iter().all(|v| v.is_finite()) && q.iter().all(|v| v.is_finite()) && q.iter().any(|v| *v != 0.0)),
            "stream {st}: non-finite/zeroed pose for {script:08x} @ {frame}");
        let root = pose[0].0;
        assert!(root.iter().all(|v| v.abs() > 0.01) || st == 9,
            "stream {st}: the root collapsed to the world origin at {script:08x} @ {frame} (the fix-cutscenes2 signature)");
    }
    // The guard stands where the disc puts him: in front of the Delta Sector door.
    let (script, frame) = sim.script_anim((SCENE, 1), 0x0500_008c).expect("guard Obj::Anim");
    let ci = scene_char(&scene, 1, 0x0500_008c).unwrap();
    let pose = nf_game::anim::script_pose_together(&sim.scripts, &sim.seqs, &scene.characters[ci].skeleton, &scene.characters[ci].skin, script, frame).expect("guard pose");
    let root = pose[0].0;
    assert!((root[0] - GUARD_ROOT[0]).abs() < 0.05 && (root[1] - GUARD_ROOT[1]).abs() < 0.05 && (root[2] - GUARD_ROOT[2]).abs() < 0.05,
        "the guard's root {root:?} left the disc's frame-1 root {GUARD_ROOT:?} [D 0400072c]");
    // ROOK stands beside him — the UNSCALED body-track root. The skin's scale is 1.1 [D the
    // 0x05000066 record], and the game does not apply it to the root track (AnimFrameCopy's
    // tail Vec_Mul [M 0x1c5668:536]): a port that scales the root (the pre-fix bug) draws him
    // at (29.97, 1.39, −26.49) — behind the door plane, the missing Rook.
    let (rscript, rframe) = sim.script_anim((SCENE, 9), 0x0500_0066).expect("rook Obj::Anim");
    let rci = scene_char(&scene, 9, 0x0500_0066).unwrap();
    let rook_skin = &scene.characters[rci].skin;
    assert!((rook_skin.scale[0] - 1.1).abs() < 0.001 && (rook_skin.scale[1] - 1.1).abs() < 0.001 && (rook_skin.scale[2] - 1.1).abs() < 0.001,
        "Rook's skin scale should read 1.1 from the disc record [D 0x05000066], got {:?}", rook_skin.scale);
    let rpose = nf_game::anim::script_pose_together(&sim.scripts, &sim.seqs, &scene.characters[rci].skeleton, rook_skin, rscript, rframe).expect("rook pose");
    let rroot = rpose[0].0;
    assert!((rroot[0] - ROOK_ROOT[0]).abs() < 0.05 && (rroot[1] - ROOK_ROOT[1]).abs() < 0.05 && (rroot[2] - ROOK_ROOT[2]).abs() < 0.05,
        "Rook's root {rroot:?} left the disc's frame-1 root {ROOK_ROOT:?} — the skin scale must not reach the root track \
         ([M AnimFrameCopy 0x1c5668:536's Vec_Mul(param_2, 0x319d90)]; the 1.1× root (29.97, 1.39, −26.49) is the missing-Rook bug)");
    eprintln!("rook root at start: {rroot:?} (disc frame-1 {ROOK_ROOT:?}, scale {:?})", rook_skin.scale);
}

/// The render path: with the script camera driving the eye and the portal walk feeding
/// `skin_palette`'s room mask, NEITHER actor's palette may go FAR (room-hidden) or collapse
/// while the scene plays — what the live draw loop uploads.
#[test]
fn the_escape_scene_actors_survive_the_render_path() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = start_the_scene(&iso);
    let g = scene.cel_graph.clone();
    let (layout, _) = scene.skin_layout();
    let far = [[0.0f32; 4], [0.0; 4], [0.0; 4], [0.0, -10000.0, 0.0, 1.0]];
    let mut last = 0u32;
    for extra in [30u32, 240, 600] {
        for _ in 0..(extra - last) { scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false); }
        last = extra;
        let sim = scene.sim.as_ref().unwrap();
        let Some((pos, yaw, pitch, _fov)) = sim.script_camera() else { panic!("no script camera at +{last}") };
        let world = scene.sim.as_ref().map(|s| &s.world);
        let vp = view_proj(pos, yaw, pitch, 16.0 / 9.0);
        let depth0 = depth0_game(vp, pos);
        let mut state = nf_view::scene::ViewerCel::default();
        let cell = nf_view::scene::eye_cell(&g, &mut state, pos, world);
        let vis: Option<Vec<bool>> = cell.and_then(|c| nf_view::scene::portal_visible_cels_from_cell(&g, c, pos, &depth0))
            .map(|cell_vis| g.room_to_cell.iter().map(|m| m.map(|c2| cell_vis[c2]).unwrap_or(true)).collect());
        let (pal, _) = scene.skin_palette(&layout, None, 300.0, nf_view::scene::ViewOpts::default(), vis.as_deref());
        for &(st, skin, _) in &ACTORS {
            let ci = scene_char(&scene, st, skin).expect("pre-built");
            let base = layout.char_base[ci] as usize;
            let nb = scene.characters[ci].skeleton.bones as usize;
            let mats = &pal[base..=(base + nb).min(pal.len() - 1)];
            assert!(!mats.iter().all(|m| *m == far), "[+{last}] stream {st} skin {skin:08x}: palette FAR (room-hidden) at the script camera's eye cell {cell:?}");
            assert!(!mats.iter().all(|m| m[0] == [0.0; 4] && m[1] == [0.0; 4] && m[2] == [0.0; 4]),
                "[+{last}] stream {st} skin {skin:08x}: palette collapsed (frame_state hidden)");
        }
    }
}

fn view_proj(pos: [f32; 3], yaw: f32, pitch: f32, aspect: f32) -> [[f32; 4]; 4] {
    use glam::{Mat4, Vec3};
    let (near_p, far, fov) = (0.05f32, 4000.0f32, std::f32::consts::FRAC_PI_3);
    let f = 1.0 / (fov * 0.5).tan();
    let proj = Mat4::from_cols_array_2d(&[
        [f / aspect, 0.0, 0.0, 0.0],
        [0.0, f, 0.0, 0.0],
        [0.0, 0.0, far / (near_p - far), -1.0],
        [0.0, 0.0, near_p * far / (near_p - far), 0.0],
    ]);
    let cp = pitch.cos();
    let fw = Vec3::new(cp * yaw.sin(), pitch.sin(), cp * yaw.cos()).normalize();
    let s = fw.cross(Vec3::Y).normalize();
    let u = s.cross(fw);
    let view = Mat4::from_cols_array_2d(&[
        [s.x, u.x, -fw.x, 0.0],
        [s.y, u.y, -fw.y, 0.0],
        [s.z, u.z, -fw.z, 0.0],
        [-s.dot(Vec3::from(pos)), -u.dot(Vec3::from(pos)), fw.dot(Vec3::from(pos)), 1.0],
    ]);
    (proj * view).to_cols_array_2d()
}

fn depth0_game(vp: [[f32; 4]; 4], eye: [f32; 3]) -> [[f32; 4]; 5] {
    let planes = nf_view::gpu::extract_frustum(vp);
    let fw = [planes[4][0], planes[4][1], planes[4][2]];
    let fe = fw[0] * eye[0] + fw[1] * eye[1] + fw[2] * eye[2];
    [planes[0], planes[1], planes[2], planes[3], [-fw[0], -fw[1], -fw[2], fe + 200.0]]
}
