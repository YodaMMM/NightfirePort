//! floating-rifle-02-2: the 0x8019/0x8018 0x8000000 mark is COLLISION-ONLY — the flagged statics
//! DRAW. 786's "bake, not drawn" reading skipped every identity-rotation 0x8019/0x8018 and broke
//! decal_order_live (07000007's hall_shadows*/OBJECT_banistershadows* dropped). The decompile says
//! otherwise:
//! - `parseentity_fixup_entity` 0x1d00c0 sets the bit when the placement's translation AND rotation
//!   vectors (cel+0x60/+0x70) are BOTH exactly zero — `Vec_IsEqual` with epsilon 0.0f against the
//!   zero vector at 0x2d6200 (disasm 0x1d02e0-0x1d0324: `mtc1 $zero, $f20`, constant 16 zero bytes);
//! - the bit's only consumers are the collision fast paths `Collide_Pick` 0x1eb320:166,249 /
//!   `Collide_Intersect` 0x1ebaa8:142 (the `Ident` identity-rotation shortcut); no draw path reads it;
//! - `View_AddCels` 0x1e6ba0 walks every inner-chain 0x8019 (glist+0x44 bit 0x10 gate, the
//!   0x8018-vs-0xc023 special case, `Vision_InView`) and `View_DrawCelGlist` 0x1e6538 draws it at
//!   Rot(cel+0x70)·Trans(cel+0x60) — for the sandals (t=r=0 [D]) the identity transform at the art's
//!   absolute coordinates.
//! The 07000002 sandals (map 51 objects 143/144, kind 0x8019, t=r=0, glist centres
//! (−39.098, 5.395, 11.266)/(−39.073, 5.396, 9.961), radii 0.339/0.379, glist+0x44 = 0 [D]) draw at
//! those measured positions; PCSX2 shows nothing at the doorway eye [L 2026-10-20, fresh boot] — the
//! remaining visibility question is the report's live checks, not a port-side hide rule.
//!
//! Run: `NF_ISO=… cargo test -p nf-view --test baked_statics_02_live -- --test-threads=1`

use nf_formats::{disc, iso9660::IsoImage, level::Level};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

/// The port's mirror of `parseentity_fixup_entity` 0x1d00c0's 0x8019/0x8018 mark condition: the
/// placement's translation and rotation vectors both exactly zero (every component ±0.0 — the
/// game's `Vec_IsEqual` with epsilon 0.0f accepts exactly |diff| ≤ 0, i.e. diff == ±0.0; NaN fails).
fn zero_placement_mark(t: [f32; 3], r: [f32; 3]) -> bool {
    let zero = |v: [f32; 3]| v.iter().all(|c| c.to_bits() & 0x7fff_ffff == 0);
    zero(t) && zero(r)
}

/// The placement's `cel+0xa3` byte per the 0x8019 arm (0x1d00c0:162): `props[0] | props[1] << 4`
/// over the placement's parameter table (props[k] = the value of parameter id k, 0 when absent).
fn tweak_byte(params: &[(u32, u32)]) -> u8 {
    let prop = |k: u32| params.iter().find(|(i, _)| *i == k).map(|&(_, v)| v as u8).unwrap_or(0);
    prop(0) | ((prop(1) as i8 as u8) << 4)
}

/// `View_DrawCelGlist` 0x1e6538's psiSetTweakARGB alpha byte (disasm 0x1e656c-0x1e659c):
/// mode = a3 & 3, nib = a3 >> 4, alpha = GameState[0x34]·nib (mode 2), 255 − that (mode 1),
/// 0xff (modes 0/3). The GameState+0x34 byte's writer is not in the decompile — the CALLER
/// supplies it; no shipped static uses modes 1/2 (all probed 0x8019s on 07000002/07000007 are
/// mode 0), so the source stays [S] and out of the render path.
pub fn cel_tweak_alpha(a3: u8, gs34: u8) -> u8 {
    let (mode, nib) = (a3 & 3, a3 >> 4);
    match mode {
        1 => 255u8.wrapping_sub(gs34.wrapping_mul(nib)),
        2 => gs34.wrapping_mul(nib),
        _ => 0xff,
    }
}

