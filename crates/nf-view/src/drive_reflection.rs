//! The driving view's per-frame car reflection map (docs/formats/car-model.md §10.5 pass 3,
//! §10.6.3, §10.6.4): one 256 × 256 map per frame, drawn from the reflection camera over the
//! player car, that every reflective car strip samples.
//!
//! - [`DriveReflection`] (on `DrivingState`) holds the static inputs: the CarRender globals, the
//!   track's `Map in#0` items with their geometry, the vehicles' draws, the layer steps.
//! - [`DriveReflection::fill`] builds this frame's [`MapFrame`] (camera, kept items, vehicles) from
//!   `nf_formats::rendertun` (`CarRender::reflection_camera`, `ReflectionProjection::clip`,
//!   `ReflectionItemQuery::keeps`), and `DrivingState::reflection_frame` adds the car's reflection
//!   overlay (bondrm.o pass 3) to the frame's additive groups.
//! - [`ReflGpu`] (inside `gpu::Renderer`) renders the map into its own render target before the
//!   main pass; the map is a scene texture, so the car overlay samples it like any other texture.
//!
//! The map is drawn in the game's own texel space ([`ReflectionProjection::map_texel`]): the view
//! rows are the documented (R, U, F), whose R × U = +F mirror image is kept as is, and the main
//! view's `Scene::mirror_x` is not applied. The car's reflection ST (`carmesh::reflection_vertex`)
//! read the map in that same space.

use crate::scene::{Draw, Scene, TextureImage, Vertex};
use nf_formats::carmesh::CarPassKind;
use nf_formats::rendertun::{
    CarRender, ReflectionItemQuery, ReflectionProjection, ReflectionStep, ReflectionView, REFLECTION_FOG, REFLECTION_MAP_SIZE,
    REFLECTION_VEHICLES,
};

/// The map's texel format: the port's textures are Rgba8Unorm with 0x80 = 1.0 in the shader.
pub const MAP_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
/// At most this many draws go into the map per frame (one uniform slot each).
pub const MAP_SLOTS: usize = 1024;
/// The map's scene texture key: `(0, MAP_KEY)`.
pub const MAP_KEY: u32 = 0xfe50_0000;
/// The track items' draw state in the map: the track's own (`driving.rs` push_strip), fog off.
pub const ITEM_DRAW: Draw = Draw { pass: 1, blend: 0, zwrite: true, atest: 0, fog: 0 };
/// The full-map layer quads: no depth test or write; ALPHA 0x44 = (Cs − Cd)·As + Cd (blend 0).
pub const QUAD_DRAW: Draw = Draw { pass: 6, blend: 0, zwrite: false, atest: 0, fog: 0 };
/// The `rmsk` quad: ALPHA 0x62 = Cd − Cs (blend 2, a reverse subtract).
pub const SUB_DRAW: Draw = Draw { pass: 6, blend: 2, zwrite: false, atest: 0, fog: 0 };

/// Which index buffer a map draw reads.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MapSource {
    /// [`MapScene::indices`]: the track items, grouped by item.
    Items,
    /// The scene's own index buffer: a vehicle's materials.
    Scene,
}

/// One indexed draw into the map.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MapDraw {
    /// The scene texture index + 1 (the renderer's `tex_frames` index; 0 = white).
    pub tex: usize,
    pub first: u32,
    pub count: u32,
    pub source: MapSource,
    pub draw: Draw,
}

/// One step of the map pass ([`CarRender::reflection_steps`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MapStep {
    /// A full-map quad: texture (`tex_frames` index), vertex colour bytes, the `rmsk` subtraction.
    Quad { tex: usize, rgba: [u8; 4], subtract: bool },
    /// The kept items, then the vehicles ([`MapFrame::draws`]).
    Scene,
}

