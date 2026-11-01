//! The GS alpha range in the additive draws. The GS reads As on the 0x80 = 1.0 scale and keeps
//! values above it: MODULATE gives As = At·Av/0x80 clamped to 0xFF, so ALPHA_1 0x48 (Cs·As + Cd)
//! adds up to 255/128 ≈ 2× the colour. The port's SrcAlpha blend factor stopped at 1.0.
//!
//! Which draws feed Av > 0x80 [M/D]:
//! - the emitter particles: `Emitter_LoadDefs` 0x17ab40 halves the key rgb but stores the key alpha
//!   as read, `Emitter_Update` 0x17b080 writes `(char)(int)` of the keyed colour into the slot, and
//!   `psiDrawParticleList` 0x122d50 → `CreatePointerSpriteListColour` 0x119910 unpacks the slot
//!   bytes (V4-8 at 0x1d) that the colour-list microcode copies straight into RGBAQ (vu1dis lines
//!   0x4a7 `LQ vf26, 0(vi04)`, 0x4a9/0x4ac `SQ vf26`) — no ITOF, no scale. TEX0 TCC 1 / TFX 0.
//!   Every block-0x30 def on the disc keys alpha 255 [D];
//! - nothing else: world/character/corona mesh vertex alpha never exceeds 0x80 [D, scan below];
//!   corona and object tweaks are byte/255 multipliers (`psiSetTweakARGB` 0x124770,
//!   `View_SetupRenderModes_Tweak` 0x1e5888, `Corona_Update` 0x18da78 caps obj+0x106 at 0xFF);
//!   HUD sprites halve their alpha (`psiDrawSprites` 0x120e68 `A >> 1`); the rain/snow list uses
//!   the constant RGBA (0x3f, 0x3f, 0x3f, 0x7f) (`CreatePointerSpriteList` 0x11a160).
//!
//! Opt-in via NF_ISO; the GPU test also needs an adapter.

use nf_formats::level::Level;
use nf_view::gpu;
use nf_view::scene::Vertex;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

fn parse_level(iso: &str, name: &str) -> Option<Level> {
    let img = nf_formats::iso9660::IsoImage::open(iso).unwrap();
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).unwrap();
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case(name))?;
    let data = img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).unwrap();
    Level::parse(&data).ok()
}

/// [D] Every emitter def keys alpha 255 at its first key while its rgb stays on the halved
/// 0x80 scale (≤ 127.5), so a fresh sprite reaches the GS with Av = 0xFF.
#[test]
fn emitter_keys_feed_alpha_above_0x80() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut defs = 0;
    for name in ["07000002.bin", "07000014.bin", "07000015.bin", "07000046.bin"] {
        let level = parse_level(&iso, name).expect(name);
        for d in level.maps.iter().flat_map(|m| m.emitter_defs.iter()) {
            defs += 1;
            assert_eq!(d.keys[0][3], 255.0, "{name}: def {:08x} starts at alpha 255", d.hash);
            assert!(d.keys.iter().all(|k| k[0] <= 127.5 && k[1] <= 127.5 && k[2] <= 127.5), "{name}: def {:08x} rgb is halved", d.hash);
        }
    }
    assert!(defs > 0, "the block-0x30 defs parse");
    // The slot byte the draw sees: 07000014's emitters through the real sim with their ON channels set.
    let mut scene = nf_view::load(&iso, "07000014.bin", false).unwrap();
    let chans: Vec<u16> = scene.sim.as_ref().unwrap().drones.fx.emitters.iter().map(|e| e.ch_on).filter(|&c| c != 0).collect();
    for c in chans { scene.sim.as_mut().unwrap().drones.channels.set(c as u32, 1, 0); }
    let mut max_a = 0u8;
    for _ in 0..6 {
        if let Some(sim) = scene.sim.as_mut() { for _ in 0..100 { sim.step_bond(Default::default(), false, false, false); } }
        let q = scene.sim.as_ref().unwrap().particle_quads([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        max_a = max_a.max(q.iter().map(|v| v.rgba[3]).max().unwrap_or(0));
    }
    assert!(max_a > 0x80, "07000014's particle quads carry alpha above 0x80 (max {max_a:#x})");
}

/// [D] The level meshes (world, characters, coronas, view models) never carry vertex alpha above 0x80,
/// so the range fix changes no additive mesh. Sister: 0x80 itself occurs, so the scan reads alpha.
#[test]
fn mesh_vertex_alpha_stays_at_or_below_0x80() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    for name in ["07000005.bin", "07000006.bin", "07000015.bin"] {
        let sc = nf_view::load(&iso, name, false).unwrap();
        let mut max = [0u8; 5];
        for m in &sc.materials {
            for &i in &sc.indices[m.first_index as usize..(m.first_index + m.index_count) as usize] {
                let b = (m.draw.blend as usize).min(4);
                max[b] = max[b].max(sc.vertices[i as usize].rgba[3]);
            }
        }
        assert!(max.iter().all(|&a| a <= 0x80), "{name}: vertex alpha above 0x80 per blend: {max:?}");
        assert_eq!(max[1], 0x80, "{name}: the additive draws reach 0x80 (sister)");
    }
}

