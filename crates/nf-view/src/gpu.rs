//! GPU side of the viewer (wgpu 30): a pipeline per draw state, a bind group per texture, indexed draws per
//! material in the game's pass order (docs/systems/render.md).
//!
//! Shading follows the GS "modulate" texture function: colour = texture × vertex colour, vertex RGBA on the PS2 scale
//! (0x80 = 1.0). Per material: the GS blend (ALPHA_1), depth write (ZBUF_1), alpha test (TEST_1) of its leaf, and the
//! fog: the game's screen pass (`psiDrawFogPass`, Z bits 8–15 as destination alpha, a fog-coloured sprite behind
//! Z 0x7FFF) computed per pixel from the view depth — zb = N1 + (F1 − N1)·n·(f/d − 1)/(f − n); beyond zb < 32767,
//! colour = fog + (colour − fog)·⌊zb/256⌋/128; the alpha list fades by min(⌊zb/128⌋, 255)/255 instead. Every output
//! is scaled by the display copy's gain (`SqueezeColourIn` 0x91 / 0x80, `CreateSqueezeFrame`) — linear, so the same
//! as scaling the finished frame. No back-face culling — the GS has no winding.
/*!
PS2 packet-builder citations (unported-front-end-menus-1) — the game builds these screen passes as cached
DMA/GIF packets at level load (psiPreGameRun 0x11e2e8 ← ResetMap_Load 0x1d6360) and consumes them per frame;
this renderer computes the same passes per pixel instead:
- `CreateZClearSprite` 0x117ce8 — TEST_1=0x32001 (ATST never + AFAIL=ZB_ONLY) + ZBUF_1=ZBufBase, a fullscreen
  write-z-only sprite cached in ZClearPacket and chained per frame by psiClearZ 0x10b9a0 (via psiPreDraw
  0x121590:19); here the render pass's `LoadOp::Clear(1.0)` depth clear.
- `CreateMoveRG2BA` 0x118718 — caches RGB2APacket: a PSMZ16 texture-read of the z-buffer (TEX0_1 = ZBufBase<<5)
  remapping its R,G (the z bytes) into B,A so `psiDrawFogPass` 0x10b960 gets per-pixel fog depth as dest alpha;
  here the fog pass reads the depth attachment directly in-shader.
- `CreateFogSprite` 0x119228 — caches FogOverlayPacket: a fullscreen quad in PS2FogR/G/B at fog factor 0x7fff,
  chained second in psiDrawFogPass 0x10b960; here the `scene.fog` uniform + the `zb < 32767` fog blend below.
- `CreateTotalScreenSprite` 0x1181b0 — packs (r,g,b,a) into one RGBAQ fullscreen SPRITE packet (Q=1.0, z-test
  off) that `psiFadeView` 0x10b9c8 submits per viewer for fades; the port's fade quads live in hud.rs.
- `CreatePointerSpriteList` 0x11a160 — converts a VECTOR array into one batched GIF sprite packet (56 quads per
  tag, tex bound by psiFont_SetTexture 0x120880), the snow/rain/emitter particle draw via psiDrawParticleList
  0x122d50; the port feeds wgpu quads from nf-game's world_fx ParticleSprite / Sim::snow_quads.
- VRAM texture plumbing replaced by resident wgpu uploads (see the make_tex2 block below):
  `CacheFlipBuffers` 0x1095d0, `CreateTexTranPacket` 0x11a900, `CreateTexUploadPacket` 0x11ad08,
  `CacheTextureSpace` 0x11cc70, `CreateSortedTextures` 0x11d7d8, `CheckAndAddTextureToSpace` 0x11d8b8,
  `CreateSpaceAndPackTextures` 0x11dcf8, `PageConv4to32` 0x11e640, `Conv4to32` 0x11e8d8,
  `PageConv8to32` 0x11eba8, `CacheAndFillNewGlistTextures` 0x11f180, `CheckSize` 0x120970,
  `CheckSizeAndShrink` 0x120928, and the GS double-buffer setup `CreateDB` 0x10b000 (main 0x109080:33 /
  psiPreGame_Run 0x109520:11 — NTSC 512×448, TextureBufBase 0x2a00) → wgpu surface configuration.
*/

use crate::scene::{Draw, Scene};
use wgpu::util::DeviceExt;

pub const DEPTH: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

/// One uniform slot: vp (64 B) + fog/fogk/lp0/lp1/lc0/lc1 (6 × 16 B) = 160 B, padded to the
/// 256-byte dynamic-offset alignment (fix-lighting).
const UNIFORM_SLOT: usize = 256;

/// `SqueezeColourIn` / 0x80 [L 0x91].
pub const GAIN: f32 = 145.0 / 128.0;

const SHADER: &str = r#"
override AREF: f32 = 0.0;
override AGREATER: f32 = 0.0;
override VMZ: f32 = 0.0;
override FOG: f32 = 0.0;
override FAR: f32 = 0.0;
override NOTEX: f32 = 0.0;  // debug: 1.0 = ignore the texture sample (treat as solid white) to isolate vertex colour
override NOFOG: f32 = 0.0;   // debug: 1.0 = skip the fog pass entirely
// fog: rgb + enabled; fogk: N1, (F1 − N1)·n/(f − n), f, gain.
// lp0/lp1: the frame's two closest lights' positions (psiLight_SetLights 0x10e588's TLight, w = 1);
// lc0/lc1: their TColor rows (rgb·intensity in the game's byte scale, w = 1/r²), fix-lighting.
// frame-240: one flat vec4 array, bound once per frame. us[0..4] = view_proj's columns, us[4..8] =
// view_proj_rot's (the camera-rotation sky), us[8] = fog, us[9] = fogk; then one 5-vec4 slot per
// draw, picked by the draw's instance index (slot 0 = the eye's own, the snow pass): lp0, lp1, lc0,
// lc1, sel (sel.x > 0.5 → the rotation-only matrix).
@group(0) @binding(0) var<storage, read> us: array<vec4<f32>>;
// G-043 GPU skinning: the frame's matrix palette and colour table (`Scene::skin_palette`).
@group(0) @binding(1) var<storage, read> pal: array<mat4x4<f32>>;
@group(0) @binding(2) var<storage, read> cols: array<vec4<f32>>;
// fix-gpu-facial-morphs (`SkinIt` 0x10bf30:88-118): the per-character facial-weight blocks
// (`SkinAttr.mwb`, `wt_stride` vec4s each — `AnimFacialBlend` 0x1be648's array at animobj+0xb0)
// and the concatenated MORPH_DATA targets (f32 triplets, `parsemap_block_morph_data` 0x1d23e8;
// each character's block is [attr.mtb .. attr.tend)).
@group(0) @binding(3) var<storage, read> wts: array<vec4<f32>>;
@group(0) @binding(4) var<storage, read> mtg: array<f32>;
@group(1) @binding(0) var tex: texture_2d<f32>;
@group(1) @binding(1) var samp: sampler;
struct VO { @builtin(position) p: vec4<f32>, @location(0) uv: vec2<f32>, @location(1) c: vec4<f32>, @location(2) d: f32 };
// `SkinIt` 0x10bf30's morph term, before the skinning: pos += Σ_m weight[m] × target[morph + m],
// the 12-byte reads as three f32; the game marshals the weights with an identity Morph_Index
// (`AnimObjectDraw` 0x1c4490:150), so channel m reads target base + (morph + m)·12. Channels
// past the character's target block contribute nothing (the count-bounded loop — `morph_vertex`).
fn morph_pos(p0: vec3<f32>, mtb: i32, mwb: u32, cnt: u32, tend: u32) -> vec3<f32> {
    var pos = p0;
    if (mtb >= 0 && cnt > 0u) {
        for (var m: u32 = 0u; m < cnt; m = m + 1u) {
            let w = wts[mwb + m / 4u][m % 4u];
            if (w != 0.0) {
                let e = u32(mtb) + m * 3u;
                if (e + 2u < tend) { pos = pos + w * vec3<f32>(mtg[e], mtg[e + 1u], mtg[e + 2u]); }
            }
        }
    }
    return pos;
}
@vertex fn vs(@location(0) p0: vec3<f32>, @location(1) uv: vec2<f32>, @location(2) c0: vec4<f32>,
              @location(3) sa: vec2<u32>, @location(4) sw: f32, @location(5) sc: u32, @location(6) n0: vec4<f32>,
              @location(7) mtb: i32, @location(8) mm: vec2<u32>, @location(9) tend: u32,
              @builtin(instance_index) iid: u32) -> VO {
    let b = 10u + 5u * iid;
    let lp0 = us[b]; let lp1 = us[b + 1u]; let lc0 = us[b + 2u]; let lc1 = us[b + 3u];
    var vp = mat4x4<f32>(us[0], us[1], us[2], us[3]);
    if (us[b + 4u].x > 0.5) { vp = mat4x4<f32>(us[4], us[5], us[6], us[7]); }
    var o: VO;
    var p = p0;
    var c = c0;
    // Skinned (a != 0xFFFFFFFF): w*(P[a]*p) + (1-w)*(P[b]*p), and the object's TAmbient multiplier.
    if (sa.x != 0xffffffffu) {
        // fix-gpu-facial-morphs: the morph term goes on the source position BEFORE the palette
        // blend — the game's order (`SkinIt` 0x10bf30 runs the Morph_Weight loop over the vertex
        // stream, then the VU1 skinning macro).
        let q = vec4<f32>(morph_pos(p0, mtb, mm.x, mm.y, tend), 1.0);
        p = sw * (pal[sa.x] * q).xyz + (1.0 - sw) * (pal[sa.y] * q).xyz;
        // fix-searchlight-remainder: the colour table's .a now carries the searchlight corona's alpha
        // byte (obj+0x106 / 255) — every other slot keeps 1.0, so this multiply changes only the
        // corona quads (the GS reads the byte as alpha/0x80; the ×255/128 rescale below reproduces it).
        c = vec4<f32>(c0.rgb * cols[sc].rgb, c0.a * cols[sc].a);
    }
    // The point lights (`Light_Create` + `Lights_CalcClosestLights` 0x1cb2e0 + `psiLight_SetLights`
    // 0x10e588; fix-lighting). The VU1 microcode's formula (.vutext 0x241c80, the point-light
    // program ~832-960), in world space: L = lightpos − pos, d² = |L|², 1/d² via DIV (VF0.w = 1),
    // atten = max(0, 1/d² − 1/r²), diffuse = max(0, atten · dot(N, L)) with the RAW L, and the sum
    // adds TColor.rgb · diffuse on the tinted vertex colour (the ambient row is the object's TWEAK,
    // already folded into the vertex colour by the port's fix-render path). fix-lighting2: the
    // vertex colour arrives as Unorm8x4 = byte/255 and the game's add is in BYTES, so the add is
    // /255 (the port's /128 over-added by exactly 255/128 — the saturation the merge flagged), and
    // the program clamps the summed colour to byte 255 with MINIi (I = 255.0 loaded by the LOI at
    // I-mem 900/936, applied at 919/954) — min(·, 1.0) here. No normal (sprites, snow, the radar,
    // the flash itself): unlit.
    if (FAR < 0.5) {
        var n = n0.xyz;
        if (sa.x != 0xffffffffu) { n = sw * (pal[sa.x] * vec4<f32>(n0.xyz, 0.0)).xyz + (1.0 - sw) * (pal[sa.y] * vec4<f32>(n0.xyz, 0.0)).xyz; }
        let n2 = dot(n, n);
        if (n2 > 1e-6) {
            for (var i: u32 = 0u; i < 2u; i = i + 1u) {
                let lp = select(lp0, lp1, i == 1u);
                let lc = select(lc0, lc1, i == 1u);
                if (lc.w <= 0.0 || (lc.x + lc.y + lc.z) <= 0.0) { continue; }
                let d = lp.xyz - p;
                let d2 = dot(d, d);
                let inv_d2 = 1.0 / d2;
                let atten = max(0.0, inv_d2 - lc.w);
                let term = max(0.0, atten * dot(normalize(n), d));
                c = vec4<f32>(min(c.rgb + lc.rgb * term / 255.0, vec3<f32>(1.0)), c.a);
            }
        }
    }
    o.p = vp * vec4<f32>(p, 1.0);
    o.d = o.p.w;
    if (FAR > 0.5) { o.p.z = o.p.w; }
    // view-model-last: the list-2 gun after `psiClearZ` — its depth squashed toward the eye so it covers
    // the world and the effects drawn before it, while its own parts still depth-sort.
    if (VMZ > 0.5) { o.p.z = o.p.z * 0.001; }
    o.uv = uv;
    o.c = c * (255.0 / 128.0);
    return o;
}
@fragment fn fs(i: VO) -> @location(0) vec4<f32> {
    let fog = us[8];
    let fogk = us[9];
    let s = textureSample(tex, samp, i.uv);
    let t = select(s, vec4<f32>(1.0, 1.0, 1.0, 1.0), NOTEX > 0.5);
    let o = t * i.c;
    if (AGREATER > 0.5) { if (o.a <= AREF) { discard; } } else if (o.a < AREF) { discard; }
    var rgb = min(o.rgb, vec3<f32>(1.0));
    var a = clamp(o.a, 0.0, 1.0);
    if (FOG > 0.5 && fog.w > 0.5 && NOFOG < 0.5) {
        let zb = fogk.x + fogk.y * (fogk.z / max(i.d, 0.001) - 1.0);
        if (FOG < 1.5) {
            // The GS's fog factor is fixed-point and wraps; clamp to [0,1] so a small zb can't collapse the frame to
            // the fog colour (the "ghostly white" symptom on levels with a bright fog, e.g. 0700000c) [M render.md].
            let f = clamp(floor(zb / 256.0) / 128.0, 0.0, 1.0);
            if (zb < 32767.0) { rgb = fog.rgb + (rgb - fog.rgb) * f; }
        } else {
            a = a * min(floor(zb / 128.0), 255.0) / 255.0;
        }
    }
    return vec4<f32>(min(rgb * fogk.w, vec3<f32>(1.0)), a);
}
"#;