/// What the renderer draws into the map this frame.
#[derive(Default)]
pub struct MapFrame {
    pub active: bool,
    /// The reflection camera (`steps` left empty: they are [`MapFrame::steps`]).
    pub view: Option<ReflectionView>,
    /// Column-major, wgpu clip space: `(clip.x, −clip.y, (clip.z + clip.w) / 2, clip.w)` of
    /// [`ReflectionProjection::clip`], so map row 0 is the top (texel y down).
    pub view_proj: [[f32; 4]; 4],
    pub steps: Vec<MapStep>,
    /// The scene step's draws: the kept items in table order, then the vehicles.
    pub draws: Vec<MapDraw>,
    /// `Map in#0` indices of the kept items.
    pub kept_items: Vec<usize>,
    /// Attachment slots of the drawn vehicles.
    pub vehicles: Vec<u32>,
}

/// The map's scene-side data (`Scene::drive_reflection`).
pub struct MapScene {
    /// The map's index in `Scene::textures` (a 256 × 256 placeholder the renderer replaces with
    /// its render target).
    pub tex: usize,
    /// The track items' triangles (indices into the scene's vertices), grouped by item.
    pub indices: Vec<u32>,
    pub frame: MapFrame,
}

/// One track item the reflection query tests.
#[derive(Clone, Debug)]
pub struct ReflItem {
    /// `Map in#0` index.
    pub index: usize,
    /// `+0x30`.
    pub pos: [f32; 3],
    /// `+0x3c` (`rendertun::item_bounds`).
    pub bounds: u32,
    /// `+0x0c`.
    pub flags: u8,
    /// Range into [`DriveReflection::item_draws`].
    pub draws: std::ops::Range<usize>,
}

/// The reflection pass's static inputs (`DrivingState::reflection`).
pub struct DriveReflection {
    pub render: CarRender,
    pub items: Vec<ReflItem>,
    pub item_draws: Vec<MapDraw>,
    /// (attachment slot, range into `vehicle_draws`).
    pub vehicles: Vec<(u32, std::ops::Range<usize>)>,
    pub vehicle_draws: Vec<MapDraw>,
    /// The map's scene texture index.
    pub map_tex: usize,
    /// Indices into `CarShine::strips` of the strips with a reflection pass and Reflect > 0.
    pub car_strips: Vec<usize>,
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] { [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]] }
fn dot(a: [f32; 3], b: [f32; 3]) -> f32 { a[0] * b[0] + a[1] * b[1] + a[2] * b[2] }
fn normalize(v: [f32; 3]) -> [f32; 3] {
    let l = dot(v, v).sqrt();
    if l > 0.0 { v.map(|c| c / l) } else { v }
}

/// The map's view-projection for the renderer (column-major, `m[col][row]`): for a world point p,
/// `m · (p, 1)` = `(cx, −cy, (cz + cw) / 2, cw)` with `(cx, cy, cz, cw)` =
/// `view.projection.clip(view.to_view(p))`. wgpu puts NDC y = +1 on row 0, so texel row
/// `128 + 128·cy/cw` (y down, `ReflectionProjection::map_texel`) is where the point lands; depth
/// runs 0 at the near plane to 1 at the far plane.
pub fn map_view_proj(view: &ReflectionView) -> [[f32; 4]; 4] {
    let pr = &view.projection;
    let k = 1.0 / (pr.fov_deg * 0.5).to_radians().tan();
    let (n, f) = (pr.near, pr.far);
    let a = -(f + n) / (f - n);
    let b = f * (a + 1.0);
    let e = view.eye;
    // Rows as (linear part, constant): with d = p − eye, vz = −d·F.
    // x: k·aspect… (aspect 1 on x in clip(): k·vx), y: k·aspect·vy, w: d·F, z: ((1 − a)/2)·d·F + b/2.
    let ky = k * pr.aspect;
    let zf = (1.0 - a) * 0.5;
    let rows: [([f32; 3], f32); 4] = [
        (view.right.map(|c| c * k), -k * dot(view.right, e)),
        (view.up.map(|c| c * ky), -ky * dot(view.up, e)),
        (view.forward.map(|c| c * zf), -zf * dot(view.forward, e) + 0.5 * b),
        (view.forward, -dot(view.forward, e)),
    ];
    let mut m = [[0.0f32; 4]; 4];
    for (r, (lin, c)) in rows.iter().enumerate() {
        for col in 0..3 {
            m[col][r] = lin[col];
        }
        m[3][r] = *c;
    }
    m
}

