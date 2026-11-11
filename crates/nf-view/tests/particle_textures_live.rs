//! particle-textures: the emitter sprites draw with their def's texture (def+0x04) and blend
//! (def+0x3c) — `Emitter_LoadDefs` 0x17ab40 keeps the texture hashtable entry, `Emitter_Draw`
//! 0x17b708 path A passes it and `def+0x3c == 0` to `psiDrawParticleList` 0x122d50, which binds
//! it through `CreatePointerSpriteListColour` 0x119910 → `psiFont_SetTexture` 0x120880 and picks
//! ALPHA_1 0x48 (additive) for a set flag, 0x44 (normal) for a clear one [M disasm 0x17b80c-0x17b830,
//! 0x122da8-0x122dc4, 0x1214f8]. Opt-in via NF_ISO.

use nf_formats::level::Level;
use nf_view::scene::{Scene, TextureImage};

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

fn parse_level(iso: &str, name: &str) -> Level {
    let img = nf_formats::iso9660::IsoImage::open(iso).unwrap();
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).unwrap();
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case(name)).unwrap_or_else(|| panic!("no {name}"));
    let data = img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).unwrap();
    Level::parse(&data).unwrap()
}

/// The satchel burn sparks' def [D 0700004a.bin block 0x30]: def 0x0c00000a → texture 0x03000082
/// (a 64×64 map texture), tag>>8 = 0 → additive.
const SPARK_DEF: u32 = 0x0c00_000a;
const SPARK_TEX: u32 = 0x0300_0082;

/// Every block-0x30 def with a texture hash resolves to a scene texture whose source map texture
/// carries exactly that hashcode, and its pixels are that texture's decode (pixel-exact against a
/// fresh `TextureImage::from_texture` of the disc texture). Sister: the hashcode comparison can
/// fail — the scene holds other textures with different hashcodes, and a hash no map holds stays
/// unresolved.
#[test]
fn emitter_def_textures_resolve_to_their_map_textures() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    for name in ["0700004a.bin", "07000014.bin", "07000015.bin", "07000002.bin", "07000001.bin"] {
        let level = parse_level(&iso, name);
        let scene = nf_view::load(&iso, name, false).unwrap();
        let defs: Vec<_> = level.maps.iter().flat_map(|m| m.emitter_defs.iter()).collect();
        let mut resolved = 0;
        for d in &defs {
            let on_disc = level.maps.iter().any(|m| m.textures.iter().any(|t| t.hashcode == d.texture));
            let got = scene.particle_tex.get(&d.texture).copied();
            eprintln!("{name}: def {:08x} tex {:08x} blend {} size {} keys {:?} -> {:?}", d.hash, d.texture, d.blend, d.size, d.keys, got);
            if d.texture == 0 { assert!(got.is_none(), "{name}: a −1 texture hash binds nothing"); continue; }
            assert_eq!(got.is_some(), on_disc, "{name}: def {:08x}'s texture {:08x} resolves iff a map holds it", d.hash, d.texture);
            let Some(i) = got else { continue };
            let t = &scene.textures[i];
            let src = &level.maps[t.key.0].textures[t.key.1 as usize];
            assert_eq!(src.hashcode, d.texture, "{name}: the bound texture is the def's hash");
            let fresh = TextureImage::from_texture(t.key.0, src);
            assert!(fresh.rgba == t.rgba && fresh.width == t.width && fresh.height == t.height, "{name}: pixel-exact decode");
            resolved += 1;
        }
        // Sister: other scene textures carry other hashcodes, so the equality above can fail.
        assert!(scene.textures.iter().any(|t| level.maps[t.key.0].textures[t.key.1 as usize].hashcode != SPARK_TEX), "{name}: the hashcode check is not vacuous");
        assert!(!scene.particle_tex.contains_key(&0x0300_ffff), "{name}: a hash no def names is absent");
        eprintln!("{name}: {} defs, {resolved} textured", defs.len());
    }
}

fn spark_scene(iso: &str) -> Scene {
    let mut scene = nf_view::load(iso, "0700004a.bin", false).unwrap();
    let sim = scene.sim.as_mut().unwrap();
    let def = sim.drones.fx.defs().get(&SPARK_DEF).cloned().expect("0700004a carries the spark def");
    assert_eq!((def.texture, def.blend), (SPARK_TEX, 0), "the spark def [D]");
    // The satchel burn's one-shot burst (`Bullet_DoTrails` 0x12d890 → `Emitter_Create`), at Bond's
    // eye height in front of him, then two ticks so the slots spawn and fly.
    let h = sim.bond.head();
    sim.drones.fx.spawn_impact(SPARK_DEF, [h[0], h[1], h[2] + 1.0], [0.0, 1.0, 0.0]);
    for _ in 0..2 { sim.step_bond(Default::default(), false, false, false); }
    scene
}

