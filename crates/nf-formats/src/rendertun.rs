//! The driving tracks' render tuning text files (`data\tuning\Render\Fog\<track>.tun`,
//! `data\tuning\Render\Lighting\<track>.tun`, `data\tuning\Render\CarRender\default.tun`
//! (docs/formats/car-model.md §10), RefPack-packed in each `/DRIVING/*.VIV`): plain
//! `Key=value` lines, `Key{Instance}=…` for per-environment values. Data only; see
//! docs/formats/geoprim.md §7 for what reads them.
//!
//! The keys are the DRIVING.ELF tuner names without their spaces (`fog START` → `fogSTART`,
//! `Ambient Sky` → `AmbientSky`) [D: every key of the 8 tracks' files maps to one registered name
//! that way; the writer's `"%s=%s"` / `"%s{%s}=%s"` formats sit at ELF file offset 0x2669f4 — the
//! space stripping itself is not traced].

use crate::buf::{FormatError, Result};

/// `Key=value` pairs in file order (CR and blank lines dropped).
pub fn parse(text: &str) -> Vec<(String, String)> {
    text.lines()
        .filter_map(|l| {
            let l = l.trim_end_matches('\r');
            let (k, v) = l.split_once('=')?;
            Some((k.trim().to_owned(), v.trim().to_owned()))
        })
        .collect()
}

fn get<'a>(kv: &'a [(String, String)], key: &str) -> Result<&'a str> {
    kv.iter()
        .find(|(k, _)| k == key)
        .map(|(_, v)| v.as_str())
        .ok_or_else(|| FormatError::Invalid(format!("tuning key {key} missing")))
}

fn floats(v: &str) -> Result<Vec<f32>> {
    v.split(',')
        .map(|x| x.trim().parse::<f32>().map_err(|_| FormatError::Invalid(format!("tuning value {v:?} is not numeric"))))
        .collect()
}

fn f1(kv: &[(String, String)], key: &str) -> Result<f32> {
    floats(get(kv, key)?)?.first().copied().ok_or_else(|| FormatError::Invalid(format!("{key}: empty")))
}

fn f4(kv: &[(String, String)], key: &str) -> Result<[f32; 4]> {
    let v = floats(get(kv, key)?)?;
    if v.len() != 4 {
        return Err(FormatError::Invalid(format!("{key}: {} components, want 4", v.len())));
    }
    Ok([v[0], v[1], v[2], v[3]])
}

/// `Render\Fog\<track>.tun` — the `RFog` singleton's block [M `FUN_001ac668` 0x1ac668: a 0x2c-byte
/// block; `fog START` → +0x10 (1..5000), `fog END` → +0x24 (1..5000), `fog mode` → +0x1c (int 0..3),
/// `fog Density` → +0x18 (0..1), `Fog Colour` → +0x20 (packed)].
#[derive(Clone, Debug, PartialEq)]
pub struct Fog {
    pub start: f32,
    pub end: f32,
    /// Registered, not read by the apply step [M 0x1ac978]; its consumer is not traced.
    pub mode: i32,
    pub density: f32,
    /// The colour written to the GS `FOGCOL` register, R, G, B [M: `FUN_00144740` scans
    /// `"%d,%d,%d,%d"` into (a, b, c, d) and stores `a << 24 | d << 16 | c << 8 | b` — b unmasked, so
    /// a negative b ORs in its high bytes, as the snow tracks' files do; `FUN_001ac978` swaps bytes
    /// 0 and 2 and `FUN_0028cf98` swaps them back into GS register 0x3d (FOGCOL: R bits 0-7)].
    pub colour: [u8; 3],
}

/// A colour tuner value `a,b,c,d` packed the way the tuner stores it: `a << 24 | d << 16 | c << 8 |
/// b`, b unmasked [M `FUN_00144740`, the scanner of `FUN_00143ad8` / `FUN_001439a8`].
pub fn packed_colour(kv: &[(String, String)], key: &str) -> Result<u32> {
    let c: Vec<i64> = get(kv, key)?
        .split(',')
        .map(|x| x.trim().parse::<i64>().map_err(|_| FormatError::Invalid(format!("{key} {x:?}"))))
        .collect::<Result<_>>()?;
    if c.len() != 4 {
        return Err(FormatError::Invalid(format!("{key}: {} components, want 4", c.len())));
    }
    Ok(((c[0] as u32) << 24) | ((c[3] as u32) << 16) | ((c[2] as u32) << 8) | (c[1] as u32))
}

impl Fog {
    pub fn parse(text: &str) -> Result<Fog> {
        let kv = parse(text);
        let packed = packed_colour(&kv, "FogColour")?;
        Ok(Fog {
            start: f1(&kv, "fogSTART")?,
            end: f1(&kv, "fogEND")?,
            mode: f1(&kv, "fogmode")? as i32,
            density: f1(&kv, "fogDensity")?,
            colour: [packed as u8, (packed >> 8) as u8, (packed >> 16) as u8],
        })
    }

    /// The apply step's slope: `density × 255 / (end − start)` [M `FUN_001ac978`:
    /// `pfVar1[3] = (pfVar1[6] * 255.0) / (pfVar1[2] - *pfVar1)` with [0] = start, [2] = end,
    /// [6] = density]. How the VU1 code turns it into a per-vertex fog value is not traced.
    pub fn slope(&self) -> f32 { self.density * 255.0 / (self.end - self.start) }
}

/// One `{Instance}` of `Render\Lighting\<track>.tun` (`World`, `Character`, `Unused1`, `Unused2`)
/// [M `FUN_001ae630` 0x1ae630: the `RLightManager` registers 4 instances 0x70 bytes apart — `Ambient
/// Sky` +0xe0, `Ambient Ground` +0xd0, `Light 1` +0xb0, `Light 2` +0xc0 (RGBA floats), the angles
/// (degrees) in a 0x18-byte-stride table].
#[derive(Clone, Debug, PartialEq)]
pub struct LightEnv {
    pub ambient_sky: [f32; 4],
    pub ambient_ground: [f32; 4],
    pub light1: [f32; 4],
    pub light1_angle_y: f32,
    pub light1_angle_x: f32,
    pub light2: [f32; 4],
    pub light2_angle_y: f32,
    pub light2_angle_x: f32,
    pub dual_hemi_angle_y: f32,
    pub dual_hemi_angle_x: f32,
}

/// `Render\Lighting\<track>.tun`: the `World` and `Character` instances plus the globals. How
/// these combine with the world's per-vertex colours (geoprim `colors`) is not traced: the lights
/// need normals, which the colour-carrying world batches do not have [D], and the VU1 microcode
/// (eaglrm.o) is not disassembled.
#[derive(Clone, Debug, PartialEq)]
pub struct Lighting {
    pub world: LightEnv,
    pub character: LightEnv,
    pub enable_lightmaps: bool,
    pub lightmap_lighting_bias: f32,
    pub sun_height: f32,
    pub moon_size: f32,
}

