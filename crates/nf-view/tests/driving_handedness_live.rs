//! The driving engine's world is left-handed against the on-foot renderer: seen from the chase camera,
//! the Vanquish's rear number plate (vanquish.ssh atlas, u 0.75..1.0 at v 0.625..0.75, the strip at
//! car z -2.15) reads left to right only when the view mirrors screen x (`Scene::mirror_x`). Without
//! it the plate, the track's sign text and the lane the car starts in (Paris drives on the right)
//! all came out mirrored. Opt-in: NF_ISO.

#[test]
fn the_vanquish_plate_reads_left_to_right_from_the_chase_camera() {
    let Ok(iso) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set — skipping"); return };
    let (scene, drv) = nf_view::driving::load_driving_scene(&iso, "MIS01").expect("load MIS01");
    assert!(scene.mirror_x, "driving scenes mirror screen x");
    let key = scene.textures.iter().find(|t| t.width == 256 && t.height == 256 && {
        // the car atlas: the plate's yellow fills most of u 0.76..0.98, v 0.63..0.72
        let mut yellow = 0;
        for y in 162..184usize { for x in 196..250usize {
            let p = &t.rgba[(y * 256 + x) * 4..][..4];
            if p[0] > 150 && p[1] > 120 && p[2] < 90 { yellow += 1; }
        } }
        yellow > 22 * 54 / 2
    }).map(|t| t.key).expect("the vanquish atlas with the yellow plate");
    // The plate's leftmost-u and rightmost-u vertices (car space).
    let mut lo: Option<([f32; 3], f32)> = None;
    let mut hi: Option<([f32; 3], f32)> = None;
    for m in scene.materials.iter().filter(|m| m.key == Some(key)) {
        for k in 0..m.index_count as usize {
            let v = scene.vertices[scene.indices[m.first_index as usize + k] as usize];
            if v.uv[0] > 0.74 && v.uv[1] > 0.6 && v.uv[1] < 0.76 && v.pos[2] < -2.0 {
                if lo.is_none_or(|(_, u)| v.uv[0] < u) { lo = Some((v.pos, v.uv[0])); }
                if hi.is_none_or(|(_, u)| v.uv[0] > u) { hi = Some((v.pos, v.uv[0])); }
            }
        }
    }
    let (lo, hi) = (lo.expect("plate vertices").0, hi.expect("plate vertices").0);
    let (s, c) = drv.car.yaw.sin_cos();
    let world = |p: [f32; 3]| glam::Vec4::new(p[0] * c + p[2] * s + drv.car.pos[0], p[1] + drv.car.pos[1], -p[0] * s + p[2] * c + drv.car.pos[2], 1.0);
    let (eye, yaw, pitch) = drv.chase_camera();
    let cam = nf_view::Camera { pos: glam::Vec3::from(eye), yaw, pitch, fov: nf_view::GAME_FOV };
    let (vp, _) = cam.view_proj_for(16.0 / 9.0, scene.mirror_x);
    let vp = glam::Mat4::from_cols_array_2d(&vp);
    let sx = |w: glam::Vec4| { let q = vp * w; q.x / q.w };
    let (a, b) = (sx(world(lo)), sx(world(hi)));
    assert!(a < b, "the plate's u=0.75 edge ({a}) sits left of its u=1.0 edge ({b}) on screen");
}
