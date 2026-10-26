//! Scratch probe (level-14-lid-and-climb): the created lid slabs' world-space collision AABBs,
//! against the east route's crawl/drop spots. Also the static world's down-segments at the same
//! spots (what the slot looks like without the dynamic slabs). Assertion-free.
fn main() {
    let iso_path = std::env::var("NF_ISO").expect("NF_ISO");
    let mut scene = nf_view::load(&iso_path, "07000014.bin", false).unwrap();
    let sim = scene.sim.as_mut().unwrap();

    sim.step_bond(nf_game::player::Input::default(), false, false, false);
    println!("-- slab entities (kind 0x31, hash 02000750) world AABBs --");
    let mut boxes: Vec<([f32; 3], [f32; 3])> = Vec::new();
    for o in sim.drones.scripts.objects.list.iter().flatten() {
        if let nf_game::nis::Obj::Entity { hash, kind, matrix, .. } = o {
            if *hash == 0x0200_0750 && *kind == 0x31 {
                let Some(tris) = sim.world.shape(*hash) else { continue };
                let mut mn = [f32::MAX; 3];
                let mut mx = [f32::MIN; 3];
                for t in tris {
                    for p in t {
                        let w = nf_formats::placement::transform(matrix, *p);
                        for k in 0..3 {
                            mn[k] = mn[k].min(w[k]);
                            mx[k] = mx[k].max(w[k]);
                        }
                    }
                }
                println!(
                    "  slab at mat ({:.3}, {:.3}, {:.3}): world AABB x {:.3}..{:.3}  y {:.3}..{:.3}  z {:.3}..{:.3}",
                    matrix[3][0], matrix[3][1], matrix[3][2], mn[0], mx[0], mn[1], mx[1], mn[2], mx[2]
                );
                boxes.push((mn, mx));
            }
        }
    }

    let spots: [(&str, [f32; 3]); 6] = [
        ("crawl target (41.0, 2.4, 63.9)", [41.0, 2.4, 63.9]),
        ("slot mid (43.0, 2.4, 63.0)", [43.0, 2.4, 63.0]),
        ("slot east (45.2, 2.4, 63.6)", [45.2, 2.4, 63.6]),
        ("north lip (41.0, 2.4, 62.0)", [41.0, 2.4, 62.0]),
        ("mouth west lid (40.776, 2.372, 42.159)", [40.776, 2.372, 42.159]),
        ("mouth east lid (45.286, 2.372, 42.159)", [45.286, 2.372, 42.159]),
    ];
    for (label, p) in spots {
        let in_slab = boxes.iter().any(|(mn, mx)| {
            (0..3).all(|k| p[k] >= mn[k] - 0.35 && p[k] <= mx[k] + 0.35)
        });
        let down_static = sim.world.segment(p, [p[0], p[1] - 5.0, p[2]]);
        println!(
            "  {label}: inside-a-slab(+0.35 pad) {in_slab}; static down-seg t {:?}",
            down_static
        );
    }
}
