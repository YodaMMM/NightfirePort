//! The HUD sprites' blend through the real GPU pipelines, against the GS's own order of
//! operations [M psiDrawSprites 0x120e68 / FillSpriteData 0x120cf8; CreateSqueezeFrame 0x11b140
//! from psiPreDraw 0x121590]:
//! - MODULATE clamps the sprite colour first: Cs = min(Ct·Cv / 0x80, 0xff); As = At·Av / 0x80;
//! - ALPHA_1 blends into the frame: 0x44 (Cs − Cd)·As + Cd, 0x48 Cs·As + Cd, 0x42 Cd − Cs·As;
//! - the next frame's squeeze copy scales the FINISHED frame by SqueezeColourIn / 0x80
//!   (`gpu::GAIN`), so the HUD's term arrives as gain·Cs·As over a target holding gain·Cd.
//! The old pipelines applied the gain to Cs before the clamp (min(Ct·Cv·gain, 1)·As), so a
//! bright translucent sprite came out up to 1/gain dimmer than the PS2's.
//!
//! Opt-in via NF_ISO (the renderer is built from a real scene; the world is emptied so only the
//! clear colour sits under the HUD) and needs a GPU adapter.

use nf_view::gpu::{self, HudFrame, HudVertex};

const W: u32 = 32;
const H: u32 = 32;

fn quad(c: [f32; 4]) -> [HudVertex; 6] {
    let p = |x: f32, y: f32| HudVertex { p: [x, y], uv: [0.0, 0.0], c };
    [p(-1.0, 1.0), p(1.0, 1.0), p(1.0, -1.0), p(-1.0, 1.0), p(1.0, -1.0), p(-1.0, -1.0)]
}

/// One frame with these (blend, colour) full-screen quads over the cleared target, in order;
/// returns the centre pixel's RGB bytes. Texture slot 0 is the renderer's 1×1 white (Ct = 1.0).
fn render(r: &mut gpu::Renderer, device: &wgpu::Device, queue: &wgpu::Queue, quads: &[(u8, [f32; 4])]) -> [f32; 3] {
    let mut hud = HudFrame::default();
    for (blend, c) in quads {
        let first = hud.v.len() as u32;
        hud.v.extend_from_slice(&quad(*c));
        hud.runs.push((0, *blend, first, 6));
    }
    let format = wgpu::TextureFormat::Rgba8Unorm;
    let target = device.create_texture(&wgpu::TextureDescriptor {
        label: None, size: wgpu::Extent3d { width: W, height: H, depth_or_array_layers: 1 }, mip_level_count: 1, sample_count: 1,
        dimension: wgpu::TextureDimension::D2, format, usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC, view_formats: &[],
    });
    let view = target.create_view(&Default::default());
    let depth = gpu::depth_view(device, W, H);
    let id = [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]];
    let mut enc = r.render(device, queue, &view, &depth, id, id, [0.0; 3], 0, &[], &hud, None, None);
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
}

fn close(got: [f32; 3], want: [f32; 3], what: &str) {
    for k in 0..3 {
        assert!((got[k] - want[k]).abs() <= 2.5 / 255.0, "{what}: channel {k} got {:.4} want {:.4} (all {got:?} vs {want:?})", got[k], want[k]);
    }
}

#[test]
fn hud_sprites_blend_in_the_gs_order_with_the_gain_after_the_blend() {
    let Ok(iso) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set - skipping"); return };
    let mut sc = nf_view::load(&iso, "07000005.bin", false).expect("07000005");
    sc.materials.clear();
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
    let Some((device, queue)) = pollster::block_on(async {
        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions::default()).await.ok()?;
        adapter.request_device(&Default::default()).await.ok()
    }) else { eprintln!("no GPU adapter - skipping"); return };
    let mut r = gpu::Renderer::new(&device, &queue, wgpu::TextureFormat::Rgba8Unorm, &sc, false, false);
    let g = gpu::GAIN;
    let b = render(&mut r, &device, &queue, &[]);
    let map = |f: &dyn Fn(f32) -> f32| [f(b[0]), f(b[1]), f(b[2])];

    // Additive (0x48), a bright sprite: Cv 1.6 (0xcc on the 0x80 = 1.0 scale) clamps to Cs = 1.0,
    // As = 0.5 → Cd + gain·0.5. The old order gave Cd + min(1.6·gain, 1)·0.5 = Cd + 0.5.
    let got = render(&mut r, &device, &queue, &[(1, [1.6, 1.6, 1.6, 0.5])]);
    close(got, map(&|d| (d + g * 0.5).min(1.0)), "additive, bright");
    // Sister (can fail): the gain really lands — the old result is 0.066 lower.
    assert!(got[1] > b[1] + 0.5 + 0.04, "the additive term carries the gain ({:.3} over {:.3})", got[1], b[1]);

    // Additive, a dim sprite (no clamp): Cs = 0.25 → Cd + gain·0.25·0.75.
    let got = render(&mut r, &device, &queue, &[(1, [0.25, 0.25, 0.25, 0.75])]);
    close(got, map(&|d| (d + g * 0.25 * 0.75).min(1.0)), "additive, dim");

    // Subtractive (0x42) after an additive lift: Cd' = Cd + gain·0.75, then − gain·0.5·0.5.
    let got = render(&mut r, &device, &queue, &[(1, [1.0, 1.0, 1.0, 0.75]), (2, [0.5, 0.5, 0.5, 0.5])]);
    close(got, map(&|d| ((d + g * 0.75).min(1.0) - g * 0.25).max(0.0)), "subtractive");
    // The night pane's magenta subtract (0xff00ffff: Cv 2.0/0/2.0, As 0x7f/0x80) removes red and
    // blue entirely and keeps green [D NightSightSprInfo[2]].
    let got = render(&mut r, &device, &queue, &[(1, [1.0, 1.0, 1.0, 0.5]), (2, [255.0 / 128.0, 0.0, 255.0 / 128.0, 127.0 / 128.0])]);
    assert!(got[0] < 1.5 / 255.0 && got[2] < 1.5 / 255.0, "red and blue out: {got:?}");
    close([0.0, got[1], 0.0], [0.0, (b[1] + g * 0.5).min(1.0), 0.0], "magenta subtract keeps green");

    // Normal (0x44), half alpha white: Cd·(1 − 0.5) + gain·1.0·0.5.
    let got = render(&mut r, &device, &queue, &[(0, [1.0, 1.0, 1.0, 0.5])]);
    close(got, map(&|d| (d * 0.5 + g * 0.5).min(1.0)), "normal, translucent");
    // Normal, opaque mid grey: min(gain·0.4, 1) — the opaque case is unchanged by the order.
    let got = render(&mut r, &device, &queue, &[(0, [0.4, 0.4, 0.4, 1.0])]);
    close(got, [g * 0.4; 3], "normal, opaque");
}