impl DriveReflection {
    /// Builds the pass's inputs and adds the map (and `cref` / `rmsk` when not already there) to
    /// `scene.textures`. `track_strips` = per drawn track strip `(in#0 item, scene texture index,
    /// first index, index count)` into `scene.indices` (sky domes left out); `cref_track` = the
    /// track sheet's `cref` texture when it has one; `read` reads a decompressed archive member.
    /// `None` when the track's Map items do not decode.
    pub fn build(
        scene: &mut Scene,
        crp_bytes: &[u8],
        track_strips: &[(usize, Option<usize>, u32, u32)],
        render: CarRender,
        cref_track: Option<usize>,
        read: &dyn Fn(&str) -> Option<Vec<u8>>,
        shine: Option<&crate::driving::CarShine>,
        player_slots: &[u32],
    ) -> Option<DriveReflection> {
        let crp = nf_formats::crp::Crp::parse(crp_bytes).ok()?;
        let placements = nf_formats::geoprim::map_placements(&crp).ok()?;
        let words = nf_formats::geoprim::map_item_word15(&crp).ok()?;
        // The track items with geometry, in table order; an item the flags test drops is never
        // kept (`ReflectionItemQuery::keeps` rule 4), so its geometry is left out.
        let mut order: Vec<usize> = (0..track_strips.len()).collect();
        order.sort_by_key(|&i| (track_strips[i].0, track_strips[i].1, i));
        let by_index: std::collections::HashMap<usize, &nf_formats::geoprim::Placement> = placements.iter().map(|p| (p.index, p)).collect();
        let (mut items, mut item_draws, mut indices) = (Vec::new(), Vec::<MapDraw>::new(), Vec::<u32>::new());
        let mut k = 0;
        while k < order.len() {
            let item = track_strips[order[k]].0;
            let start = item_draws.len();
            let keep = by_index.get(&item).filter(|p| p.flags & 0x0d == 0).copied();
            while k < order.len() && track_strips[order[k]].0 == item {
                let (_, tex, first, count) = track_strips[order[k]];
                k += 1;
                let Some(_) = keep else { continue };
                let tf = tex.map_or(0, |t| t + 1);
                let at = indices.len() as u32;
                indices.extend_from_slice(&scene.indices[first as usize..(first + count) as usize]);
                let merge = item_draws.len() > start && item_draws.last().is_some_and(|d| d.tex == tf && d.first + d.count == at);
                match item_draws.last_mut() {
                    Some(d) if merge => d.count += count,
                    _ => item_draws.push(MapDraw { tex: tf, first: at, count, source: MapSource::Items, draw: ITEM_DRAW }),
                }
            }
            if let (Some(p), Some(&w)) = (keep, words.get(item)) {
                if item_draws.len() > start {
                    items.push(ReflItem { index: item, pos: p.origin, bounds: w, flags: p.flags, draws: start..item_draws.len() });
                }
            }
        }
        // The textures: the track sheet's `cref`, else `data\render\ext.ssh`'s [M FUN_001c1358];
        // `rmsk` from `data\render\common.ssh` [M FUN_001c0930], loaded only when `Desat ?` is on.
        let sheet_tex = |scene: &mut Scene, file: &str, name: &str, key: u32| -> Option<usize> {
            let b = read(&["data", "render", file].join("\\"))?;
            let t = nf_formats::ssh::Ssh::parse(&b).ok()?.textures.into_iter().find(|t| t.name == name)?;
            let i = scene.textures.len();
            scene.textures.push(TextureImage { key: (0, key | i as u32), width: t.width, height: t.height, rgba: t.rgba, more: Vec::new(), rate: 1 });
            Some(i)
        };
        let cref = cref_track.or_else(|| sheet_tex(scene, "ext.ssh", "cref", 0xfe51_0000));
        let rmsk = if render.desat_on { sheet_tex(scene, "common.ssh", "rmsk", 0xfe52_0000) } else { None };
        let n = REFLECTION_MAP_SIZE;
        let map_tex = scene.textures.len();
        scene.textures.push(TextureImage { key: (0, MAP_KEY), width: n, height: n, rgba: vec![0; (4 * n * n) as usize], more: Vec::new(), rate: 1 });
        let steps: Vec<MapStep> = render
            .reflection_steps()
            .into_iter()
            .filter_map(|s| match s {
                ReflectionStep::Scene => Some(MapStep::Scene),
                ReflectionStep::Layer(l) => {
                    let tex = match l.texture { "cref" => cref, "rmsk" => rmsk, _ => None }?;
                    Some(MapStep::Quad { tex: tex + 1, rgba: l.rgba, subtract: l.alpha_reg & 0xff == 0x62 })
                }
            })
            .collect();
        // The vehicles: each Character-layer material's attachment slot (its first vertex's).
        let mut per_slot: Vec<(u32, Vec<MapDraw>)> = Vec::new();
        for m in &scene.materials {
            if m.layer != crate::scene::Layer::Character || m.index_count == 0 {
                continue;
            }
            let Some(&v) = scene.indices.get(m.first_index as usize) else { continue };
            let Some(cv) = (v as usize).checked_sub(scene.char_vertex_start).and_then(|i| scene.char_verts.get(i)) else { continue };
            let Some(slot) = cv.attachment else { continue };
            let tex = m.key.and_then(|k| scene.textures.iter().position(|t| t.key == k)).map_or(0, |i| i + 1);
            let d = MapDraw { tex, first: m.first_index, count: m.index_count, source: MapSource::Scene, draw: m.draw };
            match per_slot.iter_mut().find(|(s, _)| *s == slot) {
                Some((_, v)) => v.push(d),
                None => per_slot.push((slot, vec![d])),
            }
        }
        let (mut vehicles, mut vehicle_draws) = (Vec::new(), Vec::new());
        // A car is one material per strip: its triangles are copied into the map's index buffer
        // grouped by (state, texture), one draw each.
        for (slot, mut d) in per_slot {
            if player_slots.contains(&slot) {
                continue;
            }
            d.sort_by_key(|x| (x.draw, x.tex));
            let s = vehicle_draws.len();
            for x in d {
                let at = indices.len() as u32;
                indices.extend_from_slice(&scene.indices[x.first as usize..(x.first + x.count) as usize]);
                let merge = vehicle_draws.len() > s && vehicle_draws.last().is_some_and(|l: &MapDraw| l.tex == x.tex && l.draw == x.draw && l.first + l.count == at);
                match vehicle_draws.last_mut() {
                    Some(l) if merge => l.count += x.count,
                    _ => vehicle_draws.push(MapDraw { tex: x.tex, first: at, count: x.count, source: MapSource::Items, draw: x.draw }),
                }
            }
            vehicles.push((slot, s..vehicle_draws.len()));
        }
        // The car strips the reflection pass draws: a material with a Reflection pass (Texture,
        // TexturePacked, Chrome, GlassPacked [M bondrm.o, `CarMaterial::pass_slots`]) and Reflect > 0.
        let car_strips = shine
            .map(|sh| {
                (0..sh.strips.len())
                    .filter(|&i| {
                        let (st, props) = &sh.strips[i];
                        props.reflect > 0.0 && st.passes().iter().any(|p| p.kind == CarPassKind::Reflection)
                    })
                    .collect()
            })
            .unwrap_or_default();
        scene.drive_reflection = Some(MapScene { tex: map_tex, indices, frame: MapFrame { steps, ..Default::default() } });
        Some(DriveReflection { render, items, item_draws, vehicles, vehicle_draws, map_tex, car_strips })
    }