impl Lighting {
    pub fn parse(text: &str) -> Result<Lighting> {
        let kv = parse(text);
        let env = |i: &str| -> Result<LightEnv> {
            let k = |n: &str| format!("{n}{{{i}}}");
            Ok(LightEnv {
                ambient_sky: f4(&kv, &k("AmbientSky"))?,
                ambient_ground: f4(&kv, &k("AmbientGround"))?,
                light1: f4(&kv, &k("Light1"))?,
                light1_angle_y: f1(&kv, &k("Light1AngleY"))?,
                light1_angle_x: f1(&kv, &k("Light1AngleX"))?,
                light2: f4(&kv, &k("Light2"))?,
                light2_angle_y: f1(&kv, &k("Light2AngleY"))?,
                light2_angle_x: f1(&kv, &k("Light2AngleX"))?,
                dual_hemi_angle_y: f1(&kv, &k("DualHemiAngleY"))?,
                dual_hemi_angle_x: f1(&kv, &k("DualHemiAngleX"))?,
            })
        };
        Ok(Lighting {
            world: env("World")?,
            character: env("Character")?,
            enable_lightmaps: f1(&kv, "Enablelightmaps")? != 0.0,
            lightmap_lighting_bias: f1(&kv, "Lightmaplightingbias")?,
            sun_height: f1(&kv, "Sunheight")?,
            moon_size: f1(&kv, "Moonsize")?,
        })
    }
}

/// The direction an `Angle X` / `Angle Y` pair (degrees) gives:
/// `(cos X · sin Y, −sin X, cos X · cos Y)` [M `FUN_001b0038` 0x1b0038: v = (0, 0, 1, 0), rotated
/// by `FUN_002d11c8(X, axis (1,0,0))` then `FUN_002d11c8(Y, axis (0,1,0))`, each applied as
/// `FUN_002d1920` (row vector × matrix); `FUN_002d11c8` builds the axis-angle matrix from
/// `angle / 360 × 2π`]. The tuner keeps X in −90..0 [M `FUN_001ae630`], so y ≥ 0: the vector
/// points from the surface up towards the light, and the car microcode lights a normal with
/// `max(0, N · dir)` as is (docs/formats/car-model.md §10).
pub fn light_direction(angle_x: f32, angle_y: f32) -> [f32; 3] {
    let (x, y) = (angle_x.to_radians(), angle_y.to_radians());
    [x.cos() * y.sin(), -x.sin(), x.cos() * y.cos()]
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] { [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]] }

fn normalize(v: [f32; 3]) -> [f32; 3] {
    let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if l > 0.0 {
        v.map(|c| c / l)
    } else {
        v
    }
}

/// The `SpecularMat` basis `[b, c, a]` the car specular pass projects world normals on
/// [M `FUN_001aefb8` 0x1aefb8, disassembled]: `a = normalize(L1 − f)` with `L1` = the Light 1
/// direction (column 0 of the LightSources rows) and `f` = the view's forward vector (also written
/// as column 3 of the LightSources rows); `b = normalize((0, 1, 0) × a)`; `c = normalize(a × b)`
/// (`VOPMULA`/`VOPMSUB` pairs at 0x1af114 and 0x1af150, each `fs × ft` with fs = (0,1,0) then a).
/// The four rows it stores at `+0x20` are `(b_i, c_i, a_i, 0)` for i = x, y, z, then (0, 0, 0, 1)
/// (`PEXTLW`/`PEXTUW`/`PCPYLD`/`PCPYUD` at 0x1af190), so a row-vector normal gives
/// `(n · b, n · c, n · a)`.
///
/// `f` [M]: the caller passes `camera + 0x20`, the camera being `**(DAT_00333760 + 4)` (the current
/// view's `+0` object, a 0xb0-byte camera with FOV at `+0xa4`, `FUN_001d40f0`) or, when no view is
/// current, the HUD manager's (`*DAT_00333ed8`) [`FUN_001aec20`, `FUN_001adc68`]; `FUN_001d5700`
/// writes `camera + 0x20 = normalize(look point − eye)`, the forward axis from the eye into the
/// scene. So `a` is the half vector of the light and the eye direction (−f), and a normal along it
/// samples the map centre.
pub fn specular_basis(light1_dir: [f32; 3], view_forward: [f32; 3]) -> [[f32; 3]; 3] {
    let a = normalize([light1_dir[0] - view_forward[0], light1_dir[1] - view_forward[1], light1_dir[2] - view_forward[2]]);
    let b = normalize(cross([0.0, 1.0, 0.0], a));
    let c = normalize(cross(a, b));
    [b, c, a]
}

/// The third light column of the `World` and `Character` environments: `FUN_001b0158` 0x1b0158
/// computes all three directions per environment, then overwrites the dual-hemisphere one with
/// (0, 1, 0) for these two [M: `+0x88/+0x98/+0xa8` and `+0xf8/+0x108/+0x118` ← 0, 1.0, 0]. (Every
/// track's `DualHemiAngleX` is −90, which gives the same vector [D].)
pub const HEMI_AXIS: [f32; 3] = [0.0, 1.0, 0.0];

impl LightEnv {
    /// `Light 1`'s direction ([`light_direction`]).
    pub fn light1_dir(&self) -> [f32; 3] { light_direction(self.light1_angle_x, self.light1_angle_y) }
    /// The car specular basis for this environment's Light 1 and the view's forward vector
    /// ([`specular_basis`]).
    pub fn specular_basis(&self, view_forward: [f32; 3]) -> [[f32; 3]; 3] { specular_basis(self.light1_dir(), view_forward) }
    /// `Light 2`'s direction ([`light_direction`]).
    pub fn light2_dir(&self) -> [f32; 3] { light_direction(self.light2_angle_x, self.light2_angle_y) }

    /// The car microcode's base-pass light for a world-space unit normal `n`, before the ×127
    /// vertex scale and the 255 clamp (docs/formats/car-model.md §10) [M every `MatCar*` microcode
    /// of `data\render\bondrm.o`, the block after `IADDIU vi01, vi00, 0x49`]:
    /// `D·Light1·max(0, n·L1) + Light2·max(0, n·L2) + Ground·max(0, n·(0,1,0)) + A·Sky`, RGB.
    /// `ambient` / `diffuse` are the material's CarRender values ([`CarMaterialProps`]); only Light 1
    /// and the sky term are scaled by them. `shade` is the per-car factor `FUN_001af2d8` multiplies
    /// Ambient Sky, Ambient Ground and Light 1 by before the upload (`FUN_001adea8`: the car's
    /// `+0x1a` byte / 255 × `FUN_001af200`, which returns 1.0); Light 2 is not scaled.
    pub fn car_light(&self, n: [f32; 3], ambient: f32, diffuse: f32, shade: f32) -> [f32; 3] {
        let dot = |a: [f32; 3]| (n[0] * a[0] + n[1] * a[1] + n[2] * a[2]).max(0.0);
        let (d1, d2, dh) = (dot(self.light1_dir()), dot(self.light2_dir()), dot(HEMI_AXIS));
        let mut out = [0.0; 3];
        for (k, o) in out.iter_mut().enumerate() {
            *o = diffuse * shade * self.light1[k] * d1
                + self.light2[k] * d2
                + shade * self.ambient_ground[k] * dh
                + ambient * shade * self.ambient_sky[k];
        }
        out
    }
}

