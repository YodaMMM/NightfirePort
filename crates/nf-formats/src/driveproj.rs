//! The driving engine's projectile graphics data: the projectile and explosion light tuning
//! (`Render\Lighting\<track>.tun`), the smoke-ribbon trail constants, the bullet-streak
//! textures, and the car items that serve as projectile models (with their launch effects).
//! docs/systems/driving-projectile-gfx.md is the write-up; the logic is
//! `nf_game::driving::projectile_gfx`.
//!
//! Tags: [M] DRIVING.ELF decompile/disassembly (work/decomp/DRIVING.ELF/), [D] disc bytes,
//! "ELF data" = initialised words of DRIVING.ELF. Nothing here is a live capture.
//!
//! - Light tuning: `FUN_001ae630` (the `Render:Lighting` tuner) registers an enum `Explosion
//!   Type` over the names at 0x333848 (Tiny, Small, Medium, Large, Huge) with `Colour` / `Size`
//!   / `Decay` per entry (0x333878, stride 0x18: colour +0, size +0x10, decay +0x14), and an enum
//!   `Missile Type` over 0x333860 (Missile, Rocket, SnowmobileRocket, EMPInflight, EMPTarget)
//!   with `Colour` / `Size` (0x3338f0, stride 0x18) [M + ELF data]. The tun keys are
//!   `Colour{Tiny}` … `Size{EMPTarget}` [D].
//! - The projectile's model is a copy of one `in#0` item of the shooter's own car model
//!   (`FUN_0014ca60` → `FUN_001c7b60(missile render object, shooter render object, sub-item,
//!   identity 0x355bb0, 0xfb, 0x40)`): the copied item gets the identity local matrix, its
//!   hidden bit cleared [M]. The weapon is found from that sub-item's action byte: `FUN_001c9328`
//!   reads `as + 0x1a` (`+ 0x1b` for a mirrored item, flag 0x80) and `FUN_0020e678` returns the
//!   weapon whose barrel list holds it [M]. When the fire event carries a type-6 or type-2 `ef`
//!   entry, the model sub-item is that entry's `+0x15` [M `FUN_0014ca60`].

use std::collections::HashMap;

use crate::buf::{invalid, FormatError, Result};
use crate::carmesh::{decode_item, decode_set, TAG_AS, TAG_IN, TAG_SR};
use crate::crp::{tag, Crp};
use crate::trackfx::{kind as efkind, parse_ef_list, part_ef_index, EfEntry};
use crate::trackobj::TAG_EF;

/// `Missile Type` names, the index the light pass uses [M 0x333860 + `FUN_001ae630`].
pub const GLOW_NAMES: [&str; 5] = ["Missile", "Rocket", "SnowmobileRocket", "EMPInflight", "EMPTarget"];
/// `Explosion Type` names [M 0x333848].
pub const BLAST_NAMES: [&str; 5] = ["Tiny", "Small", "Medium", "Large", "Huge"];

/// One `Missile Type` entry: the projectile's point light.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GlowTune {
    pub colour: [f32; 4],
    pub size: f32,
}

/// One `Explosion Type` entry: the explosion's point light.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BlastTune {
    pub colour: [f32; 4],
    pub size: f32,
    pub decay: f32,
}

/// The projectile and explosion lights of one track's lighting tuning.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProjectileLighting {
    pub glow: [GlowTune; 5],
    pub blast: [BlastTune; 5],
}

const C1: [f32; 4] = [0.04, 0.03, 0.02, 1.0];

