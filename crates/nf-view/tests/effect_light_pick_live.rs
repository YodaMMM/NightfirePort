//! effect-light-pick live test (NF_ISO, 07000005): the game picks an effect object's lights at the
//! OBJECT — `View_DrawObjList` 0x1e69a0:73-81 runs `Lights_CalcClosestLights(obj+0x80,
//! *(u16*)(obj+0xF0), &count)` + `psiLight_SetLights` for every drawn object — while the port's
//! effect draws picked once per mesh group at the load-time identity mesh centre, the world origin.
//! This walks nothing but the load's own start position (no teleport), fires through the real input
//! path (`Sim::step_bond`, the same shot the player's fire press runs), and pins the sim-side
//! per-object pick (`nf_game::effect::effect_light_picks`): a debris piece at the wall picks the
//! impact light sitting at the piece first — not whatever is nearest the origin. docs/atlas/effect.md.

use nf_formats::iso9660::IsoImage;
use nf_game::player::Input;

/// The level archive's mesh records by hashcode: glist local centre, radius and the +0x44 flags
/// word (`MapObject`) — the data `Control_SetGList` 0x134960 copies to obj+0xF0 and
/// `Control_BuildWorldSph` 0x133070 builds obj+0x80 from.
fn mesh_spheres(iso: &str, id: &str) -> std::collections::HashMap<u32, ([f32; 3], f32, u16)> {
    let img = IsoImage::open(iso).expect("the disc image opens");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name == format!("{id}.bin")).expect("the level entry");
    let data = img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("the level bin");
    let level = nf_formats::level::Level::parse(&data).expect("the level parses");
    let mut out = std::collections::HashMap::new();
    for m in &level.maps {
        for o in &m.objects { out.entry(o.hashcode).or_insert((o.centre, o.radius, o.flags as u16)); }
    }
    out
}