/// The `RReflection` per-vehicle material classes, in the order of the 0x80-byte per-vehicle block
/// (16 bytes each) and of the index `FUN_001c1778` takes [M `FUN_001c0930` registers
/// `[Glass]…` at block +0x00, `[SpecBump]…` +0x10, `[Paint]…` +0x20, `[Dull]…` +0x30, `[Chrome]…`
/// +0x40, `[Dash]…` +0x50; `FUN_001a8930` resolves `GAME::CarLighting{Glass,Specular,Glossy,Dull,
/// Dash,Chrome}Props` to `FUN_001c1778(k = 0, 1, 2, 3, 5, 4)`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum CarLightClass {
    Glass = 0,
    /// `[SpecBump]` = `GAME::CarLightingSpecularProps`.
    SpecBump = 1,
    /// `[Paint]` = `GAME::CarLightingGlossyProps`.
    Paint = 2,
    Dull = 3,
    Chrome = 4,
    Dash = 5,
}

impl CarLightClass {
    pub const ALL: [CarLightClass; 6] =
        [CarLightClass::Glass, CarLightClass::SpecBump, CarLightClass::Paint, CarLightClass::Dull, CarLightClass::Chrome, CarLightClass::Dash];

    /// The tuning section name (`Paint` for `[Paint]`).
    pub fn section(self) -> &'static str { ["Glass", "SpecBump", "Paint", "Dull", "Chrome", "Dash"][self as usize] }

    /// The class a car module's `__COORD4:::GAME::CarLighting<X>Props` reference names.
    pub fn from_props_symbol(sym: &str) -> Option<CarLightClass> {
        let x = sym.strip_prefix("__COORD4:::")?.strip_prefix("GAME::CarLighting")?.strip_suffix("Props")?;
        Some(match x {
            "Glass" => CarLightClass::Glass,
            "Specular" => CarLightClass::SpecBump,
            "Glossy" => CarLightClass::Paint,
            "Dull" => CarLightClass::Dull,
            "Chrome" => CarLightClass::Chrome,
            "Dash" => CarLightClass::Dash,
            _ => return None,
        })
    }
}

/// One class's COORD4: the quadword the material's render list uploads to VU1
/// `LightingProperties` as (x, y, z, w) = (ambient, diffuse, specular, reflect) [M `FUN_001c0930`:
/// Ambient +0, Diffuse +4, Specular +8, Reflect +0xc]. `[Dull]` and `[SpecBump]` register only
/// Diffuse/Ambient; their Specular/Reflect keep the constructor values.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CarMaterialProps {
    pub ambient: f32,
    pub diffuse: f32,
    pub specular: f32,
    pub reflect: f32,
}

/// The constructor's values for every class [M `FUN_001c0930`: 0.5, 0.5, 1.0, 0.8 per 16 bytes].
pub const CAR_PROPS_DEFAULT: CarMaterialProps = CarMaterialProps { ambient: 0.5, diffuse: 0.5, specular: 1.0, reflect: 0.8 };

/// One `{Instance}` of `Render\CarRender\default.tun` (a vehicle, `Characters`, `DynamicObjects`).
#[derive(Clone, Debug, PartialEq)]
pub struct CarRenderEntry {
    pub name: String,
    /// Block +0x60 (constructor 0.21): scales the model-space position added to the normal in the
    /// reflection pass (docs/formats/car-model.md §10).
    pub fresnel: f32,
    /// +0x64 (0.37): scales the reflection-map lookup vector.
    pub reflection_warp: f32,
    /// +0x68 (1.5): scales the specular-map lookup vector.
    pub specular_spot: f32,
    /// Indexed by [`CarLightClass`].
    pub classes: [CarMaterialProps; 6],
}

impl CarRenderEntry {
    pub fn props(&self, c: CarLightClass) -> CarMaterialProps { self.classes[c as usize] }
}

/// A full-map quad of the reflection map pass ([`CarRender::reflection_steps`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReflectionLayer {
    /// The texture's sheet entry name.
    pub texture: &'static str,
    pub rgba: [u8; 4],
    /// The GS `ALPHA_1` value (`carmesh::GsBlend::from_alpha`).
    pub alpha_reg: u64,
}

/// One step of the reflection map pass.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReflectionStep {
    Layer(ReflectionLayer),
    /// The scene drawn from the reflection camera ([`ReflectionView`]): the track items that rise
    /// over the car ([`ReflectionItemQuery`], `FUN_0021fa58`), then the kept vehicles
    /// ([`REFLECTION_VEHICLES`], `FUN_001c2720`). No sky is drawn here; the sky is the `cref` layers.
    Scene,
}

/// `Render\CarRender\default.tun` [M `FUN_001c0930` 0x1c0930, the `RReflection` private data]: the
/// globals and one entry per instance in file order (the instance list is the vehicle table +
/// `Characters` + `Dynamic Objects`; the shipped file names `default` twice [D]).
#[derive(Clone, Debug, PartialEq)]
pub struct CarRender {
    /// `Desat` (+0x04), packed like the fog colour ([`packed_colour`]): the vertex colour of the
    /// reflection map's desaturation quad ([`CarRender::reflection_steps`]).
    pub desat: u32,
    /// `Sky blend` (+0x02, 0..64): the alpha of the sky quad drawn over the reflection scene.
    pub sky_blend: u8,
    /// `Sky bright` (+0x00, 0..255): the grey level (× 0.94) of the reflection map's sky quads.
    pub sky_bright: u8,
    /// `Desat ?` (+0x08) and `Sky ?` (+0x0c).
    pub desat_on: bool,
    pub sky_on: bool,
    /// `Refl HEIGHT` (+0x10, −10..10): added to the reflection eye's y [M `FUN_001c1b18`].
    /// `Refl ANGLE` (+0x14, 5..180): the reflection view's projection angle
    /// [M `FUN_001c1560`: `FUN_00295910(+0x14, 1, 1, 2500, view)`, once, in the constructor].
    pub refl_height: f32,
    pub refl_angle: f32,
    pub entries: Vec<CarRenderEntry>,
}