fn load_level(bin_name: &str) -> Level {
    let im = IsoImage::open(&iso()).unwrap();
    let (entries, bin) = disc::files_bin_entries(&im).unwrap();
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case(bin_name)).unwrap();
    Level::parse(&im.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).unwrap()).unwrap()
}

/// The named object's first main-map instance: its placement and the object's glist centre [D].
fn named(level: &Level, name: &str) -> (nf_formats::placement::Placement, [f32; 3]) {
    let (instances, _) = level.instances(true);
    let inst = instances.iter().find(|i| level.maps[i.map].objects[i.object].name.trim() == name)
        .unwrap_or_else(|| panic!("{name} placed [D placements]"));
    (inst.placement.clone(), level.maps[inst.map].objects[inst.object].centre)
}

/// The instance's world centre: the glist centre through the instance matrix.
fn world_centre(level: &Level, inst: &nf_formats::placement::Placement, centre: [f32; 3]) -> [f32; 3] {
    let _ = level;
    let m = nf_formats::placement::object_matrix(inst.rotation, inst.translation);
    nf_formats::placement::transform(&m, centre)
}

/// The draw list maps 1:1 onto `scene.materials` (the gpu draw index IS the material index
/// [M gpu.rs 615-616]), so a material near a point is a draw there.
fn draws_near(scene: &nf_view::scene::Scene, p: [f32; 3], r: f32) -> Vec<usize> {
    scene.materials.iter().enumerate().filter_map(|(k, m)| {
        let d = ((m.centre[0] - p[0]).powi(2) + (m.centre[1] - p[1]).powi(2) + (m.centre[2] - p[2]).powi(2)).sqrt();
        (d <= r).then_some(k)
    }).collect()
}

#[test]
fn the_mark_is_the_zero_placement_condition_and_the_sandals_carry_it() {
    let level = load_level("07000002.bin");
    // The two sandals: kind 0x8019, t=r=0 → the game's bit 0x8000000 [M 0x1d00c0; D placements].
    for name in ["bath-sandalorange", "bath-sandalblue"] {
        let (pl, _) = named(&level, name);
        assert_eq!(pl.flags & 0xffff, 0x8019, "{name} is kind 0x8019 [D]");
        assert!(zero_placement_mark(pl.translation, pl.rotation),
            "{name} t=r=0 exactly (the game's mark condition, epsilon 0.0f) [D]");
        assert!(pl.params.is_empty(), "{name} carries no parameters → cel+0xa3 = 0 [D]");
        assert_eq!(tweak_byte(&pl.params), 0, "{name} tweak byte 0 → mode 0 → alpha 0xff [M 0x1d00c0:162]");
    }
    // Negative control [D]: the rotated 0x8019 light fixture keeps yaw 0.733 — NOT marked.
    let (instances, _) = level.instances(true);
    let fixture = instances.iter().find(|i| {
        level.maps[i.map].objects[i.object].name.trim() == "northerngarden4_lightfixture"
            && (i.placement.translation[0] + 41.21).abs() < 0.05
            && (i.placement.translation[2] + 34.74).abs() < 0.05
    }).expect("the light fixture placed [D placements]").placement.clone();
    assert_eq!(fixture.flags & 0xffff, 0x8019, "the fixture is kind 0x8019 [D]");
    assert!(!zero_placement_mark(fixture.translation, fixture.rotation),
        "the fixture's yaw 0.733 placement is not the zero mark [D]");
    // 07000007's shadow decals: the records 786's skip dropped — mode 0 tweak bytes → drawn.
    let seven = load_level("07000007.bin");
    let (sevens, _) = seven.instances(true);
    let mut shadow_kinds = 0;
    for inst in &sevens {
        if inst.placement.flags & 0xffff != 0x8019 { continue; }
        let n = seven.maps[inst.map].objects[inst.object].name.trim().to_string();
        if !(n.contains("shadow") || n.contains("Shadow")) { continue; }
        shadow_kinds += 1;
        assert_eq!(cel_tweak_alpha(tweak_byte(&inst.placement.params), 0), 0xff,
            "{n}: the shadow decal's tweak alpha is 0xff — the game draws it [M 0x1e6538]");
    }
    assert!(shadow_kinds >= 10, "07000007 has its shadow-decal 0x8019 set on disc [D] (got {shadow_kinds})");
}