/// The `.data` defaults the tuner starts from [ELF data 0x333878 / 0x3338f0]. The disc's files
/// override them; the explosion Decay{Tiny} default is 45, every file says 35 [D].
pub const ELF_DEFAULTS: ProjectileLighting = ProjectileLighting {
    glow: [
        GlowTune { colour: [0.0, 0.0, 0.1, 1.0], size: 5.0 },
        GlowTune { colour: [0.1, 0.1, 0.05, 1.0], size: 5.0 },
        GlowTune { colour: [0.0, 0.0, 0.1, 1.0], size: 5.0 },
        GlowTune { colour: [0.1, 0.1, 0.05, 1.0], size: 5.0 },
        GlowTune { colour: [0.0, 0.0, 0.1, 1.0], size: 5.0 },
    ],
    blast: [
        BlastTune { colour: C1, size: 1.0, decay: 45.0 },
        BlastTune { colour: C1, size: 0.8, decay: 90.0 },
        BlastTune { colour: C1, size: 0.4, decay: 120.0 },
        BlastTune { colour: C1, size: 0.2, decay: 180.0 },
        BlastTune { colour: C1, size: 0.1, decay: 240.0 },
    ],
};

fn nums(v: &str) -> Vec<f32> { v.split(',').filter_map(|x| x.trim().parse::<f32>().ok()).collect() }

impl ProjectileLighting {
    /// Reads `Colour{..}` / `Size{..}` / `Decay{..}` for both enums over the ELF defaults (a key
    /// the file lacks keeps its default).
    pub fn parse(text: &str) -> ProjectileLighting {
        let mut t = ELF_DEFAULTS;
        for (k, v) in crate::rendertun::parse(text) {
            let Some((name, inst)) = k.split_once('{') else { continue };
            let inst = inst.trim_end_matches('}');
            let n = nums(&v);
            if let Some(i) = GLOW_NAMES.iter().position(|&g| g == inst) {
                match (name, n.as_slice()) {
                    ("Colour", &[a, b, c, d]) => t.glow[i].colour = [a, b, c, d],
                    ("Size", &[s]) => t.glow[i].size = s,
                    _ => {}
                }
            } else if let Some(i) = BLAST_NAMES.iter().position(|&g| g == inst) {
                match (name, n.as_slice()) {
                    ("Colour", &[a, b, c, d]) => t.blast[i].colour = [a, b, c, d],
                    ("Size", &[s]) => t.blast[i].size = s,
                    ("Decay", &[s]) => t.blast[i].decay = s,
                    _ => {}
                }
            }
        }
        t
    }
}

// ---------------------------------------------------------------- the smoke ribbon

/// Ribbon half-width per trail type, DAT_00334e20 [ELF data; read by `FUN_001e4640`].
pub const TRAIL_WIDTH: [f32; 3] = [0.3575, 0.1, 0.1];
/// Ribbon centre jitter per trail type, DAT_00334e30 [ELF data].
pub const TRAIL_JITTER: [f32; 3] = [0.2, 0.0, 0.0];
/// Vertex capacity of one ribbon (two per sample) [M `FUN_001e4640`: `*n < 0x60`]; the record
/// is 0x650 bytes [M `FUN_001e4230`].
pub const TRAIL_MAX_VERTS: usize = 0x60;
/// Vertices one draw covers at most [M `FUN_001e4258`: start = end - 0x28].
pub const TRAIL_DRAW_VERTS: usize = 0x28;
/// The ribbon texture, `common.ssh` entry `trai` (64x64) for all three types [M
/// `FUN_001e3e28`: the key "iart" read as a little-endian word; D every archive's common.ssh].
pub const TRAIL_TEXTURE: &str = "trai";

/// The ribbon's per-vertex colour, 0x3de100[i]: RGB 0x80, alpha `(i/12)² · 30` for i < 13,
/// else 30, vertex 0 at the oldest drawn sample [M `FUN_001e3e28`].
pub fn trail_vertex_colour(i: usize) -> [u8; 4] {
    let a = if i < 13 { let f = i as f32 / 12.0; (f * 30.0 * f) as i32 } else { 30 };
    [0x80, 0x80, 0x80, a as u8]
}

// ---------------------------------------------------------------- bullet streaks