#[test]
fn a_debris_piece_picks_the_lights_at_itself_not_the_origin() {
    let Some(iso) = std::env::var("NF_ISO").ok() else { return };
    let spheres = mesh_spheres(&iso, "07000005");
    let mut scene = nf_view::load(&iso, "07000005.bin", false).unwrap();
    // hud-start: a load holds nothing (0x47 [M 0x1a7ef8:137]) - settle the start draw first.
    nf_view::settle_start_draw(scene.sim.as_mut().unwrap());
    // The debris mesh the shot's wall material feeds, and whether its mask admits kind 1: the
    // census [D archive] says 0x020/0x021/0x221 meshes do (bit 0x100 clear), 0x321/0x329 do not.
    let admits_kind1 = |mesh: u32| spheres.get(&mesh).map_or(false, |(_, _, f)| f & 0x100 == 0);
    let shot = {
        let sim = scene.sim.as_mut().unwrap();
        let surfaces = sim.data().surfaces.expect("07000005 carries the EffectInfo table");
        let eye0 = [sim.bond.pos[0], sim.bond.pos[1] + 1.2, sim.bond.pos[2]];
        // A wall whose material spawns kind-1-admitting debris, found by sweeping the real start
        // position's view fan (the impacts_live pattern — the shot itself is the input path).
        // The CLOSEST such wall wins: the second shot's muzzle light (radius = weapon_data+0x58,
        // the P2K's 7.0 [L live read]) must reach the first shot's debris, which outlives its own
        // shot's one-tick muzzle light [M Player_MuzzleFlash 0x1a4298: life +0x20 = 1].
        let mut aim: Option<(f32, f32, u32, u32, f32)> = None; // (yaw, pitch, material, debris mesh, t)
        'fan: for pitch in [-0.05f32, -0.2] { for k in -30..=30i32 {
            let yaw = k as f32 * 0.08;
            let dir = [yaw.sin() * pitch.cos(), pitch.sin(), yaw.cos() * pitch.cos()];
            let end = [eye0[0] + dir[0] * 30.0, eye0[1] + dir[1] * 30.0, eye0[2] + dir[2] * 30.0];
            let Some((t, _, mat)) = sim.data().world.segment_surface(eye0, end, false) else { continue };
            if t >= 0.98 { continue; }
            let Some(rec) = surfaces.effects.get((mat & 0x3f) as usize) else { continue };
            if rec.debris_mesh != 0 && admits_kind1(rec.debris_mesh) && rec.debris_enable != 0 {
                let better = aim.as_ref().map_or(true, |(_, _, _, _, t0)| t < *t0);
                if better { aim = Some((yaw, pitch, (mat & 0x3f) as u32, rec.debris_mesh, t)); }
            }
        } }
        let (yaw, pitch, mat, want_mesh, _t) = aim.expect("a wall with kind-1-admitting debris near the 07000005 start");
        sim.bond.yaw = yaw;
        sim.bond.pitch = pitch.asin();
        sim.bond.eye = 0.0;
        // Fire through the real input path until a piece of an admitting mesh exists (the semi-auto
        // cycle gates the shot; the carried pool is topped up like impacts' 40-round run). The
        // muzzle's spread may land the shot on a neighbouring material — accept any piece whose own
        // mesh admits kind 1 (the test's premise), whatever surface made it.
        let mut piece = None;
        let mut saw_debris = 0usize;
        for _ in 0..48 {
            let ty = sim.weapons.ammo_type(sim.gun.weapon as usize);
            if let Some(n) = sim.drones.inventory.carried.get_mut(ty) { *n = (*n).max(500); }
            sim.step_bond(Input { fire: true, ..Input::default() }, true, false, false);
            for _ in 0..20 {
                sim.step_bond(Input::default(), false, false, false);
                saw_debris = saw_debris.max(sim.drones.effects.debris.len());
                if let Some(d) = sim.drones.effects.debris.iter().find(|d| admits_kind1(d.mesh)) { piece = Some(d.clone()); break; }
            }
            if piece.is_some() { break; }
        }
        let _ = want_mesh;
        let mut piece = piece.unwrap_or_else(|| panic!("no kind-1-admitting debris in 48 shots (probed mesh 0x{want_mesh:08x} from material {mat}, peak debris count {saw_debris})"));
        // Settle the semi-auto's cycle, then fire AGAIN: the muzzle light (kind 1, life 1 tick
        // [M 0x1a4298]) is live on the fire tick, while the first shot's debris (life ≈ 55 ticks
        // [M 0x1aaae0..0x1aab08]) still lies near the wall — the piece and a kind-1 light coexist.
        for _ in 0..24 { sim.step_bond(Input::default(), false, false, false); }
        let ty = sim.weapons.ammo_type(sim.gun.weapon as usize);
        if let Some(n) = sim.drones.inventory.carried.get_mut(ty) { *n = (*n).max(500); }
        let before = sim.drones.effects.debris.iter().find(|d| d.mesh == piece.mesh).cloned();
        sim.step_bond(Input { fire: true, ..Input::default() }, true, false, false);
        // The piece's live record after the fire tick (it may have fallen a little).
        piece = sim.drones.effects.debris.iter().find(|d| d.mesh == piece.mesh)
            .cloned()
            .or(before)
            .expect("the first shot's debris outlives the second shot's fire tick");
        (piece, want_mesh)
    };
    let (piece, want_mesh) = shot;
    // The per-object pick at the tick the piece exists: the same resolved list frame_lights builds.
    let (resolved, picks) = {
        let sim = scene.sim.as_ref().unwrap();
        let mut resolved = sim.drones.lights.resolve(&|i: u32| sim.drones.searchlights.parts.get(i as usize).map(|p| p.pos).unwrap_or([0.0; 3]));
        for l in resolved.lights.iter_mut() {
            if let Some(t) = l.track {
                if let Some(p) = sim.drones.searchlights.parts.get(t as usize) {
                    if p.light.is_none() { l.active = false; }
                }
            }
        }
        let picks = nf_game::effect::effect_light_picks(&sim.drones, &resolved, &|h| spheres.get(&h).copied());
        (resolved, picks)
    };
    let d = picks.iter().find(|p| p.kind == nf_game::effect::EFFECT_PICK_DEBRIS && p.mesh == piece.mesh && p.slot as usize == piece.draw.slot as usize)
        .expect("the live debris slot carries a per-object pick");
    // The sphere is the piece's own: centre = pos, radius = mesh radius × obj+0xE8
    // (`Control_BuildWorldSph` 0x133070 × the 0.4 + FRand(0.1) scale [M 0x1aaad8]).
    let (centre, mesh_r, mesh_f) = spheres[&piece.mesh];
    let _ = centre;
    assert!((d.sphere[0] - piece.pos[0]).abs() < 0.01 && (d.sphere[1] - piece.pos[1]).abs() < 0.01 && (d.sphere[2] - piece.pos[2]).abs() < 0.01,
        "the pick sphere centres on the piece, got {:?} want {:?}", &d.sphere[..3], &piece.pos);
    let want_r = mesh_r * piece.scale;
    assert!((d.sphere[3] - want_r).abs() < 1e-4, "the pick radius is the mesh radius × scale, got {} want {want_r}", d.sphere[3]);
    assert_eq!(d.flags, mesh_f | 0x20, "the mesh word | Debris_Create's 0x20 [M 0x1aaa00:53]");
    // The piece's own mask admits kind 1 (the fan picked the material for it) — the pick must
    // select real lights, and the first one is the light at the piece: the shot's impact light
    // (`GunImp_Update` 0x1320d8:351/370, radius 9) sits at the wall the piece came from.
    assert!(mesh_f & 0x100 == 0, "the test's premise: mesh 0x{:08x} admits kind 1", piece.mesh);
    assert!(d.lights[0].is_some(), "a piece with lights in range picks one");
    let l0 = resolved.lights[d.lights[0].unwrap() as usize];
    let dist = ((l0.pos[0] - piece.pos[0]).powi(2) + (l0.pos[1] - piece.pos[1]).powi(2) + (l0.pos[2] - piece.pos[2]).powi(2)).sqrt();
    assert!(dist < 9.0 + d.sphere[3] + 0.5, "the piece's first light sits at the piece ({} units)", dist);
    // The old behaviour's witness: the same list picked at the world origin (the old per-group
    // centre) does not give the piece's answer — either nothing in range there, or a different
    // first light farther from the piece.
    let mut obuf = [(0usize, 0.0f32); 30];
    let on = resolved.closest_into([0.0, 0.0, 0.0, want_r], mesh_f as u32 | 0x20, &mut obuf);
    let old_first = obuf[..on].first().map(|&(i, _)| i);
    match old_first {
        None => {}
        Some(i) => {
            let lp = resolved.lights[i].pos;
            let od = ((lp[0] - piece.pos[0]).powi(2) + (lp[1] - piece.pos[1]).powi(2) + (lp[2] - piece.pos[2]).powi(2)).sqrt();
            assert!(od > dist + 0.5, "the origin pick's first light ({od} from the piece) is not the piece's own ({dist})");
        }
    }
    // And the port's picker, fed the sim-side sphere, reproduces the game's answer (the draw-side
    // consumption the draw wires): same list, same sphere → same two lights, nearest first.
    let (lp, _) = resolved.pick2(d.sphere, d.flags as u32);
    if let Some(i) = d.lights[0] {
        assert_eq!(lp[0], resolved.lights[i as usize].pos, "the uploaded first light is the pick's first light");
    }
}