    /// The reflection camera for the player car at `car_pos` and the main view's forward axis
    /// `main_forward` (`CarRender::reflection_view` without its per-call step list).
    pub fn view(&self, car_pos: [f32; 3], main_forward: [f32; 3]) -> ReflectionView {
        let (eye, forward) = self.render.reflection_camera(car_pos, main_forward);
        let right = normalize(cross([0.0, 0.0, -1.0], forward));
        let up = normalize(cross(forward, right));
        ReflectionView {
            eye,
            right,
            up,
            forward,
            projection: ReflectionProjection { fov_deg: self.render.refl_angle, aspect: 1.0, near: 1.0, far: 2500.0 },
            items: ReflectionItemQuery { centre: car_pos },
            vehicles: REFLECTION_VEHICLES,
            fog: REFLECTION_FOG,
            steps: Vec::new(),
        }
    }

    /// This frame's map: the camera, the kept items (table order) and up to 4 vehicles, nearest to
    /// `eye` first. `vehicles` = (position, attachment slot) of the vehicles that may be drawn.
    pub fn fill(&self, out: &mut MapFrame, car_pos: [f32; 3], main_forward: [f32; 3], eye: [f32; 3], vehicles: &mut dyn Iterator<Item = ([f32; 3], u32)>) {
        let view = self.view(car_pos, main_forward);
        out.view_proj = map_view_proj(&view);
        out.draws.clear();
        out.kept_items.clear();
        out.vehicles.clear();
        for it in &self.items {
            if view.items.keeps(it.pos, it.bounds, it.flags) {
                out.kept_items.push(it.index);
                out.draws.extend_from_slice(&self.item_draws[it.draws.clone()]);
            }
        }
        // The 4 slots keep the smallest LOD measures [M FUN_001c85f8 → FUN_001c2508]; the measure
        // here is the distance to the main view's eye [S: FUN_001ad0b8's scaling is not ported].
        let mut best: [(f32, u32); REFLECTION_VEHICLES] = [(f32::MAX, u32::MAX); REFLECTION_VEHICLES];
        for (p, slot) in vehicles {
            let d = [p[0] - eye[0], p[1] - eye[1], p[2] - eye[2]];
            let m = dot(d, d);
            if let Some(i) = best.iter().position(|b| m < b.0) {
                best.copy_within(i..REFLECTION_VEHICLES - 1, i + 1);
                best[i] = (m, slot);
            }
        }
        for &(m, slot) in &best {
            if m == f32::MAX {
                break;
            }
            if let Some((_, r)) = self.vehicles.iter().find(|(s, _)| *s == slot) {
                out.vehicles.push(slot);
                out.draws.extend_from_slice(&self.vehicle_draws[r.clone()]);
            }
        }
        out.draws.truncate(MAP_SLOTS);
        out.view = Some(view);
        out.active = true;
    }
}