/// The overlay (crosshair, HUD): screen-space triangles in normalised device coordinates, alpha-blended, no depth.
pub(crate) const HUD_SHADER: &str = r#"
@group(0) @binding(0) var tex: texture_2d<f32>;
@group(0) @binding(1) var samp: sampler;
struct HO { @builtin(position) p: vec4<f32>, @location(0) uv: vec2<f32>, @location(1) c: vec4<f32> };
@vertex fn vs(@location(0) p: vec2<f32>, @location(1) uv: vec2<f32>, @location(2) c: vec4<f32>) -> HO {
    var o: HO;
    o.p = vec4<f32>(p, 0.0, 1.0);
    o.uv = uv;
    o.c = c;
    return o;
}
// uv < 0: a plain colour; else the HUD texture modulated by the colour on the GS scale (1.0 = 0x80, so up to 2×).
@fragment fn fs(i: HO) -> @location(0) vec4<f32> {
    if (i.uv.x < 0.0) { return vec4<f32>(min(i.c.rgb * 1.1328125, vec3<f32>(1.0)), i.c.a); }
    let t = textureSample(tex, samp, i.uv);
    return vec4<f32>(min(t.rgb * i.c.rgb * 1.1328125, vec3<f32>(1.0)), min(t.a * i.c.a, 1.0));
}
"#;

/// One overlay vertex: NDC position (x right, y up) and RGBA (0–1).
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct HudVertex { pub p: [f32; 2], pub uv: [f32; 2], pub c: [f32; 4] }

/// One frame's overlay: vertices and the runs drawn in order — (texture index into `Scene::hud_textures` + 1, 0 =
/// none; blend mode; first vertex; count). Blend: 0 = normal, 1 = additive, 2 = subtractive.
#[derive(Clone, Default)]
pub struct HudFrame { pub v: Vec<HudVertex>, pub runs: Vec<(usize, u8, u32, u32)> }

impl HudFrame {
    /// Append another frame's vertices and runs (the dev overlay, WORKLOG 0081).
    pub fn append(&mut self, other: &HudFrame) {
        let offset = self.v.len() as u32;
        self.v.extend_from_slice(&other.v);
        for &(tex, add, first, count) in &other.runs {
            self.runs.push((tex, add, first + offset, count));
        }
    }
}

/// Most snow vertices per frame (4096 flakes × 6).
const SNOW_MAX: usize = 4096 * 6;
/// break-shards draw: at most this many texture groups of glass shards per frame (one uniform slot each).
const SHARD_GROUPS: usize = 16;

/// Most overlay vertices per frame.
const HUD_MAX: usize = 16384;