#[test]
fn the_sandals_draw_at_the_games_measured_position() {
    let level_data = load_level("07000002.bin");
    // The draw at the driver's doorway eye (the playtest spot).
    let mut scene = nf_view::load(&iso(), "07000002.bin", false).expect("load 07000002");
    let sim = scene.sim.as_mut().unwrap();
    sim.drones.hold_npcs = true;
    sim.bond.pos = [-43.11, 6.46, 10.37];
    sim.bond.yaw = 2.305;
    sim.bond.pitch = 0.0;
    for _ in 0..=300u32 {
        sim.step_bond(nf_game::player::Input::default(), false, false, false);
    }
    // THE FIX (the inverse of 786's): the flagged sandals DRAW at their glist centres — the game's
    // measured position [M 0x1e6538 identity transform; D glist bounds]. Pre-786 the port drew
    // exactly these 5 batches (4× texture 142 + the strap's texture 12).
    for name in ["bath-sandalorange", "bath-sandalblue"] {
        let (pl, c) = named(&level_data, name);
        let centre = world_centre(&level_data, &pl, c);
        let hits = draws_near(&scene, centre, 0.5);
        assert!(!hits.is_empty(),
            "{name} draws at its measured position {centre:?} (the 0x8000000 mark is collision-only [M 0x1eb320:166])");
    }
    // SISTER (can fail): the rotated 0x8019 fixture still draws at its placement spot.
    let (fpl, fc) = named(&level_data, "northerngarden4_lightfixture");
    let fixture_at = world_centre(&level_data, &fpl, fc);
    assert!(!draws_near(&scene, fixture_at, 1.0).is_empty(),
        "the rotated light fixture (0x8019, yaw 0.733) draws at {fixture_at:?}");
    // SISTER (can fail): the room cels themselves still draw — the World layer keeps cel draws.
    assert!(scene.materials.iter().any(|m| m.layer == nf_view::scene::Layer::World && m.cel.is_some()),
        "room-cel World draws survive [D]");
}

#[test]
fn the_tweak_alpha_math_matches_view_drawcelglist() {
    // Disasm 0x1e656c-0x1e659c: mode = a3 & 3; nib = a3 >> 4; mode 0/3 → 0xff;
    // mode 1 → 255 − gs34·nib; mode 2 → gs34·nib (byte arithmetic, u8 wrap).
    assert_eq!(cel_tweak_alpha(0x00, 200), 0xff); // mode 0
    assert_eq!(cel_tweak_alpha(0x30, 200), 0xff); // mode 3
    assert_eq!(cel_tweak_alpha(0x10, 200), 0xff); // mode 0, nib 1 — the parsed shadow decals' byte
    assert_eq!(cel_tweak_alpha(0x50, 200), 0xff); // mode 0, nib 5 — the SliderOilPainting byte
    assert_eq!(cel_tweak_alpha(0x21, 3), 249);    // mode 1, nib 2: 255 − 3·2
    assert_eq!(cel_tweak_alpha(0x22, 3), 6);      // mode 2, nib 2: 3·2
    assert_eq!(cel_tweak_alpha(0x21, 0), 0xff);   // mode 1, gs34 0: 255 − 0
    assert_eq!(cel_tweak_alpha(0x22, 0), 0);      // mode 2, gs34 0: 0 — fully transparent
    assert_eq!(cel_tweak_alpha(0xf2, 1), 15);     // mode 2, nib 15: 1·15
}
