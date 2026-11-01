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