impl crate::driving::DrivingState {
    /// This frame's reflection map ([`DriveReflection::fill`] into `scene.drive_reflection`) and the
    /// car's reflection pass (bondrm.o pass 3, docs/formats/car-model.md §10.5): per vertex
    /// ST = (0.5 + v.x, 0.5 + v.z), RGB = min(Reflect·C_w, 100000·max(0, v.y)) on the map, added
    /// over the base (`name=envmap` ALPHA 0x48 = Cs·As + Cd), vertex alpha 127. Call after
    /// `particle_groups` with the same camera axes; the group goes right after the shine's.
    pub fn reflection_frame(&self, scene: &mut Scene, right: [f32; 3], up: [f32; 3]) {
        let (Some(rf), Some(ms)) = (self.reflection.as_ref(), scene.drive_reflection.as_mut()) else { return };
        // The main view's forward axis (camera + 0x20, into the scene), as the shine takes it.
        let f = cross(up, right);
        let eye = self.cam_view.as_ref().map_or(self.car.pos, |v| v.eye);
        let mut veh = self.ai_cars.iter().map(|(d, s)| (d.car.pos, *s));
        rf.fill(&mut ms.frame, self.car.pos, f, eye, &mut veh);
        let map_tex = ms.tex;
        let Some(sh) = self.shine.as_ref() else { return };
        if rf.car_strips.is_empty() {
            return;
        }
        let r = self.car.rot;
        let c = self.car_box_centre;
        let place = |q: [f32; 3]| [0, 1, 2].map(|k| self.car.pos[k] + r[0][k] * (q[0] - c[0]) + r[1][k] * (q[1] - c[1]) + r[2][k] * (q[2] - c[2]));
        let ident = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
        let mut out = Vec::new();
        for &si in &rf.car_strips {
            let (st, props) = &sh.strips[si];
            let pv = st.pass_vertices(CarPassKind::Reflection, &r, &ident, &sh.entry, *props);
            if pv.len() != st.positions.len() || pv.len() < 3 {
                continue;
            }
            let vert = |k: usize| {
                let g = pv[k].rgb.min(255) as u8;
                Vertex { pos: place(st.positions[k]), uv: pv[k].st, rgba: [g, g, g, 127], nrm: [0; 4] }
            };
            for k in 0..pv.len() - 2 {
                out.extend_from_slice(&[vert(k), vert(k + 1), vert(k + 2)]);
            }
        }
        if out.is_empty() {
            return;
        }
        let at = scene.drive_particles.iter().position(|g| g.0.is_some() && g.0 == sh.tex).map_or(0, |i| i + 1);
        scene.drive_particles.insert(at, (Some(map_tex), true, out));
    }
}