impl CarRender {
    pub fn parse(text: &str) -> Result<CarRender> {
        let kv = parse(text);
        let byte = |k: &str| -> Result<u8> {
            let v = f1(&kv, k)?;
            if !(0.0..=255.0).contains(&v) {
                return Err(FormatError::Invalid(format!("{k}: {v} is not a byte")));
            }
            Ok(v as u8)
        };
        let names: Vec<String> =
            kv.iter().filter_map(|(k, _)| k.strip_prefix("Fresnel{").and_then(|r| r.strip_suffix('}')).map(str::to_owned)).collect();
        if names.is_empty() {
            return Err(FormatError::Invalid("CarRender: no Fresnel{…} entries".into()));
        }
        let mut entries = Vec::with_capacity(names.len());
        for (i, name) in names.iter().enumerate() {
            // A repeated instance name (`default`) repeats every key: the n-th entry of a name
            // takes the n-th occurrence, the order the writer emits them in.
            let dup = names[..i].iter().filter(|m| *m == name).count();
            let val = |key: String| -> Result<f32> {
                let v = kv
                    .iter()
                    .filter(|(k, _)| *k == key)
                    .nth(dup)
                    .map(|(_, v)| v.as_str())
                    .ok_or_else(|| FormatError::Invalid(format!("tuning key {key} missing")))?;
                floats(v)?.first().copied().ok_or_else(|| FormatError::Invalid(format!("{key}: empty")))
            };
            let mut classes = [CAR_PROPS_DEFAULT; 6];
            for c in CarLightClass::ALL {
                let sec = c.section();
                let p = &mut classes[c as usize];
                p.diffuse = val(format!("[{sec}]Diffuse{{{name}}}"))?;
                p.ambient = val(format!("[{sec}]Ambient{{{name}}}"))?;
                if !matches!(c, CarLightClass::Dull | CarLightClass::SpecBump) {
                    p.reflect = val(format!("[{sec}]Reflect{{{name}}}"))?;
                    p.specular = val(format!("[{sec}]Specular{{{name}}}"))?;
                }
            }
            entries.push(CarRenderEntry {
                name: name.clone(),
                fresnel: val(format!("Fresnel{{{name}}}"))?,
                reflection_warp: val(format!("ReflectionWarp{{{name}}}"))?,
                specular_spot: val(format!("SpecularSpot{{{name}}}"))?,
                classes,
            });
        }
        Ok(CarRender {
            desat: packed_colour(&kv, "Desat")?,
            sky_blend: byte("Skyblend")?,
            sky_bright: byte("Skybright")?,
            desat_on: f1(&kv, "Desat?")? != 0.0,
            sky_on: f1(&kv, "Sky?")? != 0.0,
            refl_height: f1(&kv, "ReflHEIGHT")?,
            refl_angle: f1(&kv, "ReflANGLE")?,
            entries,
        })
    }

    /// The first entry of that instance name (the game looks a vehicle up by name, `FUN_001965e8`;
    /// which of the two `default` rows that lands on is not traced).
    pub fn get(&self, name: &str) -> Option<&CarRenderEntry> { self.entries.iter().find(|e| e.name == name) }

    /// The grey level of the sky quads: `(int)(Sky bright × 0.94) & 0xff` [M `FUN_001c1ec0`].
    pub fn sky_level(&self) -> u8 { ((self.sky_bright as f32 * 0.94) as i32 & 0xff) as u8 }

    /// What `FUN_001c1ec0` 0x1c1ec0 draws into the 256 × 256 `GAME::ReflectionMap` each frame, in
    /// order, after clearing it [M]. Each layer is a full-map quad (`FUN_0019f960(0, 0, 256, 256,
    /// colour, texture)`) drawn with ABE on and the given GS `ALPHA` value; `rgba` is the vertex
    /// colour (R, G, B, A bytes):
    /// 1. if `Sky ?`: texture `cref` (RReflection `+0xc4`), colour (s, s, s, 0x40), ALPHA 0x44
    ///    ((Cs − Cd)·As + Cd), s = [`Self::sky_level`];
    /// 2. the scene from the reflection camera ([`ReflectionStep::Scene`]); skipped, together with
    ///    the layers after it, when a global object's `+0x30` is 1 (`DAT_003349e0`, not traced),
    ///    which draws a flat (0x80, 0x80, 0x80, 0x80) quad instead;
    /// 3. if `Sky ?` and `Sky blend` ≠ 0: `cref` again, colour (s, s, s, Sky blend), ALPHA 0x44;
    /// 4. if `Desat ?`: texture `rmsk` (RReflection `+0xcc`, `common.ssh` entry `rmsk`,
    ///    `FUN_001c0930`), colour = the packed `Desat` word as bytes, ALPHA 0x62 (Cd − Cs·FIX/128,
    ///    FIX = 0x80): a subtraction.
    ///
    /// `cref` comes from a sheet registered as key 4, else from `data\render\ext.ssh` (key 5)
    /// [M `FUN_001c1358`]. Key 4 holds the sheets the loaded modules name in their `sn  ` records
    /// (`FUN_001a9600`); the track's `sn #0` names its own sheet [D], and among every `.ssh` of the
    /// 8 archives only the track sheets of paris, uw, snow1a and snow2a and `ext.ssh` hold a `cref`
    /// (128 × 128) [D]. So the jungle tracks use `ext.ssh`'s.
    pub fn reflection_steps(&self) -> Vec<ReflectionStep> {
        let s = self.sky_level();
        let mut out = Vec::new();
        if self.sky_on {
            out.push(ReflectionStep::Layer(ReflectionLayer { texture: "cref", rgba: [s, s, s, 0x40], alpha_reg: 0x80_0000_0044 }));
        }
        out.push(ReflectionStep::Scene);
        if self.sky_on && self.sky_blend != 0 {
            out.push(ReflectionStep::Layer(ReflectionLayer { texture: "cref", rgba: [s, s, s, self.sky_blend], alpha_reg: 0x80_0000_0044 }));
        }
        if self.desat_on {
            out.push(ReflectionStep::Layer(ReflectionLayer { texture: "rmsk", rgba: self.desat.to_le_bytes(), alpha_reg: 0x80_0000_0062 }));
        }
        out
    }

    /// The reflection camera's eye and forward axis for a view forward `f` and a point `p`
    /// [M `FUN_001c1b18`]: eye = `p + (0, Refl HEIGHT, 0)`, forward = `normalize(0.8 · f + (0, 1,
    /// 0))` (0.8 = `DAT_00333e78` [D .data]). `p` is the player car's position ([`Self::reflection_view`]).
    pub fn reflection_camera(&self, p: [f32; 3], f: [f32; 3]) -> ([f32; 3], [f32; 3]) {
        let v = [0.8 * f[0], 0.8 * f[1] + 1.0, 0.8 * f[2]];
        ([p[0], p[1] + self.refl_height, p[2]], normalize(v))
    }