/// The draw handoff through the real sim: the spark burst draws as ONE additive group bound to
/// 0x03000082, six vertices per live slot, each quad `size` wide (half-extent size/2 —
/// `particle_half_extent`), modulated by the slot rgba. Sister: the group's texture is not the
/// white fallback and not another scene texture.
#[test]
fn spark_burst_draws_textured_additive_quads() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let scene = spark_scene(&iso);
    let sim = scene.sim.as_ref().unwrap();
    let sprites: Vec<_> = sim.drones.fx.particle_sprites().into_iter().filter(|s| s.texture == SPARK_TEX).collect();
    assert!(!sprites.is_empty(), "the burst has live slots after two ticks");
    let (right, up) = ([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
    let groups = scene.particle_draws(right, up);
    let want = scene.particle_tex.get(&SPARK_TEX).copied();
    assert!(want.is_some(), "0x03000082 is a scene texture on 0700004a");
    let g: Vec<_> = groups.iter().filter(|(t, _, _)| *t == want).collect();
    assert_eq!(g.len(), 1, "one group per (texture, blend)");
    let (_, additive, verts) = g[0];
    assert!(*additive, "def+0x3c = 0 → additive (ALPHA_1 0x48)");
    assert_eq!(verts.len(), sprites.len() * 6, "two triangles per live slot");
    // Quad extents: the first sprite's six vertices span `size` along right and up.
    let q = &verts[..6];
    let xs = q.iter().map(|v| v.pos[0]);
    let (lo, hi) = xs.clone().fold((f32::MAX, f32::MIN), |(a, b), x| (a.min(x), b.max(x)));
    assert!((hi - lo - sprites[0].size).abs() < 1e-4, "the quad is `size` wide (half-extent size/2): {} vs {}", hi - lo, sprites[0].size);
    assert!(q.iter().all(|v| v.rgba == sprites[0].rgba), "the slot rgba modulates the texture");
    assert!(q.iter().any(|v| v.uv == [0.0, 0.0]) && q.iter().any(|v| v.uv == [1.0, 1.0]), "the whole texture spans the sprite");
    // Sister: the texture index names 0x03000082, not the fallback (None) nor the first scene texture.
    let t = &scene.textures[want.unwrap()];
    let level = parse_level(&iso, "0700004a.bin");
    assert_eq!(level.maps[t.key.0].textures[t.key.1 as usize].hashcode, SPARK_TEX);
    assert!(groups.iter().all(|(t, _, _)| t.is_some()), "no spark group falls back to white");
}

/// The 05 stone-wall P2K hit, the PS2 reference spot [L PCSX2, audits/render-1101-particles-ps2
/// imp05: Bond at (46.95, 29.6, 56.83) facing −z, hit (46.949, 30.300, 55.003)]: the impact burst
/// is def 0x0c00000f (material 10 [D EffectInfo+0x84]), spawns 5 a tick up to its 30 slots, and
/// every spark leaves OUT of the wall (live: every velocity z 2.0..3.5) — so all of them draw in
/// front of the face, 28 at a time (`psiDrawParticleList` masks the count to & !3 [M 0x122d50]).
/// Sister: the def's cone points along local +y, which only the row-weighted basis maps onto the
/// wall normal (the transposed form sent the whole burst behind the wall).
#[test]
fn the_05_wall_burst_flies_out_of_the_wall_like_the_ps2() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = nf_view::load(&iso, "07000005.bin", false).unwrap();
    let sim = scene.sim.as_mut().unwrap();
    nf_view::settle_start_draw(sim);
    sim.bond = nf_game::player::Bond::new([46.95, 29.6, 56.83], 3.142);
    for _ in 0..30 { sim.step_bond(Default::default(), false, false, false); }
    sim.step_bond(nf_game::player::Input { fire: true, ..Default::default() }, true, false, false);
    let mut seen = Vec::new();
    for _ in 0..8 {
        sim.step_bond(Default::default(), false, false, false);
        let e: Vec<_> = sim.drones.fx.emitters.iter().filter(|e| e.placement == usize::MAX).collect();
        seen.push(e.first().map(|e| e.parts.iter().filter(|p| p.age < p.life).count()).unwrap_or(0));
    }
    let e = sim.drones.fx.emitters.iter().find(|e| e.placement == usize::MAX).expect("the impact burst");
    assert_eq!(e.def, 0x0c00_000f, "material 10's spark def");
    eprintln!("live slots per tick {seen:?}, hit {:?}", e.pos);
    assert_eq!(&seen[..6], &[5, 10, 15, 20, 25, 30], "5 a tick up to the 30 slots [L dump +3..+12]");
    let wall_z = e.pos[2];
    let n = e.basis.unwrap()[1];
    assert!(n[2] > 0.99, "the hit normal faces Bond (+z): {n:?}");
    for p in e.parts.iter().filter(|p| p.age < p.life) {
        assert!(p.vel[2] > 1.0, "out of the wall: vel {:?}", p.vel);
        assert!(p.pos[2] >= wall_z, "in front of the face: {:?} vs z {wall_z}", p.pos);
    }
    let sprites: Vec<_> = sim.drones.fx.particle_sprites().into_iter().filter(|s| s.texture == 0x0300_00bb).collect();
    assert_eq!(sprites.len(), 28, "30 lit slots draw as 28 (count & !3)");
    assert!(sprites.iter().all(|s| s.additive && s.pos[2] >= wall_z), "additive, all in front of the wall");
}

