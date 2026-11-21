//! The driving tracks' render tuning text files (`data\tuning\Render\Fog\<track>.tun`,
//! `data\tuning\Render\Lighting\<track>.tun`, RefPack-packed in each `/DRIVING/*.VIV`): plain
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

impl Fog {
    pub fn parse(text: &str) -> Result<Fog> {
        let kv = parse(text);
        let c: Vec<i64> = get(&kv, "FogColour")?
            .split(',')
            .map(|x| x.trim().parse::<i64>().map_err(|_| FormatError::Invalid(format!("FogColour {x:?}"))))
            .collect::<Result<_>>()?;
        if c.len() != 4 {
            return Err(FormatError::Invalid(format!("FogColour: {} components, want 4", c.len())));
        }
        let packed = ((c[0] as u32) << 24) | ((c[3] as u32) << 16) | ((c[2] as u32) << 8) | (c[1] as u32);
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
}