    /// Everything `FUN_001c1ec0` 0x1c1ec0 needs to draw this frame's `GAME::ReflectionMap`
    /// (docs/formats/car-model.md §10.6.3). There is one map per frame, shared by every car.
    ///
    /// - `car_pos`: the player car's position. `FUN_001c5eb0` passes `FUN_00201af0(0x339190)`:
    ///   the first car of the sim's `+0xdcc` list (the car `FUN_001fee68` places at the track
    ///   start), its body (`FUN_001ff088` / `FUN_001ff0b0`) `+ 0x10` [M].
    /// - `main_forward`: row 2 (`+0x20`) of the camera `**DAT_00333ed8` points at, read at the top
    ///   of `FUN_001c1ec0` [M].
    pub fn reflection_view(&self, car_pos: [f32; 3], main_forward: [f32; 3]) -> ReflectionView {
        let (eye, forward) = self.reflection_camera(car_pos, main_forward);
        // FUN_001c1b18: right = normalize((0, 0, -1) x forward), up = normalize(forward x right)
        // (VOPMULA/VOPMSUB at 0x1c1c14 and 0x1c1c50, fs x ft with fs = (0, 0, -1) then forward).
        let right = normalize(cross([0.0, 0.0, -1.0], forward));
        let up = normalize(cross(forward, right));
        ReflectionView {
            eye,
            right,
            up,
            forward,
            projection: ReflectionProjection { fov_deg: self.refl_angle, aspect: 1.0, near: 1.0, far: 2500.0 },
            items: ReflectionItemQuery { centre: car_pos },
            vehicles: REFLECTION_VEHICLES,
            fog: REFLECTION_FOG,
            steps: self.reflection_steps(),
        }
    }
}

/// The reflection map's size in texels: `FUN_0027ed78(0x100, 0x100, 0x10)` [M `FUN_001c0930`]; the
/// quads and both views cover (0, 0, 256, 256) [M `FUN_001c1ec0`, `FUN_001c1560`].
pub const REFLECTION_MAP_SIZE: u32 = 256;
/// How many vehicles the reflection scene draws: the 4 slots at `RReflection + 0x1f4`
/// [M `FUN_001c2720`, `FUN_001c2508`].
pub const REFLECTION_VEHICLES: usize = 4;
/// The fog range set for the scene draw: `DAT_0033376c + 4` gets (24560, 0, 0, 4080 / 24560) and is
/// restored after [M `FUN_001c1ec0` 0x1c2140]. As (start, end) this is (0, 24560): far beyond the
/// 2500 far plane, so the reflection carries almost no fog. Reading the four words as
/// (end, start, -, scale) is inferred from the `4080 / (w0 - w1)` they compute.
pub const REFLECTION_FOG: (f32, f32) = (0.0, 24560.0);

/// The reflection camera, its projection and what it draws ([`CarRender::reflection_view`]).
#[derive(Clone, Debug, PartialEq)]
pub struct ReflectionView {
    /// The camera position: the car position raised by `Refl HEIGHT`.
    pub eye: [f32; 3],
    /// The camera rows 0-2 that `FUN_001a4520` stores [M `FUN_001c1b18`]. The camera looks along
    /// `forward`. The view matrix is the inverse of the rows (right, up, -forward, eye)
    /// (`FUN_001a44f0` negates row 2, `FUN_001a4468` inverts, `FUN_00296e80` loads it), the same
    /// as for the main camera (`FUN_001d4878`). See [`ReflectionView::to_view`].
    pub right: [f32; 3],
    pub up: [f32; 3],
    pub forward: [f32; 3],
    pub projection: ReflectionProjection,
    /// The track items drawn ([`ReflectionItemQuery`]).
    pub items: ReflectionItemQuery,
    /// The vehicles drawn after the items ([`REFLECTION_VEHICLES`]).
    pub vehicles: usize,
    /// (start, end) of the fog during the scene draw ([`REFLECTION_FOG`]).
    pub fog: (f32, f32),
    /// The layers and the scene in draw order ([`CarRender::reflection_steps`]).
    pub steps: Vec<ReflectionStep>,
}

impl ReflectionView {
    /// A world point in view space: `((p - eye)·right, (p - eye)·up, -(p - eye)·forward)`. Points
    /// in front of the camera have z < 0.
    pub fn to_view(&self, p: [f32; 3]) -> [f32; 3] {
        let d = [p[0] - self.eye[0], p[1] - self.eye[1], p[2] - self.eye[2]];
        let dot = |a: [f32; 3]| d[0] * a[0] + d[1] * a[1] + d[2] * a[2];
        [dot(self.right), dot(self.up), -dot(self.forward)]
    }

    /// The map texel `(s, t)` a world point lands on, `None` when it is not in front of the camera
    /// ([`ReflectionProjection::map_texel`]).
    pub fn map_texel(&self, p: [f32; 3]) -> Option<[f32; 2]> { self.projection.map_texel(self.to_view(p)) }
}

/// The reflection view's perspective [M `FUN_001c1560`: `FUN_00295910(Refl ANGLE, 1, 1, 2500,
/// view)`, once, in the constructor; nothing on the per-frame path rewrites it]. The `.tun` value
/// is already in place then: `FUN_001426a8` reads each key from the loaded file when it is
/// registered, inside `FUN_001c0930`, which `FUN_001c1560` calls first [M]. The constructor default
/// is 120 [M `FUN_001c0930`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ReflectionProjection {
    /// Full angle in degrees, the same across and down (aspect 1, square map).
    pub fov_deg: f32,
    pub aspect: f32,
    pub near: f32,
    pub far: f32,
}

impl ReflectionProjection {
    /// The clip coordinates of a view-space point [M `FUN_00295910`]: with `k = 1 / tan(fov / 2)`,
    /// `(k·x, -k·aspect·y, a·z + b, -z)`, `a = -(far + near) / (far - near)`,
    /// `b = far·(a + 1) = -2·far·near / (far - near)`. The game also divides x and y by the view's
    /// `+0x190` scale and multiplies the viewport by it (`+0x17c`), so that factor cancels and is
    /// left out.
    pub fn clip(&self, v: [f32; 3]) -> [f32; 4] {
        let k = 1.0 / (self.fov_deg * 0.5).to_radians().tan();
        let (n, f) = (self.near, self.far);
        let a = -(f + n) / (f - n);
        [k * v[0], -k * self.aspect * v[1], a * v[2] + f * (a + 1.0), -v[2]]
    }

    /// The texel on the 256 × 256 map: `128 + 128 · clip / w` on both axes, so x goes right and y
    /// goes down (the y row of the matrix is negated). This assumes the viewport maps -1..1 onto
    /// 0..256 (`+0x16c` centre, `+0x17c` half size); the GS step itself is not followed.
    pub fn map_texel(&self, v: [f32; 3]) -> Option<[f32; 2]> {
        let c = self.clip(v);
        if c[3] <= 0.0 {
            return None;
        }
        let h = REFLECTION_MAP_SIZE as f32 * 0.5;
        Some([h + h * c[0] / c[3], h + h * c[1] / c[3]])
    }
}

/// The packed bounds word at `+0x3c` of a track item (the 0x40-byte entries of the track module's
/// `Map` → `in  ` record, `FUN_0021eae8`): `(radius, height)` [M `FUN_0021fa58` 0x21fbfc..0x21fd28,
/// disassembled]. Scale s = 16 when bit 30 is set, else 0.25 (`DAT_003999d8` [D]). radius =
/// `(w & 0x3ff) · s`; height = `((w >> 10) & 0x3ff) · s` when bit 31 is set, else `2 · radius`.
pub fn item_bounds(w: u32) -> (f32, f32) {
    let s = if w & 0x4000_0000 != 0 { 16.0 } else { 0.25 };
    let r = (w & 0x3ff) as f32 * s;
    let h = if w & 0x8000_0000 != 0 { ((w >> 10) & 0x3ff) as f32 * s } else { 2.0 * r };
    (r, h)
}

