//! cutscene-props: in 07000014's control-room scene 0x06000833 every actor who passes the door
//! passes it while the scene's door leaves stand OPEN — nobody walks through a closed leaf.
//!
//! [D 07800014.bin 0x06000833, `nf-extract nis`]: camera op at s0 t0; the actors are anim streams
//! s1 (skin 0x05000087), s3 (0x05000090), s5 (0x05000093), s18 (0x0500005e); the door is the two
//! unparented entity streams s10/s11 (hashes 0x02000891/0x02000892, obj_type 0xff) keyed on the
//! plane z 49.5721 at y 1.2965 — leaf A x 27.7915 closed → 27.0697 open, leaf B 27.7895 → 28.6815
//! (cutscene_door_07000014_live pins the leaves themselves). The placed pair #116/#117 at
//! (27.788, 1.203, 49.671) hides from the scene camera (glist 0x40000 [M Control_SetGList
//! 0x134960]) — the leaves are the only door drawn.
//!
//! This pins the scene-level result on the drawn poses: every tick a posed bone of any actor stands
//! inside the leaves' slab (their glist z extent) within the door frame, the leaves are apart —
//! nobody passes a closed door. Sister: the closed keys' leaves overlap (no opening) and fail it.
//! Measured on the keys [D]: Kiko (s18, skin 0x0500005e) starts 0.39 behind the closed door,
//! takes her first step at anim frame 0x14f [D 0x06000857 op 4] and enters the slab at t349,
//! when the opening is 0.24 (the leaves started at t252 and are fully apart by t549).
//!
//! Opt-in via `NF_ISO`.

use nf_game::player::Input;

const SCENE: u32 = 0x0600_0833;
const DOOR_Z: f32 = 49.5721;


/// The two leaves' matrices (leaf A 0x02000891, leaf B 0x02000892).
fn leaves(sim: &nf_view::scene::Sim) -> Option<([[f32; 4]; 4], [[f32; 4]; 4])> {
    let (mut a, mut b) = (None, None);
    for o in sim.drones.scripts.objects.list.iter().flatten() {
        if let nf_game::nis::Obj::Entity { hash, matrix, .. } = o {
            if *hash == 0x0200_0891 { a = Some(*matrix); }
            if *hash == 0x0200_0892 { b = Some(*matrix); }
        }
    }
    Some((a?, b?))
}

fn read_archive(iso: &str, archive: &str) -> Vec<u8> {
    let img = nf_formats::iso9660::IsoImage::open(iso).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case(archive)).expect("archive");
    img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read")
}