/// The map's GPU side, owned by `gpu::Renderer`.
pub struct ReflGpu {
    /// The 256 × 256 map (sampled by the car overlay through the scene texture table).
    pub target: wgpu::Texture,
    view: wgpu::TextureView,
    depth: wgpu::TextureView,
    ibuf: wgpu::Buffer,
    uniform: wgpu::Buffer,
    ubg: wgpu::BindGroup,
    /// The bindless array with the map's own slot swapped for the white texture (a pass may not
    /// sample the texture it renders to).
    batch_bg: Option<wgpu::BindGroup>,
    pipelines: Vec<(Draw, wgpu::RenderPipeline)>,
    quad_vbuf: wgpu::Buffer,
    staged: Vec<f32>,
    /// (pipeline, texture bind index, op, slot).
    ops: Vec<(usize, usize, ReflOp, u32)>,
    pub active: bool,
}

#[derive(Clone, Copy)]
enum ReflOp {
    /// Quad vertices `6q .. 6q + 6` of `quad_vbuf`.
    Quad(u32),
    Indexed(MapSource, u32, u32),
}

const HEAD: usize = 48;
const SLOT: usize = 20;
const QUADS: usize = 4;

/// The map texture: (texture, its bind group for the per-texture path, the sampled view).
pub fn map_target(device: &wgpu::Device, tbgl: &wgpu::BindGroupLayout, sampler: &wgpu::Sampler) -> (wgpu::Texture, wgpu::BindGroup, wgpu::TextureView) {
    let n = REFLECTION_MAP_SIZE;
    let t = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("reflection map"),
        size: wgpu::Extent3d { width: n, height: n, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: MAP_FORMAT,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let v = t.create_view(&Default::default());
    let bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("reflection map"),
        layout: tbgl,
        entries: &[wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&v) }, wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(sampler) }],
    });
    (t, bg, v)
}