/// Extract the 6 frustum planes from a column-major `view_proj` (Gribb-Hartmann). Each plane is `(a, b, c, d)` with
/// `a·x + b·y + c·z + d ≥ 0` inside the frustum, normalised so a sphere of radius `r` centred at `p` is fully outside
/// when `a·p.x + b·p.y + c·p.z + d < −r`.
///
/// `view_proj` is the column-major `Mat4` produced by `glam` and stored as `[[col0], [col1], [col2], [col3]]` —
/// `vp[i]` is the i-th column. The i-th row (what Gribb-Hartmann combines with the w-row 3) is
/// `(vp[0][i], vp[1][i], vp[2][i], vp[3][i])`. With the right-handed Vulkan/wgpu projection used here
/// (`Mat4::perspective_rh`, depth 0..1), the near plane is `row3 + row2` (the OpenGL convention, slightly conservative
/// — culls a 0.05-unit strip behind the camera but never a visible object) and the far plane is `row3 − row2`.
pub fn extract_frustum(vp: [[f32; 4]; 4]) -> [[f32; 4]; 6] {
    let row = |i: usize| [vp[0][i], vp[1][i], vp[2][i], vp[3][i]];
    let r3 = row(3);
    // 6 planes — (row index, sign): plane = r3 + sign·row(i). left/right, bottom/top, near/far.
    let combos: [(usize, f32); 6] = [(0, 1.0), (0, -1.0), (1, 1.0), (1, -1.0), (2, 1.0), (2, -1.0)];
    let mut planes = [[0.0f32; 4]; 6];
    for (k, (i, s)) in combos.iter().enumerate() {
        let ri = row(*i);
        let p = [r3[0] + s * ri[0], r3[1] + s * ri[1], r3[2] + s * ri[2], r3[3] + s * ri[3]];
        let n = (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt().max(1e-12);
        planes[k] = [p[0] / n, p[1] / n, p[2] / n, p[3] / n];
    }
    planes
}

/// Test if a sphere is fully outside any of the 6 frustum planes (Gribb-Hartmann point-plane test extended by radius).
///
/// Citations (unported-front-end-menus-1): the game's object cull is `CheckObjectClip` 0x121a60 — the 8 corners
/// of the obj+0x80 sphere's AABB CLIP-tested against BOTH view slabs (clip volumes 0x244540/0x244580, built by
/// PS2WorldViewVU1Kick2 0x10b2b0:190-233; VU0 VI18 flags, 0x3f mask), returning −1 cull / 1 draw-clipped / 0
/// draw-plain [M 0x121a60:68-292] — called from psiDrawSkinObjectMatrix 0x111828:81 (the AnimObjectDraw 0x1c4490
/// ← View_DrawObjList 0x1e69a0 ← Game_Draw 0x1e6f50 path, every frame). `CheckObjectClipBox` 0x121e30 is the
/// same machinery on a GLIST_BOX min/max [M 0x121e30:72-289], called from RecurseAndDrawBoxes 0x1233e8:11-37
/// (−1 → Ps2NumVertsRejected += box vert count, 1 → clipped draw, 0 → subtree drawn unclipped). This module's
/// single-frustum p-vertex tests (`box_outside`, `sphere_outside`) are the behavioural replacement — the
/// game's two-slab pair and the 3-way verdict are not modelled [S unverified whether the slabs ever disagree].
/// `Vision_InView`'s shape as the port reads it: a box (a cel's `cel+0x40..0x58` min/max bbox) is
/// outside the view when all eight corners lie outside the SAME frustum plane (the standard
/// p-vertex test). `View_AddCels` (0x1e6ba0) frustum/bbox-tests each cel per viewer [M].
pub fn box_outside(planes: &[[f32; 4]; 6], min: [f32; 3], max: [f32; 3]) -> bool {
    for pl in planes {
        // The p-vertex: the corner most along the plane normal; if even that is outside, the whole box is.
        let px = if pl[0] >= 0.0 { max[0] } else { min[0] };
        let py = if pl[1] >= 0.0 { max[1] } else { min[1] };
        let pz = if pl[2] >= 0.0 { max[2] } else { min[2] };
        if pl[0] * px + pl[1] * py + pl[2] * pz + pl[3] < 0.0 { return true; }
    }
    false
}

pub fn sphere_outside(planes: &[[f32; 4]; 6], centre: [f32; 3], radius: f32) -> bool {
    planes.iter().any(|p| p[0] * centre[0] + p[1] * centre[1] + p[2] * centre[2] + p[3] < -radius)
}

/// An alpha-shaped texture: more than a quarter of its texels are below half its peak alpha (the shadow decals'
/// textures keep their shape in alpha; wall textures are uniformly opaque). Drawn after the opaque list-0 draws.
pub fn alpha_shaped(rgba: &[u8]) -> bool {
    let peak = rgba.chunks_exact(4).map(|p| p[3]).max().unwrap_or(0) as u32;
    let n = rgba.len() / 4;
    peak > 0 && rgba.chunks_exact(4).filter(|p| (p[3] as u32) * 2 < peak).count() * 4 > n
}

/// One indexed draw: its layer, pass, pipeline, texture bind group, index range and the object's world centre + radius.
/// batch-draws: the bindless world bind group, the snow texture's index in it, the indirect-args
/// buffer and its per-frame staging (5 u32 per draw: index_count, instance_count, first_index,
/// base_vertex, first_instance).
struct Batch { bg: wgpu::BindGroup, snow_tex: u32, indirect: wgpu::Buffer, args: Vec<u32> }

#[derive(Clone, Copy)]
struct DrawCall { layer: crate::scene::Layer, pass: u8, pipeline: usize, tex: usize, first: u32, count: u32, centre: [f32; 3], radius: f32, cel: Option<u16>, chain: u32, node: u32, lflags: u16, skin: Option<(u32, Option<u32>)> }

pub struct Renderer {
    hud_pipeline: wgpu::RenderPipeline,
    hud_add_pipeline: wgpu::RenderPipeline,
    hud_sub_pipeline: wgpu::RenderPipeline,
    hud_vbuf: wgpu::Buffer,
    hud_bgs: Vec<wgpu::BindGroup>,
    /// The burning woman's updatable HUD texture (0x030000db): (texture, width, height).
    woman_tex: Option<(wgpu::Texture, u32, u32)>,
    /// fix-menu-movies2: the hud bind-group layout + sampler. The ONE movie plane (fix-movie-playback's
    /// `movie_tex`/`movie_bg`) builds its bind group from them in render(); the menu's looping backdrop
    /// and the intro draw through the same plane.
    hud_tbgl: wgpu::BindGroupLayout,
    hud_sampler: wgpu::Sampler,
    pipelines: Vec<(Draw, wgpu::RenderPipeline)>,
    /// Snow: the particle pipeline (additive, depth test, no write), a per-frame vertex buffer and the flake texture.
    snow_pipeline: wgpu::RenderPipeline,
    shard_pipeline: wgpu::RenderPipeline,
    shard_vbuf: wgpu::Buffer,
    /// This frame's shard groups: (tex_frames index, first vertex, count) — `set_shards`.
    shard_groups: Vec<(usize, u32, u32)>,
    snow_vbuf: wgpu::Buffer,
    snow_bg: wgpu::BindGroup,
    /// The level's fog (`psiPreGameRun`): colour 0-255, `Ps2Far1`.
    fog: Option<([u8; 3], f32)>,
    /// Debug overrides (Bug 1): `notex` ignores texture samples (solid white), `nofog` skips the fog pass.
    notex: bool,
    nofog: bool,
    uniform: wgpu::Buffer,
    uniform_bg: wgpu::BindGroup,
    /// View-projection without the camera translation (sky objects that follow the viewer).
    /// The per-frame uniform staging, reused (frame-240: no per-frame allocation).
    staged: Vec<f32>,
    /// portal-orphans: per cel-graph node, true when no portal touches it and no room's chain owns it.
    orphan_nodes: Vec<bool>,
    /// The destroyed-glist swap's draw side, per frame: the destroyed Destroy leaves' material
    /// centres (f32 bits) — the draw filter below skips them ([M Destroy_Smash 0x12bb60's
    /// `Control_SetGList` swap; the C5 leaves swap to an undrawable slot [D lod_word −1] and the
    /// doorway ends open [L]). Fed by lib.rs each frame from `Scene::destroy_centres` + the sim's
    /// `props.destroy[..].dead`; empty until set (the loading screen and the pre-sim frames).
    pub destroy_dead: Vec<[u32; 3]>,
    /// batch-draws: the world pass with every texture in one bound array (the draw's texture index
    /// rides its per-draw slot) and one `multi_draw_indexed_indirect` per run of equal pipelines, in
    /// the draw order — None = the per-draw path (no adapter support, or `NF_NOBINDLESS`).
    batch: Option<Batch>,
    vbuf: wgpu::Buffer,
    skin_buf: wgpu::Buffer,
    snow_skin: wgpu::Buffer,
    pal_buf: wgpu::Buffer,
    col_buf: wgpu::Buffer,
    pub skin_layout: crate::scene::SkinLayout,
    /// The last uploaded palette's translations — a skinned draw's live world position for its light
    /// pick (`DrawCall::skin`; the game's obj+0x80 sphere follows the object [M 0x1e69a0:73-81]).
    skin_pos: std::cell::RefCell<Vec<[f32; 3]>>,
    /// Per character: the light-pick centre when the palette's root slot is not where the
    /// character is drawn (`Scene::char_light_centres` — the script actors, whose object sits at
    /// the origin while the sequence root carries the body). Set with each palette build.
    pub char_light_centre: Vec<Option<[f32; 3]>>,
    /// fix-gpu-facial-morphs: the per-character facial-weight blocks (per frame) and the static
    /// morph-target buffer.
    wts_buf: wgpu::Buffer,
    mtg_buf: wgpu::Buffer,
    /// The frame's lights, resolved (kind-2 positions against their parts). Per draw the renderer
    /// picks the ≤2 closest within the draw's bounding sphere (`Lights_CalcClosestLights`
    /// 0x1cb2e0 / `psiLight_SetLights` 0x10e588) into the draw's uniform slot.
    pub lights: nf_game::light::ResolvedLights,
    /// The target's width / height: the movie plane keeps the game's 4:3 screen centred in it (every movie and the
    /// loading picture fill the PS2's 4:3 picture [M DecodeBGFMVFrame 0x108550 — 512×448 centred]; stretched to a
    /// 16:9 window they were a third too wide). 0 = fill the target.
    pub screen_aspect: f32,
    ibuf: wgpu::Buffer,
    tex_bgs: Vec<wgpu::BindGroup>,
    /// Per texture slot: its first bind group, frame count and rate (animated textures).
    tex_frames: Vec<(usize, u32, u32)>,
    /// Per material: layer, first index, index count (sorted by layer: sky pass first).
    draws: Vec<DrawCall>,
    /// Per-frame sort buffer reused across frames (A-11e): the per-frame draw order — `(pass, sort key, draw index)`
    /// — is `clear`ed + refilled every render instead of allocating ~64 KB each frame.
    order: Vec<(u8, f32, usize)>,
    /// The room cels' frustum-test spheres (`Scene::cel_sphere`), built once: `View_AddCels`
    /// (0x1e6ba0) frustum-tests each cel's bbox per viewer; fix-render3 skips a static World draw
    /// whose cel fails the same test. `NF_NOCELCULL` disables.
    cel_boxes: Vec<([f32; 3], [f32; 3])>,
    /// The runtime cel graph + portals (fix-perf-level): the walk's per-full-cel set gates static
    /// World draws (the `View_AddCels` GfxList set — only portal-visited cels are added). Graphs
    /// without portals fall back to the fix-render3 box test.
    cel_graph: crate::scene::CelGraph,
    /// The viewer's persistent cel (`viewer+4`, `Camera_CheckLocation` 0x1bae90) — carried across
    /// frames, updated by the move raycast / bbox keep / `build_FindCel` chain. fix-portal.
    pub viewer_cel: crate::scene::ViewerCel,
    /// This frame's room-level visibility mask, computed once per frame by `cel_visibility` (the
    /// skin palette's object cull and `render`'s World cull share it — one walk per frame).
    pub last_vis: Option<Vec<bool>>,
    /// This frame's CELL-level game mask (`View_AddCels` 0x1e6ba0: the VisCelList plus the `+0x18`
    /// chain pass) — `None` when the walk cannot run (the room box fallback gates everything).
    /// fix-portal2.
    pub last_cell_vis: Option<Vec<bool>>,
    // fix-movie-playback: the lazy full-screen movie plane (fields only touched while a movie
    // plays; everything else is additive, nothing existing changed shape). The hud layout/sampler
    // it reuses are the Renderer's `hud_tbgl`/`hud_sampler` fields (fix-menu-movies2).
    movie_tex: Option<wgpu::Texture>,
    movie_bg: Option<wgpu::BindGroup>,
    movie_size: Option<(u32, u32)>,
    movie_vbuf: wgpu::Buffer,
}

impl Renderer {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, format: wgpu::TextureFormat, scene: &Scene, notex: bool, nofog: bool) -> Renderer {
        // batch-draws: the bindless decision comes first — it picks the shader variant and the layout.
        let n_world_tex = 2 + scene.textures.iter().map(|t| 1 + t.more.len()).sum::<usize>();
        let want = wgpu::Features::TEXTURE_BINDING_ARRAY | wgpu::Features::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING | wgpu::Features::INDIRECT_FIRST_INSTANCE;
        let bindless = device.features().contains(want)
            && (n_world_tex as u32) <= device.limits().max_binding_array_elements_per_shader_stage
            && std::env::var_os("NF_NOBINDLESS").is_none();
        let src = if bindless {
            let s = (String::from("enable wgpu_binding_array;
") + SHADER)
                .replace("@group(1) @binding(0) var tex: texture_2d<f32>;", "@group(1) @binding(0) var texs: binding_array<texture_2d<f32>>;")
                .replace("struct VO { @builtin(position) p: vec4<f32>, @location(0) uv: vec2<f32>, @location(1) c: vec4<f32>, @location(2) d: f32 };",
                         "struct VO { @builtin(position) p: vec4<f32>, @location(0) uv: vec2<f32>, @location(1) c: vec4<f32>, @location(2) d: f32, @location(3) @interpolate(flat) ti: u32 };")
                .replace("    o.uv = uv;\n    o.c = c * (255.0 / 128.0);", "    o.uv = uv;\n    o.ti = u32(us[b + 4u].y);\n    o.c = c * (255.0 / 128.0);")
                .replace("let s = textureSample(tex, samp, i.uv);", "let s = textureSample(texs[i.ti], samp, i.uv);");
            assert!(s.contains("texs[i.ti]") && s.contains("o.ti = ") && s.contains("binding_array"), "the bindless shader edits must all apply");
            s
        } else { SHADER.to_string() };
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some("nf-view"), source: wgpu::ShaderSource::Wgsl(src.into()) });
        let ubgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0, visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                // fix-lighting: one 256-byte slot per draw — the game uploads a per-object light pair
                // (`Lights_CalcClosestLights` + `psiLight_SetLights` per object/cel). frame-240: a storage
                // array indexed by instance, not a dynamic-offset uniform.
                ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { read_only: true }, has_dynamic_offset: false, min_binding_size: None },
                count: None,
            },
            wgpu::BindGroupLayoutEntry { binding: 1, visibility: wgpu::ShaderStages::VERTEX, ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { read_only: true }, has_dynamic_offset: false, min_binding_size: None }, count: None },
            wgpu::BindGroupLayoutEntry { binding: 2, visibility: wgpu::ShaderStages::VERTEX, ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { read_only: true }, has_dynamic_offset: false, min_binding_size: None }, count: None },
            // fix-gpu-facial-morphs: the facial-weight blocks and the MORPH_DATA targets.
            wgpu::BindGroupLayoutEntry { binding: 3, visibility: wgpu::ShaderStages::VERTEX, ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { read_only: true }, has_dynamic_offset: false, min_binding_size: None }, count: None },
            wgpu::BindGroupLayoutEntry { binding: 4, visibility: wgpu::ShaderStages::VERTEX, ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { read_only: true }, has_dynamic_offset: false, min_binding_size: None }, count: None }],
        });
        let tbgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0, visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture { sample_type: wgpu::TextureSampleType::Float { filterable: true }, view_dimension: wgpu::TextureViewDimension::D2, multisampled: false },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry { binding: 1, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering), count: None },
            ],
        });
        let wtbgl = bindless.then(|| device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("world textures (bindless)"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0, visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture { sample_type: wgpu::TextureSampleType::Float { filterable: true }, view_dimension: wgpu::TextureViewDimension::D2, multisampled: false },
                    count: std::num::NonZeroU32::new(n_world_tex as u32),
                },
                wgpu::BindGroupLayoutEntry { binding: 1, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering), count: None },
            ],
        }));
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { label: None, bind_group_layouts: &[Some(&ubgl), Some(wtbgl.as_ref().unwrap_or(&tbgl))], immediate_size: 0 });
        // World: depth-tested. Sky pass: painted first in slot order without depth (`View_DrawSky` before the world).
        // One pipeline per draw state: blend (ALPHA_1), depth write (ZBUF_1), alpha test (TEST_1) and fog as shader
        // overrides; the pre-world sky pass ignores depth, the post-fog sky is pinned to the far plane (drawn only where
        // nothing else is).
        let make_pipeline = |d: Draw| {
            let (compare, write) = match d.pass { 0 | 6 => (wgpu::CompareFunction::Always, false), 4 => (wgpu::CompareFunction::LessEqual, false), _ => (wgpu::CompareFunction::LessEqual, d.zwrite) };
            let alpha_keep = wgpu::BlendComponent { src_factor: wgpu::BlendFactor::Zero, dst_factor: wgpu::BlendFactor::One, operation: wgpu::BlendOperation::Add };
            let blend = match d.blend {
                1 => wgpu::BlendState { color: wgpu::BlendComponent { src_factor: wgpu::BlendFactor::SrcAlpha, dst_factor: wgpu::BlendFactor::One, operation: wgpu::BlendOperation::Add }, alpha: alpha_keep },
                2 => wgpu::BlendState { color: wgpu::BlendComponent { src_factor: wgpu::BlendFactor::SrcAlpha, dst_factor: wgpu::BlendFactor::One, operation: wgpu::BlendOperation::ReverseSubtract }, alpha: alpha_keep },
                // (Cs−Cd)·FIX0 + Cd (07000041's mod batches): the framebuffer comes back unchanged, but the
                // depth test/write and the alpha test still run as configured (the GS evaluates the whole
                // equation to D = Cd) — scene.rs `Draw::from_gs` blend 3.
                3 => wgpu::BlendState { color: alpha_keep, alpha: alpha_keep },
                // (Cs−0)·Ad + Cd (07000041's env-map overlays): add the source only where the base batch
                // wrote destination alpha; the alpha channel itself comes back unchanged (D = Cd).
                4 => wgpu::BlendState { color: wgpu::BlendComponent { src_factor: wgpu::BlendFactor::DstAlpha, dst_factor: wgpu::BlendFactor::One, operation: wgpu::BlendOperation::Add }, alpha: alpha_keep },
                _ => wgpu::BlendState { color: wgpu::BlendComponent { src_factor: wgpu::BlendFactor::SrcAlpha, dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha, operation: wgpu::BlendOperation::Add }, alpha: alpha_keep },
            };
            // As >= 1 (TEST 0x5001B) -> a >= 1/128; As > 0x7F (0x507FD) -> a > 127/128.
            let (aref, greater) = match d.atest { 1 => (1.0 / 128.0, 0.0), 2 => (127.0 / 128.0, 1.0), _ => (0.0, 0.0) };
            let constants = [("AREF", aref), ("AGREATER", greater), ("FOG", d.fog as f64), ("FAR", if d.pass == 4 { 1.0 } else { 0.0 }), ("VMZ", if d.pass == 7 { 1.0 } else { 0.0 }),
                ("NOTEX", if notex { 1.0 } else { 0.0 }), ("NOFOG", if nofog { 1.0 } else { 0.0 })];
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("nf-view"),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: &shader, entry_point: Some("vs"), compilation_options: wgpu::PipelineCompilationOptions { constants: &constants, ..Default::default() },
                    buffers: &[Some(wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<crate::scene::Vertex>() as u64,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2, 2 => Unorm8x4, 6 => Snorm8x4],
                    }), Some(wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<crate::scene::SkinAttr>() as u64,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &wgpu::vertex_attr_array![3 => Uint32x2, 4 => Float32, 5 => Uint32, 7 => Sint32, 8 => Uint32x2, 9 => Uint32],
                    })],
                },
                primitive: wgpu::PrimitiveState { cull_mode: None, ..Default::default() },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: DEPTH, depth_write_enabled: Some(write), depth_compare: Some(compare),
                    stencil: Default::default(), bias: Default::default(),
                }),
                multisample: Default::default(),
                fragment: Some(wgpu::FragmentState {
                    module: &shader, entry_point: Some("fs"), compilation_options: wgpu::PipelineCompilationOptions { constants: &constants, ..Default::default() },
                    targets: &[Some(wgpu::ColorTargetState { format, blend: Some(blend), write_mask: wgpu::ColorWrites::ALL })],
                }),
                multiview_mask: None,
                cache: None,
            })
        };
        let hud_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some("nf-view hud"), source: wgpu::ShaderSource::Wgsl(HUD_SHADER.into()) });
        let hud_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { label: None, bind_group_layouts: &[Some(&tbgl)], immediate_size: 0 });
        let make_hud = |blend: wgpu::BlendState| device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("nf-view hud"),
            layout: Some(&hud_layout),
            vertex: wgpu::VertexState {
                module: &hud_shader, entry_point: Some("vs"), compilation_options: Default::default(),
                buffers: &[Some(wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<HudVertex>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2, 2 => Float32x4],
                })],
            },
            primitive: wgpu::PrimitiveState { cull_mode: None, ..Default::default() },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: DEPTH, depth_write_enabled: Some(false), depth_compare: Some(wgpu::CompareFunction::Always),
                stencil: Default::default(), bias: Default::default(),
            }),
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &hud_shader, entry_point: Some("fs"), compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState { format, blend: Some(blend), write_mask: wgpu::ColorWrites::ALL })],
            }),
            multiview_mask: None,
            cache: None,
        });
        // GS ALPHA 0x44 (normal), 0x48 (Cs·As + Cd: sprite flag 0x100), and 0x42 (Cd - Cs·As: sprite flag 0x200, subtractive).
        let hud_pipeline = make_hud(wgpu::BlendState::ALPHA_BLENDING);
        let hud_add_pipeline = make_hud(wgpu::BlendState {
            color: wgpu::BlendComponent { src_factor: wgpu::BlendFactor::SrcAlpha, dst_factor: wgpu::BlendFactor::One, operation: wgpu::BlendOperation::Add },
            alpha: wgpu::BlendComponent { src_factor: wgpu::BlendFactor::Zero, dst_factor: wgpu::BlendFactor::One, operation: wgpu::BlendOperation::Add },
        });
        let hud_sub_pipeline = make_hud(wgpu::BlendState {
            color: wgpu::BlendComponent { src_factor: wgpu::BlendFactor::SrcAlpha, dst_factor: wgpu::BlendFactor::One, operation: wgpu::BlendOperation::ReverseSubtract },
            alpha: wgpu::BlendComponent { src_factor: wgpu::BlendFactor::Zero, dst_factor: wgpu::BlendFactor::One, operation: wgpu::BlendOperation::Add },
        });
        let hud_vbuf = device.create_buffer(&wgpu::BufferDescriptor { label: None, size: (HUD_MAX * std::mem::size_of::<HudVertex>()) as u64, usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
        let mut pipelines: Vec<(Draw, wgpu::RenderPipeline)> = Vec::new();
        for m in &scene.materials { if !pipelines.iter().any(|(d, _)| *d == m.draw) { pipelines.push((m.draw, make_pipeline(m.draw))); } }
        // G-043: the skinning layout, palette and colour buffers (sized once per scene).
        let (skin_layout, char_attrs) = scene.skin_layout();
        let pal_buf = device.create_buffer(&wgpu::BufferDescriptor { label: Some("palette"), size: (skin_layout.n_mats.max(1) as u64) * 64, usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
        let col_buf = device.create_buffer(&wgpu::BufferDescriptor { label: Some("colours"), size: (skin_layout.col_one as u64 + 1) * 16, usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
        // fix-gpu-facial-morphs: the per-character facial-weight blocks (written each frame in
        // `update_skin`) and the scene's concatenated MORPH_DATA targets (static). A scene with no
        // morph bearer gets 16-byte dummies (a zero-size storage binding is a create error) — the
        // attrs' mtb −1 keeps the shader off them.
        let wts_buf = device.create_buffer(&wgpu::BufferDescriptor { label: Some("morph weights"), size: ((skin_layout.wt_stride as u64 * scene.characters.len() as u64) * 16).max(16), usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
        let blob = scene.morph_target_blob();
        let mtg_contents = if blob.is_empty() { vec![0u8; 16] } else { blob };
        let mtg_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some("morph targets"), contents: &mtg_contents, usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST });
        let slots = (scene.materials.len() + 2) as u64;
        let make_uniform = || {
            let buf = device.create_buffer(&wgpu::BufferDescriptor { label: None, size: UNIFORM_SLOT as u64 * slots, usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
            let bg = device.create_bind_group(&wgpu::BindGroupDescriptor { label: None, layout: &ubgl, entries: &[wgpu::BindGroupEntry { binding: 0, resource: buf.as_entire_binding() }, wgpu::BindGroupEntry { binding: 1, resource: pal_buf.as_entire_binding() }, wgpu::BindGroupEntry { binding: 2, resource: col_buf.as_entire_binding() }, wgpu::BindGroupEntry { binding: 3, resource: wts_buf.as_entire_binding() }, wgpu::BindGroupEntry { binding: 4, resource: mtg_buf.as_entire_binding() }] });
            (buf, bg)
        };
        let (uniform, uniform_bg) = make_uniform();
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::Repeat, address_mode_v: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear, min_filter: wgpu::FilterMode::Linear, ..Default::default()
        });
        let make_tex2 =|w: u32, h: u32, rgba: &[u8]| {
            // The game stages level/HUD texture patches through a double-buffered GS VRAM bump
            // allocator per frame (unported-misc-1: initcache 0x109688 / allocache 0x109700 /
            // CacheTexture 0x11cd58 / CacheFlipBuffers 0x1095d0 — docs/systems/boot-fs-cache.md);
            // here every texture is decoded once and uploaded directly, which is the same job
            // without the per-frame staging window.
            let size = wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 };
            let t = device.create_texture(&wgpu::TextureDescriptor {
                label: None, size, mip_level_count: 1, sample_count: 1, dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm, usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST, view_formats: &[],
            });
            queue.write_texture(
                wgpu::TexelCopyTextureInfo { texture: &t, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
                rgba, wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(4 * w), rows_per_image: Some(h) }, size);
            let view = t.create_view(&Default::default());
            let bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None, layout: &tbgl,
                entries: &[wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&view) }, wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&sampler) }],
            });
            (bg, view)
        };
        let make_tex = |w: u32, h: u32, rgba: &[u8]| make_tex2(w, h, rgba).0;
        /* Citations (unported-front-end-menus-1) — every texture here is uploaded linear-RGBA and stays resident
        for the whole level, which replaces the game's PS2 VRAM texture-cache plumbing outright (all reached per
        frame / per level load; evidence in docs/systems/psi-render-plumbing.md):
        - `CacheFlipBuffers` 0x1095d0 — the texture-cache double-buffer window flip (CacheFrame = (CacheFrame+1)&1,
          CacheStart/CacheSize from initcache 0x109688's TextureBufBase 0x2a00 split), driven by
          `CacheAndFillNewGlistTextures` 0x11f180 when a draw's texture cost exceeds the half-cache.
        - `CacheTextureSpace` 0x11cc70 — per-frame VRAM cost of a texture's Space page (0x400-entry × 0x1c table at
          0x25cae0) + its CLUT, with the animated-frame pick (GameState+0x56 / rate) % frames.
        - `CreateSortedTextures` 0x11d7d8 — texture indices selection-sorted big-to-small into TexSize (the packer
          priority), then Space ids set 0xffff.
        - `CheckAndAddTextureToSpace` 0x11d8b8 — greedy shelf packer into a Space with a 64-rect occupancy list
          (TextureRectList 0x316984), called from Reduce 0x11daf0.
        - `CreateSpaceAndPackTextures` 0x11dcf8 — copies each texture's pixels into its packed Space, converts
          4bpp/8bpp Spaces to 32bpp (`Conv4to32` 0x11e8d8 / `Conv8to32` 0x11eeb0 — per-page via `PageConv4to32`
          0x11e640 / `PageConv8to32` 0x11eba8 GS page swizzles), and builds the upload packets
          (`CreateTexTranPacket` 0x11a900 — GIF BITBLTBUF/TRXPOS/TRXREG per Space and per CLUT).
        - `CreateTexUploadPacket` 0x11ad08 — PS2WipeVram 0x11cab0's blank-image VRAM packet (0x40-page rows),
          run from psiPreGame_Run 0x109520:10 / psiResetResources 0x11f390:17.
        - `CheckSize` 0x120970 (`end − start`, 8 bytes) and `CheckSizeAndShrink` 0x120928 (Mem_Shrink the packet
          allocation to its used bytes) — DMA packet size helpers with no Rust counterpart.
        - `CreateDB` 0x10b000 — the NTSC/PAL GS double-buffer + VRAM layout (TextureBufBase 0x2a00,
          PS2ScreenWidth 512 × 448/512) → this process's wgpu surface configuration.
        */
        // batch-draws: the world textures' views, index-aligned with `tex_bgs` (the bindless array).
        let mut tex_views: Vec<wgpu::TextureView> = Vec::new();
        let (bg0, v0) = make_tex2(1, 1, &[255, 255, 255, 255]);
        tex_views.push(v0);
        let mut tex_bgs = vec![bg0];
        let mut tex_frames = vec![(0, 1, 1)];
        let hud_bgs = std::iter::once(make_tex(1, 1, &[255, 255, 255, 255])).chain(scene.hud_textures.iter().map(|(_, t)| make_tex(t.width, t.height, &t.rgba))).collect();
        // The burning woman's texture (0x030000db, crate::woman): its own updatable texture — the frame loop rewrites its
        // texels while a paused page shows it. Its bind group replaces the static one built above.
        let mut hud_bgs: Vec<wgpu::BindGroup> = hud_bgs;
        let woman_tex = scene.hud_textures.iter().position(|(h, _)| *h == 0x0300_00db).map(|i| {
            let t = &scene.hud_textures[i].1;
            let size = wgpu::Extent3d { width: t.width, height: t.height, depth_or_array_layers: 1 };
            let tex = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("woman"), size, mip_level_count: 1, sample_count: 1, dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm, usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST, view_formats: &[],
            });
            queue.write_texture(
                wgpu::TexelCopyTextureInfo { texture: &tex, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
                &t.rgba, wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(4 * t.width), rows_per_image: Some(t.height) }, size);
            let view = tex.create_view(&Default::default());
            hud_bgs[i + 1] = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None, layout: &tbgl,
                entries: &[wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&view) }, wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&sampler) }],
            });
            (tex, t.width, t.height)
        });
        for t in &scene.textures {
            tex_frames.push((tex_bgs.len(), 1 + t.more.len() as u32, t.rate));
            let (bg, v) = make_tex2(t.width, t.height, &t.rgba);
            tex_bgs.push(bg); tex_views.push(v);
            for f in &t.more { let (bg, v) = make_tex2(t.width, t.height, f); tex_bgs.push(bg); tex_views.push(v); }
        }
        let material_tex: Vec<usize> = scene.materials.iter()
            .map(|m| m.key.and_then(|k| scene.textures.iter().position(|t| t.key == k)).map_or(0, |i| i + 1)).collect();
        // Bug A-2a diagnostic: log every material that falls back to the 1×1 white texture (index 0), to
        // tell whether the "white-translucent character" symptom on a level is the texture fallback (key=None or
        // missing TextureImage) or a blend-mode mismatch (alpha & 0xff == 0x48 = additive — see scene.rs).
        if std::env::var_os("NF_TEX_TRACE").is_some() {
            for (m, &t) in scene.materials.iter().zip(&material_tex) {
                if t == 0 {
                    eprintln!("gpu: WHITE FALLBACK material: L={:?} pass={} blend={} atest={} key={:?} (first_index={} count={})",
                        m.layer, m.draw.pass, m.draw.blend, m.draw.atest, m.key, m.first_index, m.index_count);
                }
            }
        }
        // The Character-layer tail holds the bind-pose positions; the shader poses them with the palette.
        let mut verts = scene.vertices.clone();
        for (v, cv) in verts[scene.char_vertex_start..].iter_mut().zip(&scene.char_verts) { v.pos = cv.pos; }
        let vbuf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: None, contents: bytemuck::cast_slice(&verts), usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST });
        let stat = crate::scene::SkinAttr { a: u32::MAX, b: u32::MAX, w: 1.0, col: 0, mtb: -1, mwb: 0, cnt: 0, tend: 0 };
        let mut attrs = vec![stat; scene.char_vertex_start];
        attrs.extend_from_slice(&char_attrs);
        attrs.resize(verts.len(), stat);
        let skin_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some("skin attrs"), contents: bytemuck::cast_slice(&attrs), usage: wgpu::BufferUsages::VERTEX });
        let snow_skin = device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some("snow attrs"), contents: bytemuck::cast_slice(&vec![stat; SNOW_MAX]), usage: wgpu::BufferUsages::VERTEX });
        let ibuf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: None, contents: bytemuck::cast_slice(&scene.indices), usage: wgpu::BufferUsages::INDEX });
        // Draw order (docs/systems/render.md): sky before the world, list 0, characters, list 1 (alpha), sky after the fog
        // pass, effects; the vertex buffer keeps its layout (the character tail stays contiguous).
        // List 0 order per `View_DrawObjList` (0x1e69a0) + `View_AddCels` (0x1e6ba0): the viewer's per-list array holds
        // both cel entries and object entries, QuickSorted by key. `View_AddCels` pushes the ROOM cels' primary
        // glists — one per 0xc023/0xc047 placement — into list 0 with the SYNTHETIC key −90000 + 10k over the
        // +0x20 chain `Vision_Portal_Recurse` (0x1e86e8) threads per frame from the depth-sorted `VisCelList`
        // (QuickSort 0x1d30a8 comparator[3]: ascending portal depth); the non-room 0x8000 placements (the baked
        // decals, walkways, blockers — the +0x18 inner nodes) and the dynamic World objects draw at ±dist²
        // (`View_AddObjects` 0x1e62d8). So the rooms draw first (per-frame chain order), then the distance-keyed
        // entries front-to-back. `Material::chain` carries the room's build-time parse-rank fallback, `node` its
        // per-frame chain cel (u32::MAX = a distance-keyed entry).
        let draws: Vec<DrawCall> = scene.materials.iter().zip(&material_tex)
            .map(|(m, &tex)| DrawCall { layer: m.layer, pass: m.draw.pass, pipeline: pipelines.iter().position(|(d, _)| *d == m.draw).unwrap(), tex, first: m.first_index, count: m.index_count, centre: m.centre, radius: m.radius, cel: m.cel, chain: m.chain, node: m.node, lflags: m.lflags, skin: m.skin }).collect();
        let fog = scene.fog;
        let snow_pipeline = make_pipeline(Draw { pass: 5, blend: 1, zwrite: false, atest: 0, fog: 0 });
        // break-shards draw: normal alpha, no depth write, the alpha-list test [S: psiDrawShard's
        // GS state is in its microcode packet, unread — the pane's own list-1 state stood in].
        let shard_pipeline = make_pipeline(Draw { pass: 5, blend: 0, zwrite: false, atest: 1, fog: 0 });
        let shard_vbuf = device.create_buffer(&wgpu::BufferDescriptor { label: Some("shards"), size: (SNOW_MAX * std::mem::size_of::<crate::scene::Vertex>()) as u64, usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
        let snow_vbuf = device.create_buffer(&wgpu::BufferDescriptor { label: None, size: (SNOW_MAX * std::mem::size_of::<crate::scene::Vertex>()) as u64, usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
        let (snow_bg, snow_view) = match &scene.snow_texture { Some(t) => make_tex2(t.width, t.height, &t.rgba), None => make_tex2(1, 1, &[255, 255, 255, 255]) };
        let batch = wtbgl.as_ref().map(|l| {
            let snow_tex = tex_views.len() as u32;
            let mut refs: Vec<&wgpu::TextureView> = tex_views.iter().collect();
            refs.push(&snow_view);
            assert_eq!(refs.len(), n_world_tex, "the bindless array holds exactly the counted textures");
            let bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("world textures (bindless)"), layout: l,
                entries: &[wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureViewArray(&refs) }, wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&sampler) }],
            });
            let indirect = device.create_buffer(&wgpu::BufferDescriptor { label: Some("draw args"), size: 20 * (scene.materials.len() as u64 + 1), usage: wgpu::BufferUsages::INDIRECT | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
            Batch { bg, snow_tex, indirect, args: Vec::new() }
        });
        let order_cap = draws.len();
        let cel_boxes = scene.cels.iter().map(|c| (c.min, c.max)).collect();
        let cel_graph = scene.cel_graph.clone();
        // fix-movie-playback: the full-screen movie plane. The game blits each decoded 512×448 frame
        // straight into the frame buffer (DecodeBGFMVFrame 0x108550 → PS2UploadBGMImage 0x108740);
        // the port draws it as a textured quad through the HUD pipeline after everything else. The
        // texture + bind group are (re)built lazily in render() at the frame's size, so only the
        // static quad buffer and the shared layout/sampler are made here.
        let movie_vbuf = device.create_buffer(&wgpu::BufferDescriptor { label: None, size: 6 * std::mem::size_of::<HudVertex>() as u64, usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
        queue.write_buffer(&movie_vbuf, 0, bytemuck::cast_slice(&[
            // Row 0 of the frame at the top of the screen (NDC y up, uv v down — the HUD quad
            // convention, hud.rs quad()).
            HudVertex { p: [-1.0, 1.0], uv: [0.0, 0.0], c: [1.0; 4] }, HudVertex { p: [1.0, 1.0], uv: [1.0, 0.0], c: [1.0; 4] }, HudVertex { p: [1.0, -1.0], uv: [1.0, 1.0], c: [1.0; 4] },
            HudVertex { p: [-1.0, 1.0], uv: [0.0, 0.0], c: [1.0; 4] }, HudVertex { p: [1.0, -1.0], uv: [1.0, 1.0], c: [1.0; 4] }, HudVertex { p: [-1.0, -1.0], uv: [0.0, 1.0], c: [1.0; 4] },
        ]));
        // fix-menu-movies2: ONE movie plane serves every movie — the mission-flow briefing/outro
        // (fix-movie-playback) and the front end's looping page backdrop + the P_INTRO pair
        // (fix-menu-movies, now routed through this same plane). The game uploads the decoded
        // frame to VRAM as the page's own background texture [M PS2UploadBGMImage 0x108740;
        // movies.md §Presentation], so the quad draws BEFORE the HUD runs (under the page
        // controls); state 6 suppresses the overlay while its movie plays, so the draw position
        // is equivalent there.
        let orphan_nodes: Vec<bool> = {
            let n = cel_graph.cells.len();
            let mut reached = vec![false; n];
            for p in &cel_graph.portals { if p.cel_a < n { reached[p.cel_a] = true; } if p.cel_b < n { reached[p.cel_b] = true; } }
            for ch in &cel_graph.chains { for &c in ch { if (c as usize) < n { reached[c as usize] = true; } } }
            reached.iter().map(|r| !r).collect()
        };
        Renderer { shard_pipeline, shard_vbuf, shard_groups: Vec::new(), woman_tex, snow_pipeline, snow_vbuf, snow_bg, hud_pipeline, hud_add_pipeline, hud_sub_pipeline, hud_vbuf, hud_bgs, pipelines, fog, notex, nofog, uniform, uniform_bg, staged: Vec::new(), orphan_nodes, destroy_dead: Vec::new(), batch, vbuf, skin_buf, snow_skin, pal_buf, col_buf, wts_buf, mtg_buf, skin_layout, skin_pos: Default::default(), char_light_centre: Vec::new(), lights: Default::default(), screen_aspect: 0.0, ibuf, tex_bgs, tex_frames, draws, order: Vec::with_capacity(order_cap), cel_boxes, cel_graph, viewer_cel: Default::default(), last_vis: None, last_cell_vis: None, movie_tex: None, movie_bg: None, movie_size: None, movie_vbuf, hud_tbgl: tbgl, hud_sampler: sampler }
    }

    /// The game's depth-0 pyramid for the walk [M `Vision_Calculate_View_Fustrum` 0x1e7df8,
    /// disassembled]: the four fov side planes through the eye (the Gribb-Hartmann extraction's
    /// first four are the same half-spaces for this fov/aspect) plus the near plane through the
    /// 200-unit quad corners (c1,c3,c2) whose normal points BACK at the eye — it keeps
    /// `fw·p <= fw·eye + 200` (beyond 200 units is fog; the game clips those portal polygons away).
    /// `fw` = the GH near plane's normalized normal (it points forward, away from the eye).
    fn depth0_game(planes: &[[f32; 4]; 6], eye: [f32; 3]) -> [[f32; 4]; 5] {
        let fw = [planes[4][0], planes[4][1], planes[4][2]];
        let fe = fw[0] * eye[0] + fw[1] * eye[1] + fw[2] * eye[2];
        [planes[0], planes[1], planes[2], planes[3], [-fw[0], -fw[1], -fw[2], fe + 200.0]]
    }

    /// This frame's room-level visibility mask — one walk per frame, shared by the skin palette's
    /// object cull (`View_AddObjects` 0x1e62d8 draws only the visible cels' objects) and `render`'s
    /// World cull. The eye's cel comes from the persistent `Camera_CheckLocation` (0x1bae90) state
    /// via `scene::eye_cell`; the walk from that cell is `Vision_Portal_Recurse` 0x1e86e8 with the
    /// game's depth-0 planes. `world` (the sim's scoped collision) feeds `build_FindCel` 0x1c8cb0's
    /// collision tie-break when the placement re-finds. `None` (the walk cannot run: no portals /
    /// no cel) = the fix-render3 box test everywhere, and `NF_NOPORTAL` forces it (the walk is the
    /// default again, fix-portal).
    /// Upload the burning woman's current frame (crate::woman) into her HUD texture.
    pub fn update_woman(&self, queue: &wgpu::Queue, rgba: &[u8]) {
        let Some((tex, w, h)) = self.woman_tex.as_ref() else { return };
        if rgba.len() != (*w * *h * 4) as usize { return; }
        let size = wgpu::Extent3d { width: *w, height: *h, depth_or_array_layers: 1 };
        queue.write_texture(
            wgpu::TexelCopyTextureInfo { texture: tex, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
            rgba, wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(4 * w), rows_per_image: Some(*h) }, size);
    }

    pub fn cel_visibility(&mut self, eye: [f32; 3], view_proj: [[f32; 4]; 4], world: Option<&nf_game::world::CollisionWorld>, doors: Option<&[bool]>) -> Option<Vec<bool>> {
        let planes = extract_frustum(view_proj);
        let portal_enabled = room_cull_enabled() && std::env::var_os("NF_NOPORTAL").is_none();
        // The cell-level game mask (`View_AddCels` 0x1e6ba0: the VisCelList plus the +0x18 chain
        // pass under `Vision_InView`) gates the World draws; the strict VisCelList mask gates the
        // rooms/objects (`View_DrawObjList` 0x1e69a0 walks the VisCelList cels' object lists —
        // chain nodes never enter the VisCelList). fix-portal2.
        let (cell_vis, strict_vis) = if portal_enabled && !self.cel_graph.portals.is_empty() {
            let depth0 = Self::depth0_game(&planes, eye);
            let sides = [planes[0], planes[1], planes[2], planes[3]];
            match crate::scene::eye_cell(&self.cel_graph, &mut self.viewer_cel, eye, world) {
                Some(cell) => {
                    let mut game = crate::scene::portal_visible_cels_game_from_cell_doors(&self.cel_graph, cell, eye, &depth0, &sides, doors);
                    // fix-portal-sweep: the near-LOS rescue (scene::near_los_rescue) — the game-mask
                    // pass restores the near unoccluded generics the walk drops where the port's
                    // per-cel attribution approximation (the FindCel 0x1c8cb0 tie-break input)
                    // disagrees with the game's. No-op without a collision world.
                    if let (Some(m), Some(w)) = (game.as_mut(), world) {
                        let rescued = crate::scene::near_los_rescue(&self.cel_graph, m, eye, &sides, w, 30.0);
                        if rescued > 0 && std::env::var_os("NF_PORTAL_DEBUG").is_some() {
                            eprintln!("[rescue] eye cell {cell}: {rescued} near cels restored");
                        }
                    }
                    (game, crate::scene::portal_visible_cels_from_cell_doors(&self.cel_graph, cell, eye, &depth0, doors))
                }
                None => (None, None),
            }
        } else { (None, None) };
        // cull-relax (objects and rooms vanished in doorways and
        // between areas): every cel one portal beyond the walk's set draws too, so a room being
        // stepped into or looked through is never cut at the threshold. Port choice, not the game's
        // walk — the walk is exact only where the port's cel attribution is, and the port ranks
        // nothing-missing above the culled frames (the draws stay frustum-culled per draw).
        let widen = |m: Option<Vec<bool>>| m.map(|mut v| {
            let seed = v.clone();
            for pe in &self.cel_graph.portals {
                if pe.cel_a < seed.len() && pe.cel_b < seed.len() {
                    if seed[pe.cel_a] { v[pe.cel_b] = true; }
                    if seed[pe.cel_b] { v[pe.cel_a] = true; }
                }
            }
            v
        });
        let (cell_vis, strict_vis) = (widen(cell_vis), widen(strict_vis));
        self.last_cell_vis = cell_vis;
        let vis = strict_vis.map(|cell_vis| self.cel_graph.room_to_cell.iter().map(|m| m.map(|c| cell_vis[c]).unwrap_or(true)).collect::<Vec<bool>>());
        let vis = vis.unwrap_or_else(|| self.cel_boxes.iter().map(|&(min, max)| !box_outside(&planes, min, max)).collect());
        self.last_vis = Some(vis.clone());
        Some(vis)
    }

    /// G-043: upload one frame's skinning palette and colour table (replaces re-uploading every posed vertex).
    /// `vis`: the frame's room visibility mask (see `Scene::skin_palette`).
    pub fn update_skin(&self, queue: &wgpu::Queue, scene: &Scene, choice: Option<usize>, time: f32, opts: crate::scene::ViewOpts, vis: Option<&[bool]>) {
        let (pal, cols) = scene.skin_palette(&self.skin_layout, choice, time, opts, vis);
        if !pal.is_empty() { queue.write_buffer(&self.pal_buf, 0, bytemuck::cast_slice(&pal)); }
        *self.skin_pos.borrow_mut() = pal.iter().map(|m| [m[3][0], m[3][1], m[3][2]]).collect();
        queue.write_buffer(&self.col_buf, 0, bytemuck::cast_slice(&cols));
        self.upload_morph_weights(queue, scene);
    }

    /// The shot path's companion to `update_skin` (it takes `&self`): the per-character light centres.
    pub fn set_char_light_centres(&mut self, scene: &Scene) { self.char_light_centre = scene.char_light_centres(); }

    /// fix-gpu-facial-morphs: one frame's facial morph weights into the weight buffer (the GPU half
    /// of `SkinIt` 0x10bf30's Morph_Weight marshalling). No-op on scenes without morph bearers.
    pub fn upload_morph_weights(&self, queue: &wgpu::Queue, scene: &Scene) {
        let wts = scene.skin_morph_weights(&self.skin_layout);
        if !wts.is_empty() { queue.write_buffer(&self.wts_buf, 0, bytemuck::cast_slice(&wts)); }
    }

    /// Upload an already-built (possibly interpolated) palette and colour table.
    pub fn upload_skin(&self, queue: &wgpu::Queue, pal: &[nf_formats::anim::Mat4], cols: &[[f32; 4]]) {
        if !pal.is_empty() { queue.write_buffer(&self.pal_buf, 0, bytemuck::cast_slice(pal)); }
        { let mut sp = self.skin_pos.borrow_mut(); sp.clear(); sp.extend(pal.iter().map(|m| [m[3][0], m[3][1], m[3][2]])); }
        if !cols.is_empty() { queue.write_buffer(&self.col_buf, 0, bytemuck::cast_slice(cols)); }
    }

    /// `view_proj`: full camera; `view_proj_rot`: the same without the camera translation.
    /// Rewrites the Character-layer tail of the vertex buffer (starting at vertex `start`).
    pub fn update_vertices(&self, queue: &wgpu::Queue, start: usize, vertices: &[crate::scene::Vertex]) {
        if vertices.is_empty() { return; }
        queue.write_buffer(&self.vbuf, (start * std::mem::size_of::<crate::scene::Vertex>()) as u64, bytemuck::cast_slice(vertices));
    }

    /// `hud`: overlay triangles drawn last.
    /// `eye`: the camera position (the per-frame draw sort: list 0 front to back, list 1 back to front by the objects'
    /// squared distance, as `View_AddCels` orders cels).
    ///
    /// A-11e: CPU-side frustum cull. Before the draw loop the 6 frustum planes are extracted from `view_proj`
    /// (Gribb-Hartmann); each draw's bounding sphere is tested, and draws fully outside the frustum are skipped
    /// (`continue`) — no pipeline, bind group or draw_indexed call is issued for them. Sky draws (pass 4, drawn after
    /// the fog pass with the rotation-only uniform) follow the viewer and are never culled. The per-frame `order` Vec
    /// is `clear`ed + refilled (not reallocated); `set_pipeline`/`set_bind_group(0)` are skipped when the value is
    /// unchanged from the previous draw.
    /// `pillar` = the game's 4:3 cut-scene band [x, y, w, h] in target pixels (fix-cutscene-letterbox):
    /// while a script camera runs every pass — world, fog, sky, snow and the HUD overlay — draws only
    /// inside that band (`set_viewport`), the projection already narrowed to the game's aspect by the
    /// caller (`crate::cutscene_frame`); the area outside stays at the black clear. The game rendered
    /// every cut-scene at the console's own 4:3 frame, so a wider window pillar-boxes instead of
    /// widening the view (gameplay stays widescreen — the caller passes None there). A movie frame
    /// ignores the band: the FMV owns the whole screen [M 0x209e88].
    /// break-shards draw: this frame's glass shards (`Scene::shard_draws` — scene texture index,
    /// world vertices), uploaded before `render`.
    pub fn set_shards(&mut self, queue: &wgpu::Queue, groups: &[(usize, Vec<crate::scene::Vertex>)]) {
        self.shard_groups.clear();
        let mut all: Vec<crate::scene::Vertex> = Vec::new();
        for (ti, v) in groups.iter().take(SHARD_GROUPS) {
            let n = v.len().min(SNOW_MAX - all.len()) / 3 * 3;
            if n == 0 { break; }
            self.shard_groups.push((ti + 1, all.len() as u32, n as u32));
            all.extend_from_slice(&v[..n]);
        }
        if !all.is_empty() { queue.write_buffer(&self.shard_vbuf, 0, bytemuck::cast_slice(&all)); }
    }

    pub fn render(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, target: &wgpu::TextureView, depth: &wgpu::TextureView, view_proj: [[f32; 4]; 4], view_proj_rot: [[f32; 4]; 4], eye: [f32; 3], ticks: u32, snow: &[crate::scene::Vertex], hud: &HudFrame, movie: Option<(u32, u32, &[u8])>, pillar: Option<[f32; 4]>) -> wgpu::CommandEncoder {
        let ns = snow.len().min(SNOW_MAX);
        if ns > 0 { queue.write_buffer(&self.snow_vbuf, 0, bytemuck::cast_slice(&snow[..ns])); }
        let n = hud.v.len().min(HUD_MAX) / 3 * 3;
        if n > 0 { queue.write_buffer(&self.hud_vbuf, 0, bytemuck::cast_slice(&hud.v[..n])); }
        // Fog constants: N1 = `Ps2Near1` 8, n = `Ps2Near2` 0.1, f = `Ps2Far2` 2^24 [L]; F1 per level.
        let (fog_rgb, fog_on, k) = match self.fog {
            Some((c, f1)) => ([c[0] as f32 / 255.0, c[1] as f32 / 255.0, c[2] as f32 / 255.0], 1.0, (f1 - 8.0) * 0.1 / (16_777_216.0 - 0.1)),
            None => ([0.0; 3], 0.0, 0.0),
        };
        let extra = [fog_rgb[0], fog_rgb[1], fog_rgb[2], fog_on, 8.0, k, 16_777_216.0, GAIN];
        // The frame starts as fog (what the fog pass covers where nothing was drawn: zb 0); the
        // pillar-box clears black so the sides of a cut-scene frame are the void around the game's
        // 4:3 picture (the fog pass repaints the band's background itself).
        let clear = if (pillar.is_some() && movie.is_none()) || (movie.is_some() && self.screen_aspect > 0.0) { wgpu::Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 } }
            else if fog_on > 0.0 { wgpu::Color { r: (fog_rgb[0] * GAIN) as f64, g: (fog_rgb[1] * GAIN) as f64, b: (fog_rgb[2] * GAIN) as f64, a: 1.0 } } else { wgpu::Color { r: 0.05, g: 0.06, b: 0.08, a: 1.0 } };
        let mut enc = device.create_command_encoder(&Default::default());
        {
            let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target, depth_slice: None, resolve_target: None,
                    ops: wgpu::Operations { load: wgpu::LoadOp::Clear(clear), store: wgpu::StoreOp::Store },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    // CreateZClearSprite 0x117ce8's job: the game clears z through a fullscreen write-z-only
                    // sprite (TEST_1 0x32001, AFAIL ZB_ONLY) chained by psiClearZ 0x10b9a0; here the pass clear.
                    view: depth, depth_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Clear(1.0), store: wgpu::StoreOp::Discard }), stencil_ops: None,
                }),
                timestamp_writes: None, occlusion_query_set: None, multiview_mask: None,
            });
            if let Some([x, y, w, h]) = pillar.filter(|_| movie.is_none()) {
                pass.set_viewport(x, y, w, h, 0.0, 1.0);
            }
            pass.set_vertex_buffer(0, self.vbuf.slice(..));
            pass.set_vertex_buffer(1, self.skin_buf.slice(..));
            pass.set_index_buffer(self.ibuf.slice(..), wgpu::IndexFormat::Uint32);
            use crate::scene::Layer;
            let d2 = |c: [f32; 3]| (c[0] - eye[0]).powi(2) + (c[1] - eye[1]).powi(2) + (c[2] - eye[2]).powi(2);
            // The 6 frustum planes for the CPU-side cull (Gribb-Hartmann from `view_proj`).
            let planes = extract_frustum(view_proj);
            // fix-cel-order: the room cels' −90000 + 10·k list-0 keys are handed out over the chain
            // `Vision_Portal_Recurse` (0x1e86e8) threads per frame from the depth-sorted `VisCelList`
            // (`QuickSort` 0x1d30a8 comparator[3] @0x1d3430: ascending portal depth, ties in the
            // game's own unstable order). Compute that chain for THIS eye and map each room cel
            // (`Material::node`) to its k. When the walk cannot run (no graph / the eye's room
            // unmapped) the build-time parse-rank fallback (`Material::chain`) stands in. Ordering
            // only — the cull below is untouched (NF_PORTAL still gates the cull walk).
            let chain_ord: Option<Vec<Option<u32>>> = if !self.cel_graph.portals.is_empty() {
                // The same resolved viewer cel + the game's depth-0 planes the walk used (the
                // chain order is the VisCelList the game threaded from ITS walk [M 0x1e86e8]).
                let depth0 = Self::depth0_game(&planes, eye);
                self.viewer_cel.cell.and_then(|cell| crate::scene::portal_cel_chain_from_cell(&self.cel_graph, cell, eye, &depth0))
                    .map(|chain| {
                        let mut ord: Vec<Option<u32>> = vec![None; self.cel_graph.cells.len()];
                        for (k, c) in chain.into_iter().enumerate() { if c < ord.len() { ord[c] = Some(k as u32); } }
                        ord
                    })
            } else { None };
            // fix-render2: the World draws' bounding spheres are exact — static geometry through the
            // placement matrix (scene.rs add_instance) — so they cull always. Everything posed per
            // frame keeps its build-time centre (doors swinging, pickups, the truck, searchlight
            // parts, muzzle flashes, tracers — the Character/Effect layers) and the sky follows the
            // viewer, so those only cull under the NF_CULL test override. The HUD draws from its own
            // buffer below and is never in this list. This was A-11's vanish bug: the full cull made
            // the truck and the HUD disappear by view direction; scoping the cull to
            // the static World layer keeps every moving thing drawn while still skipping the bulk —
            // 07000006's start view submits ~4k draws unculled (render2_live counts them).
            let test_cull = std::env::var_os("NF_CULL").is_some();
            // blood-puff-render: the rule moved to `Layer::frustum_cullable` (the Effect pools'
            // build-time sphere is the identity-matrix origin — culling it dropped every puff the
            // moment the origin left the frustum, wherever they were posed).
            let cull = |dc: &DrawCall| dc.layer.frustum_cullable(test_cull);
            // Counters for the worklog (env-gated; no per-frame cost otherwise).
            let trace = std::env::var_os("NF_DRAWS").is_some();
            let (mut submitted, mut culled) = (0usize, 0usize);
            // fix-render3: the room-cel visibility test (`View_AddCels` 0x1e6ba0's `Vision_InView`
            // per cel bbox). Computed once per frame — ~200 sphere tests; a World draw whose cel is
            // out is skipped before the pipeline/bind work. The eye's own cel always passes (its
            // bbox contains the eye, so its sphere can never be outside the frustum).
            let cel_cull = room_cull_enabled() && std::env::var_os("NF_NOCELCULL").is_none();
            // fix-perf-level: the visibility set is the game's own portal walk
            // (`Vision_Portal_Recurse` 0x1e86e8 → `Vision_Recurse` 0x1e82e0, the GfxList set) from the
            // eye's room cel, with the depth-0 pyramid = the frustum's four side planes + near (the
            // game's five `Vision_Calculate_View_Fustrum` 0x1e7df8 planes). Cels the walk cannot run
            // for (no graph / no portals / eye outside every cel) keep the fix-render3 box test.
            // fix-portal: the walk is the default again (the collapse is fixed: the viewer cel is
            // `Camera_CheckLocation` 0x1bae90's persistent state and the depth-0 pyramid is the
            // game's [M 0x1e7df8]); `NF_NOPORTAL` disables it. The mask is computed once per frame
            // by `cel_visibility` (the skin palette shares it) — reuse it when fresh.
            let cel_visible: Vec<bool> = if cel_cull {
                // The lazy fallback never fires in the live paths (cel_visibility runs first — lib.rs 1496 / shot());
                // it has no `world` here, so the FindCel tie-break degrades to the approximation.
                self.last_vis.clone().unwrap_or_else(|| self.cel_visibility(eye, view_proj, None, None).unwrap())
            } else { Vec::new() };
            // fix-portal2: the cell-level game mask. A cel placement's glist draws when ITS cel is
            // visible (`View_AddCels` 0x1e6ba0 — the VisCelList cel itself or a `+0x18` chain node
            // under `Vision_InView`); `Material::node` = the placement's own cell for every
            // creates_cel placement (u32::MAX = the distance-keyed objects, gated by their
            // containing room as before).
            let cell_visible: Option<&Vec<bool>> = self.last_cell_vis.as_ref();
            // portal-orphans (07000015: the silo tunnel walls were missing): a node no
            // room's chain owns and no portal reaches can never be marked by the walk, so the cel
            // cull dropped it from every position. The game draws its unlinked objects from the
            // forced list (`View_AddForcedObjects` 0x1e5fb8) — they take only the frustum test.
            let orphan = &self.orphan_nodes;
            let mut cel_culled = 0usize;
            // perf-culldiff: read the diagnostic switch ONCE — the per-draw `env::var_os` (a locked
            // GetEnvironmentVariableW, ~1 µs) ran for every cel-culled draw and cost 4-5 ms a frame on
            // 07000007/12/15 (thousands of cel-culled draws), pushing them under 240 fps.
            static CULLDIFF: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
            let culldiff = *CULLDIFF.get_or_init(|| std::env::var_os("NF_CULLDIFF").is_some());
            // Skip redundant `set_pipeline` calls (A-11e). The group-0 bind now carries a per-draw
            // dynamic offset (fix-lighting: each draw's own light pair), so it is issued every draw.
            let mut last_pipeline: Option<usize> = None;
            let mut last_tex: Option<usize> = None;
            let (mut __pipe_sw, mut __tex_sw) = (0usize, 0usize);
            // Debug: `NF_SKIP_MAT=i,j,..` skips those material indices (bisecting a wrong-looking surface).
            static SKIP: std::sync::OnceLock<Vec<usize>> = std::sync::OnceLock::new();
            let skip = SKIP.get_or_init(|| std::env::var("NF_SKIP_MAT").map(|v| v.split(',').filter_map(|t| t.trim().parse().ok()).collect()).unwrap_or_default());
            // Reuse the cached `order` Vec (A-11e): clear + refill, no per-frame alloc.
            self.order.clear();
            let prof_cull = nf_game::prof::scope("enc: cull filter");
            // frame-240: the cull runs BEFORE the sort and the uniform staging, so the per-frame
            // work (light picks, uniform slots, the sort) scales with the submitted draws, not with
            // every draw the level owns (the effect pools alone are thousands of parked slots).
            let draws = &self.draws;
            // The destroyed-glist swap's draw side (see the field doc): a dead Destroy leaf's
            // materials — matched by the build-time centre bits — drop out of the frame.
            let destroy_dead = &self.destroy_dead;
            let parked_pos = self.skin_pos.borrow();
            let layout = &self.skin_layout;
            self.order.extend(draws.iter().enumerate().filter(|&(k, dc)| {
                if skip.contains(&k) { return false; }
                if !destroy_dead.is_empty() && dc.layer == Layer::World
                    && destroy_dead.contains(&dc.centre.map(f32::to_bits)) { return false; }
                // perf-240: a skinned draw whose owner slot holds the FAR park matrix (an empty effect /
                // projectile pool slot, a room-culled character — `skin_palette`'s FAR: zero 3×3,
                // 10000 below the world) draws only a degenerate point far under the map: skip it.
                // The same test the light pick uses (`p[1] > -9000`).
                if let Some((ci, att)) = dc.skin {
                    let idx = match att { None => layout.char_base.get(ci as usize).copied(), Some(a) => Some(layout.attach_base + a) };
                    if idx.and_then(|i| parked_pos.get(i as usize)).is_some_and(|p| p[1] <= -9000.0) {
                        if trace { culled += 1; }
                        return false;
                    }
                }
                // Sky after the fog pass (pass 4) follows the viewer and is never culled.
                if cull(dc) && dc.pass != 4 && sphere_outside(&planes, dc.centre, dc.radius) {
                    if trace { culled += 1; }
                    return false;
                }
                if cel_cull && dc.layer == Layer::World {
                    let hidden = match (cell_visible, dc.node) {
                        (Some(cv), n) if n != u32::MAX => !cv[n as usize] && !orphan.get(n as usize).copied().unwrap_or(false),
                        _ => dc.cel.is_some_and(|c| !cel_visible[c as usize]),
                    };
                    if hidden {
                        if trace { cel_culled += 1; }
                        // Dev diagnostic: `NF_CULLDIFF=1` names the near, in-frustum World draws the room
                        // cull removes (the cel/node the walk failed to reach).
                        if culldiff && d2(dc.centre) < 400.0 + dc.radius * dc.radius && !sphere_outside(&planes, dc.centre, dc.radius) {
                            eprintln!("[culldiff] draw {k} node {} cel {:?} chain {} centre {:?} r {:.2}", dc.node, dc.cel, dc.chain, dc.centre, dc.radius);
                        }
                        return false;
                    }
                }
                true
            }).map(|(k, dc)| {
                // List 0 (pass 1): the room cels' −90000 + 10·k (`View_AddCels` 0x1e6ba0 over the
                // per-frame depth-sorted chain, `Material::node` → k; `chain` = the build-time
                // parse-rank fallback when the walk cannot run), every distance-keyed entry (the
                // +0x18 inner-node statics and `View_AddObjects`' objects) by squared distance.
                // List 1 (pass 3): back-to-front (−dist²) for everything. Ties fall through to the
                // draw index (the material order = the build's group order, i.e. leaf/batch order
                // within one object).
                let key = match dc.pass {
                    1 => if dc.chain != u32::MAX {
                        chain_ord.as_ref().and_then(|o| o.get(dc.node as usize).copied().flatten())
                            .map(|k2| -90_000.0 + 10.0 * k2 as f32)
                            .unwrap_or_else(|| -90_000.0 + 10.0 * dc.chain as f32)
                    } else { d2(dc.centre) },
                    3 => -d2(dc.centre),
                    _ => 0.0,
                };
                // fix-0c-pylon: the frame slot order is Game_Draw's (0x1e6f50, decompiled): sky group 1
                // (View_DrawSky v,1 → GfxList+0x4a == 1), list 0 (View_DrawObjList v,0), then
                // psiPostSolid 0x121630 = psiDrawFogPass 0x10b960 and sky group 0 (View_DrawSky v,0 →
                // GfxList+0x4a == 0) — and ONLY THEN list 1 (View_DrawObjList v,1), the alpha list the
                // pylon rides. The port drew its post-fog sky (pass 4) after the alpha list; that sky is
                // pinned to the far plane with a LessEqual test, so it repainted every pixel where only
                // no-depth-write alpha-list draws happened (the 0700000c pylon: zwrite off, against
                // sky/sea — erased; its baked shadow survived because it writes depth). Slot map:
                // pass 0 (pre-world sky) → 0, pass 1 (list 0) → 2, pass 4 (post-fog sky) → 3,
                // pass 2 (characters) → 4, pass 3 (alpha list) → 6, pass 5 (effects) → 10, pass 6 (sort list 2, drawn
                // after `psiClearZ` [M Game_Draw 0x1e6f50:162-175] — no depth test) → 12.
                let slot = match dc.pass { 4 => 3, p => p * 2 };
                (slot, key, k)
            }));
            drop(parked_pos);
            drop(prof_cull);
            let prof_sort = nf_game::prof::scope("enc: sort");
            self.order.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.total_cmp(&b.1)).then(a.2.cmp(&b.2)));
            // Per-draw uniform slots (fix-lighting): the game uploads the ≤2 closest lights per object
            // (`View_DrawObjList` 0x1e69a0 / `View_DrawCelGlist` 0x1e6538 → `Lights_CalcClosestLights`
            // 0x1cb2e0 + `psiLight_SetLights` 0x10e588). Slot 0 = the eye's own sphere (the snow pass — the
            // game's particle path inherits the previous upload and the flakes have no normals, so the
            // pick is inert); slot j+1 = the j-th SUBMITTED draw (frame-240: compact, one buffer — a
            // camera-rotation sky draw carries view_proj_rot in its own slot). fix-lighting2: the per-draw
            // flags are the source object's glist flags word (`Control_SetGList` 0x134960 → obj+0xF0 /
            // cel+0x94), not the hardcoded 0x80 that let the kind-0 ambient radiators crowd the muzzle
            // light off the world draws; the sky draws upload NO lights (`View_DrawSky` 0x1e5ad0 /
            // `View_DrawSprites` 0x1e5290 call `psiLight_SetLights(0,0,0,0,0)`).
            drop(prof_sort);
            let prof_lights = nf_game::prof::scope("enc: uniforms + light picks");
            {
                const HEAD: usize = 40;
                const SLOT: usize = 20;
                let mut staged = std::mem::take(&mut self.staged);
                staged.clear();
                staged.resize(HEAD + SLOT * (self.order.len() + 1 + self.shard_groups.len()), 0.0);
                // break-shards draw: one slot per shard group after the draws — no lights, the
                // group's texture as the array index (batch-draws).
                for (g, &(tf, _, _)) in self.shard_groups.iter().enumerate() {
                    let (fb, nf, rate) = self.tex_frames[tf];
                    staged[HEAD + (self.order.len() + 1 + g) * SLOT + 17] = (fb + ((ticks / rate) % nf) as usize) as f32;
                }
                for c in 0..4 { staged[c * 4..c * 4 + 4].copy_from_slice(&view_proj[c]); staged[16 + c * 4..20 + c * 4].copy_from_slice(&view_proj_rot[c]); }
                staged[32..40].copy_from_slice(&extra);
                {
                    let s = &mut staged[HEAD..HEAD + SLOT];
                    // batch-draws: slot 0 (the snow pass) samples the snow texture from the array.
                    s[17] = self.batch.as_ref().map_or(0.0, |b| b.snow_tex as f32);
                    let (lp, lc) = self.lights.pick2([eye[0], eye[1], eye[2], 2.0], 0x80);
                    for (i, v) in lp.iter().chain(lc.iter()).enumerate() { s[i * 4..i * 4 + 4].copy_from_slice(v); }
                }
                let skin_pos = self.skin_pos.borrow();
                for (j, &(_, _, k)) in self.order.iter().enumerate() {
                    let dc = &self.draws[k];
                    let s = &mut staged[HEAD + (j + 1) * SLOT..][..SLOT];
                    if dc.layer == Layer::SkyCamera { s[16] = 1.0; }
                    // batch-draws: the draw's texture (animated frames included) as an index into the array.
                    { let (fb, nf, rate) = self.tex_frames[dc.tex]; s[17] = (fb + ((ticks / rate) % nf) as usize) as f32; }
                    if matches!(dc.layer, Layer::SkyCamera | Layer::SkyFixed) {
                        // The game zeroes the light slots for the sky and sprite passes [M 0x1e5ad0:89].
                        continue;
                    }
                    // light-viewmodel: the view model IS lit. `Game_Draw` zeroes the lights before list 2
                    // [M 0x1e6f50:150], but every animated object draws through `AnimObjectDraw`
                    // [M 0x1c4490:117-122], which re-picks: `View_SetupRenderModes` at the object's
                    // sphere [M 0x1e78b0:21-23] or `View_SetupRenderModes2` at the viewer's camera with
                    // radius 2.0 [M 0x1e7908:28-36]. The hands and gun sit at the eye either way — the
                    // pick centres on the eye, radius 2.0, with the draw's own flags word.
                    if dc.node == crate::scene::VIEW_MODEL_NODE && dc.layer != Layer::World {
                        let (lp, lc) = self.lights.pick2([eye[0], eye[1], eye[2], 2.0], dc.lflags as u32);
                        for (i, v) in lp.iter().chain(lc.iter()).enumerate() { s[i * 4..i * 4 + 4].copy_from_slice(v); }
                        continue;
                    }
                    // A skinned draw picks its lights at its owner's LIVE root (the palette slot the GPU skins
                    // with) — the game's per-object obj+0x80 sphere [M 0x1e69a0:73-81]; a hidden owner (the FAR
                    // collapse) picks none. Static draws keep their build centre.
                    let centre = match dc.skin {
                        Some((ci, att)) => {
                            let idx = match att { None => self.skin_layout.char_base.get(ci as usize).copied(), Some(a) => Some(self.skin_layout.attach_base + a) };
                            match idx.and_then(|i| skin_pos.get(i as usize).copied()) {
                                // light-actors: a script actor's object sits at the origin while its
                                // sequence root carries the body [Script_AnimStart 0x1d8f40's origin
                                // create; AnimSeqTick 0x1c28d0 root motion] — the game's per-object
                                // sphere (obj+0x80) follows the body, so the pick centres there.
                                Some(p) if p[1] > -9000.0 => match att {
                                    None => self.char_light_centre.get(ci as usize).copied().flatten().unwrap_or(p),
                                    Some(_) => p,
                                },
                                Some(_) => continue,
                                None => dc.centre,
                            }
                        }
                        None => dc.centre,
                    };
                    let (lp, lc) = self.lights.pick2([centre[0], centre[1], centre[2], dc.radius], dc.lflags as u32);
                    for (i, v) in lp.iter().chain(lc.iter()).enumerate() { s[i * 4..i * 4 + 4].copy_from_slice(v); }
                }
                drop(skin_pos);
                queue.write_buffer(&self.uniform, 0, bytemuck::cast_slice(&staged));
                self.staged = staged;
            }
            drop(prof_lights);
            let _prof_rec = nf_game::prof::scope("enc: record draws");
            pass.set_bind_group(0, &self.uniform_bg, &[]);
            if let Some(b) = self.batch.as_mut() {
                // batch-draws: every texture is bound at once, so only a pipeline change splits the
                // stream; each run of equal pipelines (in the draw order — the blend order is kept) is
                // ONE indirect multi-draw whose per-draw first_instance selects the uniform slot.
                pass.set_bind_group(1, &b.bg, &[]);
                b.args.clear();
                for (j, &(_, _, k)) in self.order.iter().enumerate() {
                    let dc = &self.draws[k];
                    b.args.extend_from_slice(&[dc.count, 1, dc.first, 0, j as u32 + 1]);
                }
                if !b.args.is_empty() { queue.write_buffer(&b.indirect, 0, bytemuck::cast_slice(&b.args)); }
                let mut start = 0usize;
                while start < self.order.len() {
                    let pl = self.draws[self.order[start].2].pipeline;
                    let mut end = start + 1;
                    while end < self.order.len() && self.draws[self.order[end].2].pipeline == pl { end += 1; }
                    pass.set_pipeline(&self.pipelines[pl].1);
                    pass.multi_draw_indexed_indirect(&b.indirect, start as u64 * 20, (end - start) as u32);
                    if trace { __pipe_sw += 1; }
                    start = end;
                }
                if trace { submitted = self.order.len(); }
            } else {
                for (j, &(_, _, k)) in self.order.iter().enumerate() {
                    let dc = self.draws[k];
                    if trace { submitted += 1; }
                    let (first, count) = (dc.first, dc.count);
                    if trace && last_pipeline != Some(dc.pipeline) { __pipe_sw += 1; }
                    if last_pipeline != Some(dc.pipeline) {
                        pass.set_pipeline(&self.pipelines[dc.pipeline].1);
                        last_pipeline = Some(dc.pipeline);
                    }
                    let (first_bg, n_frames, rate) = self.tex_frames[dc.tex];
                    // frame-240: a texture bind only when it changes (the sort keeps a material's draws together).
                    let tbg = first_bg + ((ticks / rate) % n_frames) as usize;
                    if trace && last_tex != Some(tbg) { __tex_sw += 1; }
                    if last_tex != Some(tbg) {
                        pass.set_bind_group(1, &self.tex_bgs[tbg], &[]);
                        last_tex = Some(tbg);
                    }
                    pass.draw_indexed(first..first + count, 0, (j as u32 + 1)..(j as u32 + 2));
                }
            }
            if trace {
                eprintln!("render: pipe switches {__pipe_sw} tex switches {__tex_sw}; {submitted} draws submitted, {culled} frustum-culled, {cel_culled} cel-culled ({} total, {:.1}% skipped)",
                    submitted + culled + cel_culled, 100.0 * (culled + cel_culled) as f32 / (submitted + culled + cel_culled).max(1) as f32);
            }
            // `DrawDrops` runs after the fog pass, the alpha list and the shards.
            if ns > 0 {
                pass.set_pipeline(&self.snow_pipeline);
                pass.set_bind_group(0, &self.uniform_bg, &[]);
                match self.batch.as_ref() { Some(b) => pass.set_bind_group(1, &b.bg, &[]), None => pass.set_bind_group(1, &self.snow_bg, &[]) }
                pass.set_vertex_buffer(0, self.snow_vbuf.slice(..));
                pass.set_vertex_buffer(1, self.snow_skin.slice(..));
                pass.draw(0..ns as u32, 0..1);
            }
            // break-shards draw: `DrawAllShards` 0x12bef8 runs before `DrawDrops` [M Game_Draw] — the
            // shards after the alpha list, each texture group in its own uniform slot.
            if !self.shard_groups.is_empty() {
                pass.set_pipeline(&self.shard_pipeline);
                pass.set_bind_group(0, &self.uniform_bg, &[]);
                pass.set_vertex_buffer(0, self.shard_vbuf.slice(..));
                pass.set_vertex_buffer(1, self.snow_skin.slice(..));
                let base = self.order.len() as u32 + 1;
                for (g, &(tf, first, count)) in self.shard_groups.iter().enumerate() {
                    match self.batch.as_ref() {
                        Some(b) => pass.set_bind_group(1, &b.bg, &[]),
                        None => { let (fb, nf, rate) = self.tex_frames[tf]; pass.set_bind_group(1, &self.tex_bgs[fb + ((ticks / rate) % nf) as usize], &[]); }
                    }
                    pass.draw(first..first + count, base + g as u32..base + g as u32 + 1);
                }
            }
            // fix-menu-movies2: the ONE movie plane draws BEFORE the HUD runs. The game uploads
            // the decoded frame to VRAM and the page draws it as its own background texture
            // [M PS2UploadBGMImage 0x108740; movies.md §Presentation — "a full-screen background
            // texture, not an IPU overlay"], i.e. UNDER the menu controls; the GameFlow state-6
            // briefing draws over the empty backdrop with the overlay suppressed, so drawing
            // before the (empty) HUD runs is the same picture there.
            // The game's upload path is the PATH3 tag-builder family (unported-misc-1: its only
            // user in the whole binary is this BG FMV upload; docs/systems/fmv-plumbing.md):
            // DecodeBGFMVFrame 0x108550 builds chains via setLoadImageTags 0x107b90 →
            // setLoadImageTagsYX 0x107910 → setDMAscTag 0x107bd0 / setGIFtag 0x107c20 /
            // setBITBLTBUF 0x107c70 / setTRXPOS 0x107ca0 / setTRXREG 0x107cd0 / setTRXDIR
            // 0x107d08 (16×16 CT32 blocks), and the INTC handler kicks them with
            // loadImage 0x107d30 (D2_CHCR=0x105) from PS2UploadBGMImage — all replaced by this
            // wgpu write_texture into the full-screen plane's texture.
            if let Some((w, h, rgba)) = movie {
                if self.movie_size != Some((w, h)) || self.movie_bg.is_none() {
                    let size = wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 };
                    let t = device.create_texture(&wgpu::TextureDescriptor {
                        label: None, size, mip_level_count: 1, sample_count: 1, dimension: wgpu::TextureDimension::D2,
                        format: wgpu::TextureFormat::Rgba8Unorm, usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST, view_formats: &[],
                    });
                    let view = t.create_view(&Default::default());
                    self.movie_bg = Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
                        label: None, layout: &self.hud_tbgl,
                        entries: &[wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&view) }, wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&self.hud_sampler) }],
                    }));
                    self.movie_tex = Some(t);
                    self.movie_size = Some((w, h));
                }
                if let Some(tex) = &self.movie_tex {
                    let sx = if self.screen_aspect > 0.0 { ((4.0 / 3.0) / self.screen_aspect).min(1.0) } else { 1.0 };
                    let v = |x: f32, y: f32, u: f32, w: f32| HudVertex { p: [x * sx, y], uv: [u, w], c: [1.0; 4] };
                    queue.write_buffer(&self.movie_vbuf, 0, bytemuck::cast_slice(&[
                        v(-1.0, 1.0, 0.0, 0.0), v(1.0, 1.0, 1.0, 0.0), v(1.0, -1.0, 1.0, 1.0),
                        v(-1.0, 1.0, 0.0, 0.0), v(1.0, -1.0, 1.0, 1.0), v(-1.0, -1.0, 0.0, 1.0)]));
                    queue.write_texture(
                        wgpu::TexelCopyTextureInfo { texture: tex, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
                        rgba, wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(4 * w), rows_per_image: Some(h) },
                        wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 });
                    pass.set_pipeline(&self.hud_pipeline);
                    pass.set_bind_group(0, self.movie_bg.as_ref().unwrap(), &[]);
                    pass.set_vertex_buffer(0, self.movie_vbuf.slice(..));
                    pass.draw(0..6, 0..1);
                }
            }
            if n > 0 {
                pass.set_vertex_buffer(0, self.hud_vbuf.slice(..));
                for &(tex, blend, first, count) in &hud.runs {
                    let end = (first + count).min(n as u32);
                    if first >= end { continue; }
                    pass.set_pipeline(match blend { 1 => &self.hud_add_pipeline, 2 => &self.hud_sub_pipeline, _ => &self.hud_pipeline });
                    pass.set_bind_group(0, &self.hud_bgs[tex.min(self.hud_bgs.len() - 1)], &[]);
                    pass.draw(first..end, 0..1);
                }
            }
        }
        enc
    }
}
pub fn depth_view(device: &wgpu::Device, w: u32, h: u32) -> wgpu::TextureView {
    device.create_texture(&wgpu::TextureDescriptor {
        label: None, size: wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 }, mip_level_count: 1, sample_count: 1,
        dimension: wgpu::TextureDimension::D2, format: DEPTH, usage: wgpu::TextureUsages::RENDER_ATTACHMENT, view_formats: &[],
    }).create_view(&Default::default())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A view_proj for the camera `Camera::view_proj` produces: at the origin (0, 0, 0) looking down +z, 60° vertical
    /// FOV, 16:9 aspect, near 0.05 far 4000 (the live values from `lib.rs` and `Camera::view_proj`). Column-major,
    /// stored as `[[col0], [col1], [col2], [col3]]` — `vp[i]` is the i-th column.
    fn vp_origin_looking_z() -> [[f32; 4]; 4] {
        // proj columns: (f/aspect, 0, 0, 0), (0, f, 0, 0), (0, 0, far/(near-far), -1), (0, 0, near*far/(near-far), 0)
        // — f = 1/tan(30°) ≈ 1.7320508, aspect = 16/9, so f/aspect ≈ 0.9742786.
        // view columns (yaw=0, pitch=0, eye=origin): (-1, 0, 0, 0), (0, 1, 0, 0), (0, 0, -1, 0), (0, 0, 0, 1).
        // view_proj = proj * view → columns as below (worked out by hand).
        [
            [-0.9742786, 0.0, 0.0, 0.0],          // col 0
            [0.0, 1.7320508, 0.0, 0.0],            // col 1
            [0.0, 0.0, 1.0000125, 1.0],           // col 2
            [0.0, 0.0, -0.05, 0.0],               // col 3
        ]
    }

    #[test]
    fn frustum_cull_keeps_a_sphere_in_front_of_the_camera() {
        let vp = vp_origin_looking_z();
        let planes = extract_frustum(vp);
        // 10 units in front of the camera along +z, radius 1: should be inside the frustum.
        assert!(!sphere_outside(&planes, [0.0, 0.0, 10.0], 1.0), "in-frustum sphere was culled");
    }

    #[test]
    fn frustum_cull_drops_a_sphere_behind_the_camera() {
        let vp = vp_origin_looking_z();
        let planes = extract_frustum(vp);
        // 10 units BEHIND the camera (−z), small radius: should be culled.
        assert!(sphere_outside(&planes, [0.0, 0.0, -10.0], 0.5), "behind-camera sphere was kept");
    }

    #[test]
    fn frustum_cull_drops_a_sphere_far_off_to_the_side() {
        let vp = vp_origin_looking_z();
        let planes = extract_frustum(vp);
        // 1000 units to +x (outside the 60° FOV cone), small radius: should be culled.
        assert!(sphere_outside(&planes, [1000.0, 0.0, 10.0], 0.5), "off-axis sphere was kept");
    }

    #[test]
    fn frustum_cull_keeps_a_large_sphere_even_just_past_the_far_plane() {
        let vp = vp_origin_looking_z();
        let planes = extract_frustum(vp);
        // Centre 5000 units away (past far = 4000), but radius 2000 → intersects the far plane, must NOT be culled.
        assert!(!sphere_outside(&planes, [0.0, 0.0, 5000.0], 2000.0), "large sphere intersecting the far plane was culled");
        // Same centre, tiny radius → fully outside the far plane, must be culled.
        assert!(sphere_outside(&planes, [0.0, 0.0, 5000.0], 1.0), "small sphere past the far plane was kept");
    }

    #[test]
    fn frustum_cull_planes_are_normalised() {
        let vp = vp_origin_looking_z();
        let planes = extract_frustum(vp);
        for p in &planes {
            let n = (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt();
            assert!((n - 1.0).abs() < 1e-5, "plane normal not unit length: {n}");
        }
    }
}

