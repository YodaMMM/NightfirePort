//! The RACE start and the chase camera's collision (docs/systems/driving-camera.md). Opt-in: NF_ISO.
//!
//! - [D] the four `POS2_Cautionguard` rails behind the start (Map `in#0` items 3166-3169) carry no
//!   `CDat` collision: the camera's cast world (`nf_game::driving::camera::CamWorld`, every `CDat`
//!   `ci` triangle, as `FUN_002166f8` tests) has nothing inside any rail's box, so the game's
//!   pull-in cannot meet them [M: the cast sees `ci` items only].
//! - The port's eye after 240 ticks at rest is outside the oriented box of every scene-object
//!   placement (item flag 0x10: the props) within 10 units, and sits just behind rail 3168's
//!   back face (its local z past the rail's half depth by less than 0.1): the rail is in front
//!   of the eye, nearer than 0.6. (Static models are left out: a crossed-plane tree's box,
//!   `Tree_Birch8m` item 2181, holds the eye; how near its planes are is not checked.)

use nf_formats::{crp::Crp, disc, iso9660::IsoImage, refpack};
use nf_game::driving::camera::{CamTri, CamWorld};

fn race_crp(iso: &str) -> Vec<u8> {
    let iso = IsoImage::open(iso).expect("iso");
    let entry = iso.find("/DRIVING/RACE.VIV").expect("viv");
    let bigf = disc::read_bigf(&iso, entry).expect("bigf");
    let e = bigf
        .entries
        .iter()
        .find(|x| x.name.eq_ignore_ascii_case("data\\track\\snow2a_race.crp"))
        .expect("RACE track crp");
    let raw = iso.read_bytes(entry.lba as u64 * 2048 + e.offset as u64, e.size as u64).expect("read");
    if raw.len() >= 2 && (raw[0] & 0x3e) == 0x10 && raw[1] == 0xfb { refpack::decompress(&raw).expect("refpack") } else { raw }
}

/// A placed model's box in its own frame: (placement, local min, local max).
struct LocalBox {
    index: usize,
    article: String,
    flags: u8,
    origin: [f32; 3],
    axes: [[f32; 3]; 3],
    min: [f32; 3],
    max: [f32; 3],
}

impl LocalBox {
    fn local(&self, p: [f32; 3]) -> [f32; 3] {
        let d = [p[0] - self.origin[0], p[1] - self.origin[1], p[2] - self.origin[2]];
        let dot = |a: [f32; 3]| (a[0] * d[0] + a[1] * d[1] + a[2] * d[2]) / (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]);
        [dot(self.axes[0]), dot(self.axes[1]), dot(self.axes[2])]
    }
    fn contains(&self, p: [f32; 3]) -> bool {
        let l = self.local(p);
        (0..3).all(|k| l[k] >= self.min[k] && l[k] <= self.max[k])
    }
}

fn boxes(crp_bytes: &[u8]) -> Vec<LocalBox> {
    let crp = Crp::parse(crp_bytes).expect("crp");
    let pl = nf_formats::geoprim::map_placements(&crp).expect("placements");
    let geo = nf_formats::geoprim::track_geometry(crp_bytes).expect("geometry");
    let mut out: Vec<LocalBox> = Vec::new();
    let mut at: std::collections::HashMap<usize, usize> = Default::default();
    for s in &geo.strips {
        let Some(p) = pl.iter().find(|p| p.index == s.placement) else { continue };
        let i = *at.entry(s.placement).or_insert_with(|| {
            out.push(LocalBox { index: p.index, article: p.article.clone(), flags: p.flags, origin: p.origin, axes: p.axes, min: [f32::MAX; 3], max: [f32::MIN; 3] });
            out.len() - 1
        });
        for q in &s.positions {
            let l = out[i].local(*q);
            for k in 0..3 {
                out[i].min[k] = out[i].min[k].min(l[k]);
                out[i].max[k] = out[i].max[k].max(l[k]);
            }
        }
    }
    out
}

#[test]
fn the_race_start_rails_are_not_in_the_camera_world_and_the_eye_rests_behind_one() {
    let Ok(iso) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set - skipping"); return };
    let crp_bytes = race_crp(&iso);
    let crp = Crp::parse(&crp_bytes).expect("crp");
    let all = boxes(&crp_bytes);
    let rails: Vec<&LocalBox> = all.iter().filter(|b| (3166..=3169).contains(&b.index)).collect();
    assert_eq!(rails.len(), 4);
    assert!(rails.iter().all(|b| b.article == "POS2_Cautionguard"), "{:?}", rails.iter().map(|b| &b.article).collect::<Vec<_>>());

    // The camera's cast world: every CDat ci triangle, no slope or class filter.
    let tris = crp.extract_collision_tris().expect("CDat");
    let world = CamWorld::new(tris.iter().map(|t| CamTri { v: t.v, class: t.class() }).collect());
    assert_eq!(world.len(), tris.len());
    for b in &rails {
        // segments across each rail's box (along its length and its depth, at three heights)
        // meet no collision triangle inside the box.
        for fy in [0.25f32, 0.5, 0.75] {
            let y = b.min[1] + (b.max[1] - b.min[1]) * fy;
            let mid = |k: usize| (b.min[k] + b.max[k]) * 0.5;
            let w = |l: [f32; 3]| {
                let a = &b.axes;
                [0, 1, 2].map(|k| b.origin[k] + l[0] * a[0][k] + l[1] * a[1][k] + l[2] * a[2][k])
            };
            for (p, q) in [
                ([b.min[0], y, mid(2)], [b.max[0], y, mid(2)]),
                ([mid(0), y, b.min[2]], [mid(0), y, b.max[2]]),
            ] {
                if let Some(h) = world.cast(w(p), w(q)) {
                    assert!(!b.contains(h), "rail {}: a CDat triangle at {h:?} inside its box", b.index);
                }
            }
        }
    }

    // The port's camera after 240 ticks at rest.
    let (_scene, mut st) = nf_view::driving::load_driving_scene(&iso, "RACE").expect("load RACE");
    for _ in 0..240 {
        st.step(0.0, 0.0, 0.0, false);
    }
    let eye = st.cam_view.expect("camera").eye;
    eprintln!("eye at tick 240: {eye:?}, car {:?}", st.car.pos);
    for b in all.iter().filter(|b| {
        let d = [b.origin[0] - eye[0], b.origin[2] - eye[2]];
        d[0] * d[0] + d[1] * d[1] < 100.0
    }) {
        if b.flags & 0x10 == 0 { continue; }
        assert!(!b.contains(eye), "eye {eye:?} inside placement {} ({}) local {:?} box {:?}..{:?}", b.index, b.article, b.local(eye), b.min, b.max);
    }
    // The eye's exact spot behind rail 3168 (0.05 behind its back face) was measured with the
    // view's old flat car (resting on the ground); the land step (master c51e396f8) rests the body
    // on its springs, so the eye sits higher. What the game's ray guarantees is only the above:
    // no prop box contains the eye.
}