impl ReflGpu {
    /// `skin` = the four storage buffers of the main uniform group (palette, colours, morph
    /// weights, morph targets); `make` builds a main-shader pipeline for [`MAP_FORMAT`].
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        target: wgpu::Texture,
        scene: &Scene,
        ubgl: &wgpu::BindGroupLayout,
        skin: [&wgpu::Buffer; 4],
        batch_bg: Option<wgpu::BindGroup>,
        make: &dyn Fn(Draw) -> wgpu::RenderPipeline,
    ) -> ReflGpu {
        let ms = scene.drive_reflection.as_ref().expect("a reflection scene");
        let view = target.create_view(&Default::default());
        let n = REFLECTION_MAP_SIZE;
        let depth = crate::gpu::depth_view(device, n, n);
        let idx: &[u32] = if ms.indices.is_empty() { &[0, 0, 0] } else { &ms.indices };
        let ibuf = wgpu::util::DeviceExt::create_buffer_init(device, &wgpu::util::BufferInitDescriptor { label: Some("reflection items"), contents: bytemuck::cast_slice(idx), usage: wgpu::BufferUsages::INDEX });
        let _ = queue;
        let uniform = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("reflection uniforms"),
            size: ((HEAD + SLOT * (MAP_SLOTS + QUADS)) * 4) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let ubg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("reflection uniforms"),
            layout: ubgl,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: uniform.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: skin[0].as_entire_binding() },
                wgpu::BindGroupEntry { binding: 2, resource: skin[1].as_entire_binding() },
                wgpu::BindGroupEntry { binding: 3, resource: skin[2].as_entire_binding() },
                wgpu::BindGroupEntry { binding: 4, resource: skin[3].as_entire_binding() },
            ],
        });
        let mut states = vec![ITEM_DRAW, QUAD_DRAW, SUB_DRAW];
        for m in &scene.materials {
            if m.layer == crate::scene::Layer::Character && !states.contains(&m.draw) {
                states.push(m.draw);
            }
        }
        let pipelines = states.into_iter().map(|d| (d, make(d))).collect();
        let quad_vbuf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("reflection quads"),
            size: (QUADS * 6 * std::mem::size_of::<Vertex>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        ReflGpu { target, view, depth, ibuf, uniform, ubg, batch_bg, pipelines, quad_vbuf, staged: Vec::new(), ops: Vec::with_capacity(MAP_SLOTS + QUADS), active: false }
    }

    /// Stages this frame's map: the uniforms (the map's view-projection; identity for the quads;
    /// fog off; gain 1.0), the quads' vertices and the draw list. `tex_frames` maps a texture
    /// (`tex_frames` index) to its first bind index; animated frames stay on frame 0.
    pub fn set_frame(&mut self, queue: &wgpu::Queue, frame: Option<&MapFrame>, tex_frames: &[(usize, u32, u32)]) {
        self.ops.clear();
        let Some(f) = frame.filter(|f| f.active) else {
            self.active = false;
            return;
        };
        let mut staged = std::mem::take(&mut self.staged);
        staged.clear();
        let quads = f.steps.iter().filter(|s| matches!(s, MapStep::Quad { .. })).count().min(QUADS);
        staged.resize(HEAD + SLOT * (f.draws.len().min(MAP_SLOTS) + quads), 0.0);
        for c in 0..4 {
            staged[c * 4..c * 4 + 4].copy_from_slice(&f.view_proj[c]);
            staged[16 + c * 4 + c] = 1.0;
        }
        // us[8] = fog (off), us[9] = (N1, k, f, gain): the map is a texture, not the display.
        staged[36..40].copy_from_slice(&[8.0, 0.0, 16_777_216.0, 1.0]);
        let pipe = |d: Draw| self.pipelines.iter().position(|(s, _)| *s == d);
        let tbind = |tf: usize| tex_frames.get(tf).map_or(0, |t| t.0);
        let mut quad_v = [Vertex { pos: [0.0; 3], uv: [0.0; 2], rgba: [0; 4], nrm: [0; 4] }; QUADS * 6];
        let (mut slot, mut q) = (0u32, 0u32);
        for s in &f.steps {
            match *s {
                MapStep::Quad { tex, rgba, subtract } => {
                    if q as usize >= QUADS {
                        continue;
                    }
                    let Some(p) = pipe(if subtract { SUB_DRAW } else { QUAD_DRAW }) else { continue };
                    let b = tbind(tex);
                    let at = HEAD + slot as usize * SLOT;
                    staged[at + 16] = 1.0;
                    staged[at + 17] = b as f32;
                    // Full map: NDC (−1, 1) is texel (0, 0) with uv (0, 0).
                    let v = |x: f32, y: f32| Vertex { pos: [x, y, 0.5], uv: [(x + 1.0) * 0.5, (1.0 - y) * 0.5], rgba, nrm: [0; 4] };
                    let c = [v(-1.0, 1.0), v(1.0, 1.0), v(1.0, -1.0), v(-1.0, 1.0), v(1.0, -1.0), v(-1.0, -1.0)];
                    quad_v[q as usize * 6..q as usize * 6 + 6].copy_from_slice(&c);
                    self.ops.push((p, b, ReflOp::Quad(q), slot));
                    slot += 1;
                    q += 1;
                }
                MapStep::Scene => {
                    for d in f.draws.iter().take(MAP_SLOTS) {
                        let Some(p) = pipe(d.draw) else { continue };
                        let b = tbind(d.tex);
                        staged[HEAD + slot as usize * SLOT + 17] = b as f32;
                        self.ops.push((p, b, ReflOp::Indexed(d.source, d.first, d.count), slot));
                        slot += 1;
                    }
                }
            }
        }
        staged.truncate(HEAD + SLOT * slot as usize);
        if q > 0 {
            queue.write_buffer(&self.quad_vbuf, 0, bytemuck::cast_slice(&quad_v[..q as usize * 6]));
        }
        queue.write_buffer(&self.uniform, 0, bytemuck::cast_slice(&staged));
        self.staged = staged;
        self.active = true;
    }

    /// Draws the map: clear, then the staged steps in order. `scene_ibuf` / `vbuf` / `skin` are the
    /// main buffers; `static_skin` a buffer of unskinned attributes (≥ 24 entries) for the quads;
    /// `tex_bgs` the per-texture bind groups (used when `batch` is false).
    #[allow(clippy::too_many_arguments)]
    pub fn encode(
        &self,
        enc: &mut wgpu::CommandEncoder,
        vbuf: &wgpu::Buffer,
        skin: &wgpu::Buffer,
        scene_ibuf: &wgpu::Buffer,
        static_skin: &wgpu::Buffer,
        tex_bgs: &[wgpu::BindGroup],
        batch: bool,
    ) {
        if !self.active {
            return;
        }
        let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("reflection map"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &self.view,
                depth_slice: None,
                resolve_target: None,
                // The clear value is not traced (car-model.md §10.7); black with alpha 1.0, so the
                // car pass's Cs·As keeps the map's colour.
                ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 }), store: wgpu::StoreOp::Store },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth,
                depth_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Clear(1.0), store: wgpu::StoreOp::Discard }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_bind_group(0, &self.ubg, &[]);
        if batch {
            if let Some(bg) = &self.batch_bg {
                pass.set_bind_group(1, bg, &[]);
            }
        }
        let (mut last_p, mut last_t, mut last_src) = (usize::MAX, usize::MAX, None::<Option<MapSource>>);
        for &(p, t, op, slot) in &self.ops {
            if p != last_p {
                pass.set_pipeline(&self.pipelines[p].1);
                last_p = p;
            }
            if !batch && t != last_t {
                if let Some(bg) = tex_bgs.get(t) {
                    pass.set_bind_group(1, bg, &[]);
                }
                last_t = t;
            }
            match op {
                ReflOp::Quad(q) => {
                    if last_src != Some(None) {
                        pass.set_vertex_buffer(0, self.quad_vbuf.slice(..));
                        pass.set_vertex_buffer(1, static_skin.slice(..));
                        last_src = Some(None);
                    }
                    pass.draw(q * 6..q * 6 + 6, slot..slot + 1);
                }
                ReflOp::Indexed(src, first, count) => {
                    if last_src != Some(Some(src)) {
                        if !matches!(last_src, Some(Some(_))) {
                            pass.set_vertex_buffer(0, vbuf.slice(..));
                            pass.set_vertex_buffer(1, skin.slice(..));
                        }
                        let ib = if src == MapSource::Items { &self.ibuf } else { scene_ibuf };
                        pass.set_index_buffer(ib.slice(..), wgpu::IndexFormat::Uint32);
                        last_src = Some(Some(src));
                    }
                    pass.draw_indexed(first..first + count, 0, slot..slot + 1);
                }
            }
        }
    }
}