/// Which track items the reflection scene draws [M `FUN_0021fa58` 0x21fa58, called from
/// `FUN_001c1ec0` as `FUN_0021fa58(50.0, world = DAT_0033a5d4, list, car_pos, (0, 1, 0, 1))`].
///
/// - The world's cell tree (`*(DAT_0033aac8 + 0x1c)`, `FUN_0021f8d0`) gives the cells whose circle
///   (size · 1.414) meets the 50-unit sphere round `centre`. Each cell lists u16 item indices.
/// - An item of those cells is kept by [`Self::keeps`].
/// - The kept items are sorted by address (`FUN_002207c0` with `FUN_00220bc0` comparing the item
///   pointers), which is table order, and each is drawn with `FUN_001ad618`: skipped when
///   `+0x0c & 1` or when it has no `as` set (`+0x1c`); otherwise the item matrix (`FUN_001beee0`)
///   and the LOD pick `FUN_001ad2e0`, as for the car parts (car-model.md §11).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ReflectionItemQuery {
    /// The car position (not raised by `Refl HEIGHT`).
    pub centre: [f32; 3],
}

/// The query sphere's radius (`DAT_00333e7c` = 50.0 [D .data]).
pub const REFLECTION_ITEM_RADIUS: f32 = 50.0;
/// An item is drawn only when its top is more than this above the car: 5.0 + `DAT_0033a5f0` (5.0
/// [D .data]; no writer in DRIVING.ELF) [M 0x21fb84, 0x21fd00].
pub const REFLECTION_ITEM_RISE: f32 = 10.0;