/// `BulletStreakTexture` 0..3 → the `common.ssh` entry [M `FUN_001d7620`: the four keys at
/// 0x3349a0; D every archive's common.ssh holds them, 32x16].
pub const STREAK_TEXTURES: [&str; 4] = ["wtrc", "ytrc", "otrc", "btrc"];
/// Streak vertices per texture bucket and frame: a new quad only while the count is < 0x46
/// [M `FUN_001d7d78`].
pub const STREAK_BUCKET_LIMIT: usize = 0x46;
/// Live streaks at most [M `FUN_0019a9b8`: count < 0x40].
pub const STREAK_LIST_LIMIT: usize = 0x40;
/// The six (u, v) of a streak quad, by vertex index % 6 [M `FUN_001d7620`'s 0x426490 fill].
pub const STREAK_UV: [[f32; 2]; 6] = [[0.0, 1.0], [0.0, 0.0], [1.0, 0.1], [1.0, 0.1], [1.0, 1.0], [0.0, 1.0]];
/// Streak GS ALPHA: Cs·As + Cd, additive [M `FUN_001d7620` 0x3342d0 = 0x8000000048].
pub const STREAK_ALPHA_REG: u64 = 0x80_0000_0048;

// ---------------------------------------------------------------- explosions

/// The Giotto effects `FUN_0015a098` (the explosion event) can start [M].
pub const EXPLOSION_EFFECTS: [&str; 15] = [
    "FODHitEffect", "snowspecificexplosion", "rocketsplash", "Un_ExplL", "Un_ExplM", "Un_ExplS",
    "Ex_SmallD", "Ex_SmallD2", "Ex_SmallD3", "Ex_SmB00", "Ex_SmB01", "Ex_SmB02", "Ex_Small", "Ex_Small2", "Ex_Small3",
];

// ---------------------------------------------------------------- car items

/// One launch effect of a weapon item: an entry of the item's `as` effect list.
#[derive(Clone, Debug, PartialEq)]
pub struct LaunchFx {
    pub entry: EfEntry,
    /// Type 6: the `sr` string (`GIOT::Wp_Rock`, `GIOT::Wp_EMP`, …).
    pub name: Option<String>,
}

/// One `in#0` item of a car model and the vehicle action its `as` set names.
#[derive(Clone, Debug, PartialEq)]
pub struct ActionItem {
    pub index: usize,
    /// Item `+0x0c` (0x80 mirrored, 0x01 hidden, 0x40 not decoded).
    pub flags: u8,
    pub set: u16,
    /// `as + 0x1a`, or `+ 0x1b` for a mirrored item [M `FUN_001c9328`].
    pub action: u8,
    /// The item origin in the car frame.
    pub origin: [f32; 3],
    /// LOD `(distance, EAGL model name)`.
    pub lods: Vec<(f32, Option<String>)>,
    pub effects: Vec<LaunchFx>,
}

impl ActionItem {
    /// The item an event from this item's effect entry `e` copies as the model: the entry's
    /// `+0x15` for a type-6 or type-2 entry, else the item itself [M `FUN_0014ca60`].
    pub fn model_item(&self, e: &EfEntry) -> usize {
        if e.kind == efkind::GIOTTO || e.kind == efkind::GLARE { e.part as usize } else { self.index }
    }
}

fn cstr(p: &[u8]) -> String {
    let n = p.iter().position(|&b| b == 0).unwrap_or(p.len());
    String::from_utf8_lossy(&p[..n]).into_owned()
}