/// The additive pipeline through the real GPU, via the particle pass (pass 5, blend 1): a full-screen
/// sprite over the cleared target, the 1×1 white texture (Ct = 1.0, At = 0x80).
#[test]
fn additive_sprites_keep_the_gs_alpha_range_above_0x80() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut sc = nf_view::load(&iso, "07000005.bin", false).expect("07000005");
    sc.materials.clear();
    sc.snow_texture = None;
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
    let Some((device, queue)) = pollster::block_on(async {
        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions::default()).await.ok()?;
        adapter.request_device(&Default::default()).await.ok()
    }) else { eprintln!("no GPU adapter - skipping"); return };
    let mut r = gpu::Renderer::new(&device, &queue, wgpu::TextureFormat::Rgba8Unorm, &sc, false, false);
    const W: u32 = 32;
    const H: u32 = 32;
    let mut render = |quads: &[[u8; 4]]| -> [f32; 3] {
        let mut v: Vec<Vertex> = Vec::new();
        for &rgba in quads {
            let p = |x: f32, y: f32| Vertex { pos: [x, y, 0.5], uv: [0.5, 0.5], rgba, nrm: [0; 4] };
            v.extend_from_slice(&[p(-1.0, 1.0), p(1.0, 1.0), p(1.0, -1.0), p(-1.0, 1.0), p(1.0, -1.0), p(-1.0, -1.0)]);
        }
        let format = wgpu::TextureFormat::Rgba8Unorm;
        let target = device.create_texture(&wgpu::TextureDescriptor {
            label: None, size: wgpu::Extent3d { width: W, height: H, depth_or_array_layers: 1 }, mip_level_count: 1, sample_count: 1,
            dimension: wgpu::TextureDimension::D2, format, usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC, view_formats: &[],
        });
        let view = target.create_view(&Default::default());
        let depth = gpu::depth_view(&device, W, H);
        let id = [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]];
        let mut enc = r.render(&device, &queue, &view, &depth, id, id, [0.0; 3], 0, &v, &gpu::HudFrame::default(), None, None);
        let row = (4 * W).next_multiple_of(256);
        let buf = device.create_buffer(&wgpu::BufferDescriptor { label: None, size: (row * H) as u64, usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ, mapped_at_creation: false });
        enc.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo { texture: &target, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
            wgpu::TexelCopyBufferInfo { buffer: &buf, layout: wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(row), rows_per_image: Some(H) } },
            wgpu::Extent3d { width: W, height: H, depth_or_array_layers: 1 });
        queue.submit([enc.finish()]);
        buf.map_async(wgpu::MapMode::Read, .., |r| r.expect("map"));
        device.poll(wgpu::PollType::wait_indefinitely()).expect("poll");
        let data = buf.get_mapped_range(..).expect("map range");
        let o = (H as usize / 2) * row as usize + (W as usize / 2) * 4;
        let px = [data[o] as f32 / 255.0, data[o + 1] as f32 / 255.0, data[o + 2] as f32 / 255.0];
        drop(data);
        px
    };
    let close = |got: [f32; 3], want: [f32; 3], what: &str| {
        for k in 0..3 { assert!((got[k] - want[k]).abs() <= 2.5 / 255.0, "{what}: channel {k} got {:.4} want {:.4}", got[k], want[k]); }
    };
    let g = gpu::GAIN;
    let b = render(&[]);
    let want = |term: f32| [0, 1, 2].map(|k| (b[k] + g * term).min(1.0));

    // Av = 0xFF (an emitter's fresh sprite), Cv = 0x20: Cs = 0.25, As = 255/128 → Cd + gain·0.25·1.992.
    let got = render(&[[0x20, 0x20, 0x20, 0xff]]);
    close(got, want(0.25 * 255.0 / 128.0), "additive, As = 0xFF");
    // Sister (can fail): the clamped factor gave Cd + gain·0.25 — 0.28 lower.
    assert!(got[1] > (b[1] + g * 0.25) + 0.2, "As above 0x80 brightens past the 1.0 clamp ({:.3} over {:.3})", got[1], b[1]);
    // Av = 0xC0: As = 1.5.
    let got = render(&[[0x20, 0x20, 0x20, 0xc0]]);
    close(got, want(0.25 * 1.5), "additive, As = 0xC0");
    // Av ≤ 0x80 is unchanged: 0x40 → As = 0.5; 0x80 → As = 1.0.
    let got = render(&[[0x20, 0x20, 0x20, 0x40]]);
    close(got, want(0.25 * 0.5), "additive, As = 0x40");
    let got = render(&[[0x20, 0x20, 0x20, 0x80]]);
    close(got, want(0.25), "additive, As = 0x80");
    // Two stacked 0xFF sprites saturate (COLCLAMP) instead of wrapping.
    let got = render(&[[0x40, 0x40, 0x40, 0xff], [0x40, 0x40, 0x40, 0xff]]);
    close(got, [1.0; 3], "two bright sprites clamp at 1.0");
}

/// The premultiplied term applies to the two Cs·As blend shapes only (0x48 → 1, 0x42 → 2); normal
/// (0), FIX-0 (3) and destination-alpha (4) keep their own factors.
#[test]
fn premul_covers_the_cs_as_blends_only() {
    assert_eq!([0u8, 1, 2, 3, 4].map(gpu::premul), [0.0, 1.0, 1.0, 0.0, 0.0]);
}