#[test]
fn every_actor_crosses_the_control_room_door_while_it_stands_open() {
    let Ok(iso) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set — skipping"); return };
    std::env::set_var("NF_SILENT", "1");
    // The leaves' glist boxes [D 07000014.bin]: the inner edge of each leaf is its matrix x plus
    // the box's x bound on the doorway side.
    let data = read_archive(&iso, "07000014.bin");
    let lv = nf_formats::level::Level::parse(&data).expect("parse 07000014");
    let bounds = |h: u32| lv.maps.iter().flat_map(|m| m.objects.iter()).find(|o| o.hashcode == h).map(|o| (o.min, o.max))
        .unwrap_or_else(|| panic!("leaf glist {h:08x} [D]"));
    let (a_box, b_box) = (bounds(0x0200_0891), bounds(0x0200_0892));
    eprintln!("leaf A box {a_box:?}, leaf B box {b_box:?}");

    let mut scene = nf_view::load(&iso, "07000014.bin", false).expect("load 07000014");
    // The vent-drop touch #301 fires the scene on the real path (cutscene_door_07000014_live's spot).
    scene.sim.as_mut().unwrap().bond.pos = [40.7, 2.6, 39.0];
    if std::env::var_os("NF_PROBE").is_some() {
        let sim = scene.sim.as_ref().unwrap();
        for h in [0x0600_0857u32, 0x0600_0854, 0x0600_0856] {
            if let Some(sc) = sim.scripts.get(&h) {
                eprintln!("script {h:08x}: length {} looping {}", sc.length, sc.looping);
                for c in &sc.commands { eprintln!("   op {} args {:x?}", c.op, c.args); }
            } else { eprintln!("script {h:08x}: not loaded"); }
        }
    }
    let mut t = 0u32;
    // (stream, tick, bone x, opening lo, opening hi) for every actor bone inside the door slab.
    let mut in_door: Vec<(u8, u32, f32, f32, f32)> = Vec::new();
    let mut worst: Option<(u8, u32, f32, f32, f32)> = None;
    for _ in 0..1700u32 {
        scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false);
        let sim = scene.sim.as_ref().unwrap();
        if !sim.disabled { if t > 0 { break } else { continue } }
        t += 1;
        let Some((la, lb)) = leaves(sim) else { continue };
        // The leaves key translation only (identity rotation on every key [D]).
        assert!((la[0][0] - 1.0).abs() < 1e-3 && (lb[0][0] - 1.0).abs() < 1e-3, "t {t}: a leaf rotated — the edge rule assumes slide-only keys");
        // Leaf A slides −x, leaf B +x: the opening is A's max-x edge .. B's min-x edge.
        let (open_lo, open_hi) = (la[3][0] + a_box.1[0], lb[3][0] + b_box.0[0]);
        // The opening counts only where the doorway is: both leaves on the door plane, inside the
        // frame (a leaf thrown elsewhere leaves the real doorway shut by nothing drawn there).
        let on_door = |m: &[[f32; 4]; 4]| (m[3][2] - DOOR_Z).abs() < 0.02 && (26.9..=28.7).contains(&m[3][0]);
        let (open_lo, open_hi) = if on_door(&la) && on_door(&lb) { (open_lo, open_hi) } else { (f32::MAX, f32::MIN) };
        let fs = scene.frame_state_for(None, 0.0, nf_view::scene::ViewOpts::default(), false);
        for (ci, c) in scene.characters.iter().enumerate() {
            let Some((h, si)) = c.script else { continue };
            if h != SCENE { continue }
            let Some(Some((_, world, m))) = fs.posed.get(ci) else { continue };
            for bone in world.iter() {
                let p = [bone[3][0], bone[3][1], bone[3][2]];
                let w = [0, 1, 2].map(|k| m[0][k] * p[0] + m[1][k] * p[1] + m[2][k] * p[2] + m[3][k]);
                // A bone inside the leaves' own slab (their glist z extent around the plane),
                // inside the frame (x 26.9..28.7, the closed leaves' span), under the lintel.
                let dz = w[2] - DOOR_Z;
                if dz > a_box.0[2].min(b_box.0[2]) && dz < a_box.1[2].max(b_box.1[2]) && (26.9..=28.7).contains(&w[0]) && w[1] < 3.3 {
                    in_door.push((si, t, w[0], open_lo, open_hi));
                    let over = (open_lo - w[0]).max(w[0] - open_hi);
                    if worst.map_or(true, |(_, _, wx, lo, hi)| over > (lo - wx).max(wx - hi)) { worst = Some((si, t, w[0], open_lo, open_hi)); }
                }
            }
        }
    }
    assert!(t >= 1300, "the scene played (1400 ticks [D]), got {t}");
    assert!(!in_door.is_empty(), "no actor passed the control-room doorway — the scene's walk-through [D] did not run");
    let first = in_door.first().unwrap();
    eprintln!("first doorway bone: s{} at t {} (opening {:.3}..{:.3}); worst {worst:?}", first.0, first.1, first.3, first.4);
    // Every tick an actor's bone stands in the leaves' slab, the leaves are apart (an opening
    // exists): nobody passes a closed door.
    for &(si, tick, x, lo, hi) in &in_door {
        assert!(hi - lo > 0.0,
            "t {tick}: an s{si} bone at x {x:.3} is in the door while the leaves are closed (opening {lo:.3}..{hi:.3}) — the actor walks through the door");
    }
    // Sister: the CLOSED leaves (the [D] t0 keys 27.7915 / 27.7895) overlap — no opening — so the
    // same check fails on them.
    let (closed_lo, closed_hi) = (27.7915 + a_box.1[0], 27.7895 + b_box.0[0]);
    assert!(!(closed_hi - closed_lo > 0.0), "sister: a closed door must fail the doorway check");
}