/// Every `in#0` item of a car `.crp` with a nonzero action byte, in item order.
pub fn action_items(crp_bytes: &[u8]) -> Result<Vec<ActionItem>> {
    let crp = Crp::parse(crp_bytes)?;
    let arti = crp.pool()?.into_iter().find(|r| r.tag == tag::ARTI).ok_or_else(|| FormatError::Invalid("no Arti in the car .crp pool".into()))?;
    let (s, e) = arti.window(crp.data)?;
    let mut sr: HashMap<u16, String> = HashMap::new();
    let mut sets: HashMap<u16, &[u8]> = HashMap::new();
    let mut ef: HashMap<u16, Vec<EfEntry>> = HashMap::new();
    let mut items = None;
    for r in crp.window_records(s, e)? {
        let code = r.tag_code();
        let inst = r.instance().unwrap_or(0);
        if code == TAG_SR {
            sr.insert(inst, cstr(crp.payload(&r)?));
        } else if code == TAG_AS {
            sets.insert(inst, crp.payload(&r)?);
        } else if code == TAG_EF {
            ef.insert(inst, parse_ef_list(crp.payload(&r)?));
        } else if code == TAG_IN && r.instance() == Some(0) && items.is_none() {
            items = Some(r);
        }
    }
    let items = items.ok_or_else(|| FormatError::Invalid("no in#0 record in the car article".into()))?;
    let p = crp.payload(&items)?;
    let n = items.count.max(0) as usize;
    if p.len() < n * 0x40 {
        return invalid(format!("in#0: payload {} < count {n} × 0x40", p.len()));
    }
    let mut out = Vec::new();
    for i in 0..n {
        let (_, origin, flags, _, _, set) = decode_item(&p[i * 0x40..(i + 1) * 0x40])?;
        let a = sets.get(&set).ok_or_else(|| FormatError::Invalid(format!("in item {i}: no as#{set}")))?;
        if a.len() < 0x20 {
            return invalid(format!("as#{set}: payload {} < 0x20", a.len()));
        }
        let action = a[0x1a + (flags >> 7) as usize];
        if action == 0 {
            continue;
        }
        let (_, lods) = decode_set(a)?;
        let lods = lods.into_iter().map(|(d, k)| (d, k.and_then(|k| sr.get(&k).cloned()))).collect();
        let effects = part_ef_index(a)
            .and_then(|k| ef.get(&k))
            .map(|l| {
                l.iter()
                    .map(|e| LaunchFx { entry: *e, name: (e.kind == efkind::GIOTTO && e.word18 >> 16 == 0x7372).then(|| sr.get(&(e.word18 as u16)).cloned()).flatten() })
                    .collect()
            })
            .unwrap_or_default();
        out.push(ActionItem { index: i, flags, set, action, origin, lods, effects });
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lighting_parse_reads_both_enums_over_the_defaults() {
        let t = ProjectileLighting::parse("Colour{Missile}=0.400000,0.400000,0.260000,1.000000\nSize{Missile}=1.920600\nDecay{Tiny}=35\nSize{Huge}=0.1\n");
        assert_eq!(t.glow[0], GlowTune { colour: [0.4, 0.4, 0.26, 1.0], size: 1.9206 });
        assert_eq!(t.blast[0].decay, 35.0);
        // untouched keys keep the .data defaults
        assert_eq!(t.glow[1], ELF_DEFAULTS.glow[1]);
        assert_eq!(t.blast[4], BlastTune { colour: C1, size: 0.1, decay: 240.0 });
    }

    #[test]
    fn trail_alpha_ramps_quadratically_to_30() {
        let a: Vec<u8> = (0..14).map(|i| trail_vertex_colour(i)[3]).collect();
        // (i/12)^2*30 truncated: 0 0 0 1 3 5 7 10 13 16 20 25 30 30
        assert_eq!(a, vec![0, 0, 0, 1, 3, 5, 7, 10, 13, 16, 20, 25, 30, 30]);
        assert_eq!(trail_vertex_colour(39), [0x80, 0x80, 0x80, 30]);
    }

    #[test]
    fn model_item_follows_the_entry_part_for_giotto_and_glare_only() {
        let it = ActionItem { index: 40, flags: 0x80, set: 0x23, action: 6, origin: [0.0; 3], lods: vec![], effects: vec![] };
        let mut e = EfEntry::parse(&[0u8; 0x40]);
        e.part = 39;
        e.kind = efkind::GIOTTO;
        assert_eq!(it.model_item(&e), 39);
        e.kind = efkind::GLARE;
        assert_eq!(it.model_item(&e), 39);
        e.kind = efkind::NOOP;
        assert_eq!(it.model_item(&e), 40);
    }
}