/// The 0700004a charge under the watch, framed as the PS2 reference (the charge held 1.2 ahead of
/// Bond's eye, the view level) [L PCSX2, audits/render-1101-particles-ps2 sat4/log.txt: 16–17 burn
/// emitters of def 0x0c00000a and 41–43 live slots once the beam has held ~20 ticks, the sparks
/// leaving toward Bond, each slot 0.15–0.18 s]. One emitter per contact tick, 1 slot a tick up to
/// its 4 [M Bullet_DoTrails 0x12d890:467-468, Emitter_Update 0x17b080]. Sister: with the trigger
/// up, no burn emitter exists.
#[test]
fn the_charge_burn_keeps_the_ps2_spark_count() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = nf_view::load(&iso, "0700004a.bin", false).unwrap();
    let sim = scene.sim.as_mut().unwrap();
    sim.bond = nf_game::player::Bond::new([3.2, -7.0, -3.8], 0.0);
    let charge = |s: &nf_view::scene::Sim| s.drones.bullets.iter().position(|x| x.landed && x.family(&s.weapons) == 0x3b);
    for _ in 0..4000 { sim.step_bond(Default::default(), false, false, false); if charge(sim).is_some() { break; } }
    sim.step_bond(nf_game::player::Input { switch_to: Some(0x4e), ..Default::default() }, false, false, false);
    for _ in 0..60 { sim.step_bond(Default::default(), false, false, false); }
    // The pull comes once the watch is drawn, as a player's would: `Player_WeaponFiring` runs every
    // frame [M Player_Update 0x193600:413] and a press made while the draw holds the view state
    // spends its edge there (the timer arms, the state gate refuses the shot, and a held trigger
    // never re-arms [M 0x1a5460:601-648]), so a trigger held from inside the draw never fires.
    for _ in 0..240 {
        if sim.gun.weapon == 0x4e && !sim.gun.refire_blocked { break; }
        sim.step_bond(Default::default(), false, false, false);
    }
    assert!(sim.gun.weapon == 0x4e && !sim.gun.refire_blocked, "the watch laser is drawn and ready");
    let yaw = 0.405f32;
    let mut counts = Vec::new();
    for t in 0..60 {
        sim.bond.yaw = yaw;
        sim.bond.pitch = 0.0;
        let h = sim.bond.head();
        let i = charge(sim).expect("the landed charge");
        sim.drones.bullets[i].pos = [h[0] + yaw.sin() * 1.2, h[1] - 0.05, h[2] + yaw.cos() * 1.2];
        let fire = t < 50;
        sim.step_bond(Default::default(), fire, false, false);
        let em: Vec<_> = sim.drones.fx.emitters.iter().filter(|e| e.def == SPARK_DEF).collect();
        // The dump's "live" = age < life (a slot past its life still draws that tick).
        let lit = em.iter().map(|e| e.parts.iter().filter(|p| p.age < p.life).count()).sum::<usize>();
        if t == 0 { assert!(em.len() <= 1, "the first contact tick"); }
        if (35..50).contains(&t) {
            counts.push((em.len(), lit));
            for e in &em {
                let n = e.basis.unwrap()[1];
                for p in e.parts.iter().filter(|p| p.age < p.life) {
                    assert!(p.vel[0] * n[0] + p.vel[1] * n[1] + p.vel[2] * n[2] > 0.0, "sparks leave the face");
                    assert!(p.life > 0.149 && p.life < 0.184, "life 1/6 ± 1/60 s [D]");
                }
            }
        }
    }
    eprintln!("(emitters, lit) per tick {counts:?}");
    assert!(counts.iter().all(|&(e, l)| (15..=18).contains(&e) && (37..=47).contains(&l)), "the PS2's 16–17 emitters / 41–43 live slots: {counts:?}");
    // Sister: 10 ticks after the trigger lets go the last bursts are spent.
    for _ in 0..12 { sim.step_bond(Default::default(), false, false, false); }
    assert_eq!(sim.drones.fx.emitters.iter().filter(|e| e.def == SPARK_DEF).count(), 0, "no beam, no burn sparks");
}