impl ReflectionItemQuery {
    /// Whether an item at `pos` (`+0x30`) with bounds word `bounds` (`+0x3c`, [`item_bounds`]) and
    /// flags byte `flags` (`+0x0c`) is drawn, given that its cell was gathered. All of:
    /// 1. `pos.y + height > centre.y + 10` (the item rises over the car);
    /// 2. `|pos - centre| - 50 ≤ radius` and radius > 0 (`FUN_0021e9f0`; the direction (0, 1, 0)
    ///    has x = z = 0, so its plane test only needs radius > 0);
    /// 3. `|pos - centre| < radius + 25 + 0.75 · height`;
    /// 4. `flags & 0x0d == 0`.
    pub fn keeps(&self, pos: [f32; 3], bounds: u32, flags: u8) -> bool {
        let (r, h) = item_bounds(bounds);
        let c = self.centre;
        if !(pos[1] + h > c[1] + REFLECTION_ITEM_RISE) {
            return false;
        }
        let d = [pos[0] - c[0], pos[1] - c[1], pos[2] - c[2]];
        let dist = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
        if !(r > 0.0 && dist - REFLECTION_ITEM_RADIUS <= r) {
            return false;
        }
        dist < r + 25.0 + 0.75 * h && flags & 0x0d == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fog_colour_packs_like_the_tuner_and_lands_in_fogcol_order() {
        // The paris form (four small fields) and the snow form (a negative second field).
        let f = Fog::parse("fogSTART=1\r\nfogEND=1450.21\r\nfogmode=3\r\nfogDensity=0.000475\r\nFogColour=255,22,15,20\r\n").unwrap();
        assert_eq!((f.start, f.end, f.mode), (1.0, 1450.21, 3));
        assert_eq!(f.colour, [22, 15, 20]);
        // A plain left-to-right RGB reading of the last three fields would give [15, 20, ?]: wrong.
        assert_ne!(f.colour, [15, 20, 255]);
        let s = Fog::parse("fogSTART=1.0\nfogEND=250.949905\nfogmode=3\nfogDensity=1.0\nFogColour=255,-13488856,45,50\n").unwrap();
        // -13488856 = 0xff322c28; OR 50 << 16 | 45 << 8 → 0xff322d28.
        assert_eq!(s.colour, [0x28, 0x2d, 0x32]);
        assert!((s.slope() - 255.0 / 249.949905).abs() < 1e-4);
        assert!(Fog::parse("fogSTART=1\n").is_err());
    }

    /// A CarRender file in the shipped shape: two instances, `default` repeated with different
    /// values, `[Dull]` / `[SpecBump]` without Reflect/Specular.
    fn car_render_text() -> String {
        let mut t = String::from("Desat=128,-2137943663,145,145\r\nSkyblend=24\r\nSkybright=127\r\nDesat?=0\r\nSky?=1\r\nReflHEIGHT=0.000000\r\nReflANGLE=120.000000\r\n");
        let names = ["default", "vanquish", "default"];
        let per = |key: &str, vals: [f32; 3], t: &mut String| {
            for (n, v) in names.iter().zip(vals) {
                t.push_str(&format!("{key}{{{n}}}={v:.6}\r\n"));
            }
        };
        per("Fresnel", [0.21, 0.22, 0.23], &mut t);
        per("ReflectionWarp", [0.37, 0.38, 0.39], &mut t);
        per("SpecularSpot", [1.5, 0.9, 1.4], &mut t);
        for (i, sec) in ["Paint", "Dull", "SpecBump", "Glass", "Chrome", "Dash"].iter().enumerate() {
            let b = i as f32;
            per(&format!("[{sec}]Diffuse"), [0.5, 1.0 + b, 0.5], &mut t);
            per(&format!("[{sec}]Ambient"), [0.5, 2.0 + b, 0.5], &mut t);
            if !matches!(*sec, "Dull" | "SpecBump") {
                per(&format!("[{sec}]Reflect"), [0.8, 3.0 + b, 0.8], &mut t);
                per(&format!("[{sec}]Specular"), [1.0, 4.0 + b, 1.0], &mut t);
            }
        }
        t
    }

    #[test]
    fn car_render_reads_every_class_into_its_block_slot_and_keeps_repeated_names_apart() {
        let c = CarRender::parse(&car_render_text()).unwrap();
        assert_eq!(c.entries.iter().map(|e| e.name.as_str()).collect::<Vec<_>>(), ["default", "vanquish", "default"]);
        assert_eq!((c.sky_blend, c.sky_bright, c.desat_on, c.sky_on, c.refl_angle), (24, 127, false, true, 120.0));
        // 0x80919191: the second field is ORed in unmasked, as for FogColour.
        assert_eq!(c.desat, 0x8091_9191);
        let v = c.get("vanquish").unwrap();
        assert_eq!((v.fresnel, v.reflection_warp, v.specular_spot), (0.22, 0.38, 0.9));
        // File section order (Paint, Dull, SpecBump, Glass, Chrome, Dash) is not the block order.
        let p = v.props(CarLightClass::Paint);
        assert_eq!((p.diffuse, p.ambient, p.reflect, p.specular), (1.0, 2.0, 3.0, 4.0));
        assert_eq!(v.props(CarLightClass::Glass), CarMaterialProps { diffuse: 4.0, ambient: 5.0, reflect: 6.0, specular: 7.0 });
        assert_eq!(v.props(CarLightClass::Dash).diffuse, 6.0);
        // Dull keeps the constructor's specular/reflect.
        let d = v.props(CarLightClass::Dull);
        assert_eq!((d.diffuse, d.ambient, d.specular, d.reflect), (2.0, 3.0, 1.0, 0.8));
        // The second `default` row is its own entry, not a copy of the first.
        assert_eq!((c.entries[0].fresnel, c.entries[2].fresnel), (0.21, 0.23));
        assert_eq!(c.entries[2].specular_spot, 1.4);
        // A missing class value is an error, not a silent default.
        let broken = car_render_text().replace("[Chrome]Ambient{vanquish}", "[Chrome]Ambiant{vanquish}");
        assert!(CarRender::parse(&broken).is_err());
        assert_eq!(CarLightClass::from_props_symbol("__COORD4:::GAME::CarLightingGlossyProps"), Some(CarLightClass::Paint));
        assert_eq!(CarLightClass::from_props_symbol("__COORD4:::GAME::CarLightingSpecularProps"), Some(CarLightClass::SpecBump));
        assert_eq!(CarLightClass::from_props_symbol("__COORD4:::GAME::SwitchData[1]"), None);
    }

    fn close(a: [f32; 3], b: [f32; 3]) -> bool { a.iter().zip(b).all(|(x, y)| (x - y).abs() < 1e-5) }

    #[test]
    fn light_angles_turn_into_the_vector_the_microcode_dots_with_the_normal() {
        // X = −90: straight up; X = 0, Y = 90: +x; X = 0, Y = 0: +z.
        assert!(close(light_direction(-90.0, 37.0), [0.0, 1.0, 0.0]));
        assert!(close(light_direction(0.0, 90.0), [1.0, 0.0, 0.0]));
        assert!(close(light_direction(0.0, 0.0), [0.0, 0.0, 1.0]));
        // Paris World Light 1 (X −51.3, Y 208.8): up and towards −z, −x.
        let d = light_direction(-51.2999, 208.8);
        assert!(d[1] > 0.78 && d[0] < 0.0 && d[2] < 0.0, "{d:?}");
        assert!((d.iter().map(|c| c * c).sum::<f32>() - 1.0).abs() < 1e-5);
        // A Y-then-X order or a sign flip on y would put it below the horizon or elsewhere.
        assert!(!close(d, [-d[0], -d[1], d[2]]));

        let env = LightEnv {
            ambient_sky: [0.72, 0.72, 0.72, 0.0],
            ambient_ground: [0.1, 0.2, 0.3, 0.0],
            light1: [0.9, 0.94, 1.0, 0.0],
            light1_angle_y: 0.0,
            light1_angle_x: -90.0,
            light2: [0.5, 0.5, 0.5, 0.0],
            light2_angle_y: 90.0,
            light2_angle_x: 0.0,
            dual_hemi_angle_y: 0.0,
            dual_hemi_angle_x: -90.0,
        };
        // An upward normal: Light 1 and the hemisphere term at full, Light 2 (along +x) at 0.
        let up = env.car_light([0.0, 1.0, 0.0], 0.79, 0.42, 1.0);
        assert!(close(up, [0.42 * 0.9 + 0.1 + 0.79 * 0.72, 0.42 * 0.94 + 0.2 + 0.79 * 0.72, 0.42 * 1.0 + 0.3 + 0.79 * 0.72]), "{up:?}");
        // A +x normal: only Light 2 (unscaled by the material) and the sky.
        let side = env.car_light([1.0, 0.0, 0.0], 0.79, 0.42, 0.5);
        assert!(close(side, [0.5 + 0.79 * 0.5 * 0.72; 3]), "{side:?}");
        // A downward normal gets the sky term alone.
        assert!(close(env.car_light([0.0, -1.0, 0.0], 1.0, 1.0, 1.0), [0.72; 3]));
    }

    #[test]
    fn the_specular_basis_is_the_half_vector_frame_with_the_traced_cross_product_order() {
        // Light from straight above, view along −z (eye looking forward).
        let [b, c, a] = specular_basis([0.0, 1.0, 0.0], [0.0, 0.0, -1.0]);
        let h = std::f32::consts::FRAC_1_SQRT_2;
        assert!(close(a, [0.0, h, h]), "{a:?}");
        // (0,1,0) × a, not a × (0,1,0): +x, not −x.
        assert!(close(b, [1.0, 0.0, 0.0]), "{b:?}");
        // a × b, not b × a: (0, .707, −.707).
        assert!(close(c, [0.0, h, -h]), "{c:?}");
        // A light from the side moves the half vector with it.
        let [_, _, a2] = specular_basis([1.0, 0.0, 0.0], [0.0, 0.0, -1.0]);
        assert!(close(a2, [h, 0.0, h]), "{a2:?}");
    }

    #[test]
    fn the_reflection_map_pass_uses_sky_bright_blend_and_desat_in_the_traced_order() {
        let c = CarRender::parse(&car_render_text()).unwrap();
        // Skybright 127 × 0.94 = 119.38 → 119.
        assert_eq!(c.sky_level(), 119);
        // Sky? on, Sky blend 24, Desat? off: sky quad, scene, sky quad at alpha 24.
        let steps = c.reflection_steps();
        let sky = |a: u8| ReflectionStep::Layer(ReflectionLayer { texture: "cref", rgba: [119, 119, 119, a], alpha_reg: 0x80_0000_0044 });
        assert_eq!(steps, vec![sky(0x40), ReflectionStep::Scene, sky(24)]);
        // Desat on: the packed word's bytes, subtracted (ALPHA 0x62).
        let d = CarRender { desat_on: true, sky_on: false, ..c.clone() };
        assert_eq!(
            d.reflection_steps(),
            vec![ReflectionStep::Scene, ReflectionStep::Layer(ReflectionLayer { texture: "rmsk", rgba: [0x91, 0x91, 0x91, 0x80], alpha_reg: 0x80_0000_0062 })]
        );
        // Sky blend 0 drops the second sky quad only.
        let z = CarRender { sky_blend: 0, ..c.clone() };
        assert_eq!(z.reflection_steps(), vec![sky(0x40), ReflectionStep::Scene]);
        // The reflection camera: raised by Refl HEIGHT, looking up between the view axis and +y.
        let r = CarRender { refl_height: 2.0, ..c };
        let (eye, fwd) = r.reflection_camera([1.0, 0.5, -3.0], [0.0, 0.0, -1.0]);
        assert_eq!(eye, [1.0, 2.5, -3.0]);
        let l = (0.64f32 + 1.0).sqrt();
        assert!(close(fwd, [0.0, 1.0 / l, -0.8 / l]), "{fwd:?}");
    }

    #[test]
    fn the_reflection_view_looks_up_past_the_car_with_the_traced_axes() {
        let c = CarRender::parse(&car_render_text()).unwrap();
        let v = c.reflection_view([10.0, 2.0, -5.0], [0.0, 0.0, -1.0]);
        let l = (0.64f32 + 1.0).sqrt();
        // forward = normalize(0.8·f + up); right = normalize((0, 0, −1) × forward) = +x here;
        // up = forward × right. The opposite cross order would flip right and up.
        assert!(close(v.forward, [0.0, 1.0 / l, -0.8 / l]), "{:?}", v.forward);
        assert!(close(v.right, [1.0, 0.0, 0.0]), "{:?}", v.right);
        assert!(close(v.up, [0.0, -0.8 / l, -1.0 / l]), "{:?}", v.up);
        assert_eq!(v.eye, [10.0, 2.0, -5.0]);
        assert_eq!(v.projection, ReflectionProjection { fov_deg: 120.0, aspect: 1.0, near: 1.0, far: 2500.0 });
        assert_eq!(v.items.centre, [10.0, 2.0, -5.0]);
        assert_eq!((v.vehicles, v.fog), (4, (0.0, 24560.0)));
        assert_eq!(v.steps, c.reflection_steps());
        // Refl HEIGHT raises the eye only; the item query stays at the car.
        let r = CarRender { refl_height: 3.0, ..c.clone() }.reflection_view([10.0, 2.0, -5.0], [0.0, 0.0, -1.0]);
        assert_eq!((r.eye, r.items.centre), ([10.0, 5.0, -5.0], [10.0, 2.0, -5.0]));
        // Any view direction gives an orthonormal basis; right has z = 0 ((0, 0, −1) × F).
        let w = c.reflection_view([0.0; 3], normalize([0.6, -0.2, 0.7]));
        let dot = |a: [f32; 3], b: [f32; 3]| a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
        assert!(dot(w.right, w.up).abs() < 1e-5 && dot(w.right, w.forward).abs() < 1e-5 && dot(w.up, w.forward).abs() < 1e-5);
        assert!(w.right[2].abs() < 1e-6, "{:?}", w.right);
        // right × up = +forward: the map is a mirror image (a plain camera has right = forward × up).
        assert!(close(cross(w.right, w.up), w.forward), "right × up = forward");
    }

    #[test]
    fn the_reflection_projection_maps_the_view_cone_onto_the_256_map() {
        let c = CarRender::parse(&car_render_text()).unwrap();
        let v = c.reflection_view([0.0; 3], [0.0, 0.0, -1.0]);
        // A point straight along forward is at view (0, 0, −d) and lands on the centre texel.
        let ahead = v.forward.map(|x| x * 40.0);
        let p = v.to_view(ahead);
        assert!(close(p, [0.0, 0.0, -40.0]), "{p:?}");
        let t = v.map_texel(ahead).unwrap();
        assert!((t[0] - 128.0).abs() < 1e-3 && (t[1] - 128.0).abs() < 1e-3, "{t:?}");
        // 120° full angle: a point 60° off axis to the right sits on the right edge (s = 256);
        // the same angle along up sits on the top edge (t = 0): t grows downwards.
        let k = 60f32.to_radians().tan();
        let edge = v.projection.map_texel([k * 10.0, 0.0, -10.0]).unwrap();
        assert!((edge[0] - 256.0).abs() < 1e-3 && (edge[1] - 128.0).abs() < 1e-3, "{edge:?}");
        let top = v.projection.map_texel([0.0, k * 10.0, -10.0]).unwrap();
        assert!((top[1] - 0.0).abs() < 1e-3, "{top:?}");
        // Behind the camera: no texel.
        assert_eq!(v.projection.map_texel([0.0, 0.0, 5.0]), None);
        // Depth: near → −1, far → +1 after the divide (the game forms b as far·(a + 1), off by ~1e-4 in f32).
        let n = v.projection.clip([0.0, 0.0, -1.0]);
        let f = v.projection.clip([0.0, 0.0, -2500.0]);
        assert!((n[2] / n[3] + 1.0).abs() < 1e-3 && (f[2] / f[3] - 1.0).abs() < 1e-3, "{n:?} {f:?}");
        // A narrower Refl ANGLE magnifies: the 60° point leaves the map.
        let narrow = ReflectionProjection { fov_deg: 90.0, ..v.projection };
        assert!(narrow.map_texel([k * 10.0, 0.0, -10.0]).unwrap()[0] > 256.0);
    }

    #[test]
    fn item_bounds_unpack_the_scaled_radius_and_height() {
        // Bits 0-9 radius; bit 30 scale 16 (else 0.25); bit 31 = an own height in bits 10-19.
        assert_eq!(item_bounds(40), (10.0, 20.0));
        assert_eq!(item_bounds(0x4000_0003), (48.0, 96.0));
        assert_eq!(item_bounds(0x8000_0000 | (12 << 10) | 8), (2.0, 3.0));
        assert_eq!(item_bounds(0xc000_0000 | (5 << 10) | 1), (16.0, 80.0));
        // Bits above 9 are not part of the radius when bit 31 is clear.
        assert_eq!(item_bounds((7 << 10) | 4), (1.0, 2.0));
    }

    #[test]
    fn the_reflection_scene_keeps_only_items_that_rise_over_the_car_nearby() {
        let q = ReflectionItemQuery { centre: [0.0, 0.0, 0.0] };
        // r 10, h 20 at 30 units: top 20 > 10, 30 − 50 ≤ 10, 30 < 10 + 25 + 15.
        assert!(q.keeps([30.0, 0.0, 0.0], 40, 0));
        // Flag bit 1 is not in the 0x0d mask; bits 0, 2, 3 are.
        assert!(q.keeps([30.0, 0.0, 0.0], 40, 0x02));
        for f in [0x01, 0x04, 0x08] {
            assert!(!q.keeps([30.0, 0.0, 0.0], 40, f), "flag {f:#x}");
        }
        // Too low: r 10 but h 8, top 8 ≤ 10.
        let low = 0x8000_0000 | (32 << 10) | 40;
        assert!(!q.keeps([5.0, 0.0, 0.0], low, 0));
        // The same item 3 units higher rises over the car.
        assert!(q.keeps([5.0, 3.0, 0.0], low, 0));
        // 55 away: inside 50 + r but not inside r + 25 + 0.75·h = 50.
        assert!(!q.keeps([55.0, 0.0, 0.0], 40, 0));
        // A tall thin item (r 1, h 250) at 60: inside the height reach, outside 50 + r.
        let tall = 0x8000_0000 | (1000 << 10) | 4;
        assert!(!q.keeps([60.0, 0.0, 0.0], tall, 0));
        assert!(q.keeps([50.5, 0.0, 0.0], tall, 0));
        // Radius 0 never passes the sphere test.
        assert!(!q.keeps([10.0, 0.0, 0.0], 0x8000_0000 | (1000 << 10), 0));
        // The rise is measured from the car's own height.
        let up = ReflectionItemQuery { centre: [0.0, 15.0, 0.0] };
        assert!(!up.keeps([30.0, 0.0, 0.0], 40, 0));
    }
}