#[cfg(test)]
mod shader_tests {
    /// The WGSL must parse and validate offline (no GPU adapter needed). fix-lighting touched the main shader's uniform struct and
    /// light block — a syntax or validation error would break every draw at Renderer::new.
    #[test]
    fn the_shaders_parse_and_validate() {
        for (name, src) in [("main", super::SHADER), ("hud", super::HUD_SHADER)] {
            let module = naga::front::wgsl::parse_str(src).unwrap_or_else(|e| panic!("{name} WGSL parse: {e}"));
            naga::valid::Validator::new(naga::valid::ValidationFlags::all(), naga::valid::Capabilities::all())
                .validate(&module).unwrap_or_else(|e| panic!("{name} WGSL validate: {e}"));
        }
    }

    /// fix-lighting2: the light add is the game's byte-scale `TColor.rgb · term` over the Unorm8x4
    /// (byte/255) vertex colour — /255, not /128 — and the summed colour clamps at byte 255 like the
    /// microcode's MINIi (I = 255.0, .vutext I-mem 919/954). The old /128 over-added by exactly
    /// 255/128 ≈ 2× (the view-model saturation the merge flagged) and never clamped.
    #[test]
    fn the_light_add_is_in_byte_255_units_with_the_minii_clamp() {
        assert!(super::SHADER.contains("min(c.rgb + lc.rgb * term / 255.0, vec3<f32>(1.0))"),
            "the shader must add the byte-scale light over the byte/255 vertex colour and clamp to byte 255");
        assert!(!super::SHADER.contains("term / 128.0"), "the old 2× over-add must stay gone");
    }
}

/// The room/portal cull is ON for the MAP GEOMETRY only (cull-on, 2026-10-18; characters and items are
/// never room-culled): Nightfire's maps overlap rooms in space and the portal walk is
/// what keeps them apart — with it off, 07000011's stairwell drew a neighbouring room's
/// brown wall slab straight across the corridor (the map cutting into itself). The walk is widened one portal (cull-relax) so a room being stepped into is never
/// cut at the threshold. `NF_NOROOMCULL=1` turns it off (debug only).
pub fn room_cull_enabled() -> bool { std::env::var_os("NF_NOROOMCULL").is_none() }
