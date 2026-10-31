//! satchel-digits-4a: the 4a lift charge's countdown display, posed, lit and counted like the game.
//!
//! The game's rules, cited:
//! - **Placement** — `Bullet_DoTrails`' 0x3b tail spawns two kind-0x40 objects every tick in every
//!   state < 2 [M 0x12d890:485-553]: translation = the charge's obj+0x30 + the local offset through
//!   the charge's own rows (`vmulax/vmadday/vmaddz` on obj+0x90/a0/b0 [M disasm 0x12e4c8-0x12e4dc,
//!   0x12e58c-0x12e59c]) — tens (0x3dbfe246, 0xbb15e052, 0xbdfb5d03), units (0x3d03cf48, …) [M swc1
//!   stores 0x12e47c-0x12e49c]; rotation = `MulMatrix0(obj+0x90, RotMatrix((π/2, 0, π)))` [M :494-495,
//!   :514, :542], rows `Σ_k R[i][k]·S[k]` [M MulMatrix0 0x1e2820].
//! - **The charge's own matrix** — landed: `Mat_Align2Dir(obj+0x90, −normal, (0,1,0), (1,0,0))`
//!   [M Bullet_CollisionHandler 0x12ce80:256-266].
//! - **Glyphs** — `SatchelDigits[sec/10]`, `SatchelDigits[sec%10]` [L SatchelDigits @0x2bed90 =
//!   0x2000791..0x200079a], sec = `(short)(fuse·0.016666668)` [M :422]; the meshes are
//!   OBJECT_LED_0..9, flat in their local xz plane with a +y normal, one batch on texture 11
//!   [D probe 0700004a].
//! - **Lit** — `control_create_object` → `control_init_object` sets the tint bytes obj+0x103..0x106
//!   to 0xFF [M 0x1344e0:19, 0x134630:18-21]; DoTrails never writes them; the LED glist flags 0x20a0
//!   carry no 0x800, so `View_SetupRenderModes_Tweak` passes 0xFF as-is [M 0x1e5888:46-49] — the
//!   draw's colour multiplier is 1.0.
//! - **Burn-out** — `BU+0x102 < 1` redraws the fuse to `Rand_Rand(0x1518) + 0x3c` every tick after
//!   the digits were taken [M :422-433]: the display shows the previous tick's redrawn fuse.
//!
//! The charge: Bond starts on the lift's boarding plate (the `--bond 3.2,-7.0,-3.8,0` start), the
//! ride runs with no input, the landing cut-scene's end raises ch 0x22 and shooter #286 drops its
//! charge on the cab roof [D 0700004a #286; M Shooter_Update 0x17d0c0]. Opt-in: NF_ISO. NF_SILENT.

use nf_formats::{disc, iso9660::IsoImage, level::Level};
use nf_game::player::Input;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

const LED0: u32 = 0x0200_0791;
const SATCHEL_MESH: u32 = 0x0200_07b3;

/// Ride the lift from the boarding plate until the shooter's charge has landed; returns the scene.
fn ride_to_landed_charge() -> nf_view::scene::Scene {
    let mut scene = nf_view::load(&iso(), "0700004a.bin", false).expect("load 0700004a");
    scene.sim.as_mut().unwrap().bond = nf_game::player::Bond::new([3.2, -7.0, -3.8], 0.0);
    for _ in 0..4000 {
        scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false);
        let s = scene.sim.as_ref().unwrap();
        if s.drones.bullets.iter().any(|b| b.landed && b.family(&s.weapons) == 0x3b) { return scene; }
    }
    panic!("the shooter's charge landed on the cab within 4000 ticks of the ride");
}

fn level_4a() -> Level {
    let iso = IsoImage::open(&iso()).unwrap();
    let (entries, bin) = disc::files_bin_entries(&iso).unwrap();
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("0700004a.bin")).unwrap();
    let data = iso.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).unwrap();
    Level::parse(&data).unwrap()
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 { a[0] * b[0] + a[1] * b[1] + a[2] * b[2] }
fn row(m: &nf_formats::anim::Mat4, r: usize) -> [f32; 3] { [m[r][0], m[r][1], m[r][2]] }

/// A world point in the charge's local frame (S rows orthonormal): (p − pos)·S[k].
fn to_local(s: &nf_formats::anim::Mat4, pos: [f32; 3], p: [f32; 3]) -> [f32; 3] {
    let d = [p[0] - pos[0], p[1] - pos[1], p[2] - pos[2]];
    [dot(d, row(s, 0)), dot(d, row(s, 1)), dot(d, row(s, 2))]
}

#[test]
fn the_landed_4a_charge_shows_lit_led_digits_on_its_display_pads() {
    let Ok(_) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set — skipping"); return };
    std::env::set_var("NF_SILENT", "1");
    let scene = ride_to_landed_charge();
    let level = level_4a();
    let sim = scene.sim.as_ref().unwrap();
    let b = sim.drones.bullets.iter().find(|b| b.landed && b.family(&sim.weapons) == 0x3b).unwrap();
    let n = b.stick_normal.expect("the landing captured the cab roof's normal");
    assert!(n[1] > 0.99, "the charge rests on the cab roof (normal up), got {n:?}");
    // The count source [M :422]: the digits are the whole seconds of the fuse.
    let (tens, units) = b.digits.expect("the 0x3b tail wrote the digits");
    let sec = (b.fuse * 0.016_666_668) as i32;
    assert_eq!((tens as i32, units as i32), (sec / 10, sec % 10), "digits = fuse/60 (fuse {})", b.fuse);
    // The charge's own matrix — the projectile slot's drawn pose [M 0x12ce80:256-266].
    let s = nf_game::searchlight::align2dir([-n[0], -n[1], -n[2]], [0.0, 1.0, 0.0], [1.0, 0.0, 0.0]);
    let pos = b.pos;

    // The display pads on the real mesh: OBJECT_SatchelCharge's texture-5 batch (the two red
    // seven-segment pads), a flat face at local z ≈ −0.1201 with a −z normal [D probe].
    let (sm, so) = level.by_hash[&SATCHEL_MESH];
    let pads = level.maps[sm].objects[so].gfx.leaves.iter().flat_map(|l| l.batches.iter())
        .find(|bt| bt.texture == Some(5)).expect("the satchel's display-pad batch");
    let mut pmin = [f32::MAX; 3];
    let mut pmax = [f32::MIN; 3];
    for v in &pads.verts { for c in 0..3 { pmin[c] = pmin[c].min(v.pos[c]); pmax[c] = pmax[c].max(v.pos[c]); } }
    assert!((pmax[2] - pmin[2]).abs() < 1e-5, "the pad batch is one flat face");

    let (layout, _attrs) = scene.skin_layout();
    let (pal, _cols) = scene.skin_palette(&layout, None, 240.0, nf_view::scene::ViewOpts::default(), None);
    let (_m, _pk, attach_mult) = scene.colour_mults_dense();
    let mut centres = Vec::new();
    for (kind, digit) in [(4u8, tens), (5u8, units)] {
        let mesh = LED0 + digit as u32;
        let ai = scene.attachments.iter().position(|a| a.effect == Some((kind, 0, mesh)))
            .unwrap_or_else(|| panic!("kind {kind} slot 0 carries LED {digit}"));
        let m = pal[layout.attach_base as usize + ai];
        // Rotation: rows R·S with R = RotMatrix(π/2, 0, π) = (−x, −z, −y) — the LED's +y normal
        // faces away from the charge (up, off the roof), its x runs along −S.x [M :494-495, :514].
        assert!(dot(row(&m, 0), row(&s, 0)) < -0.9999, "LED x = −charge x, got {:?}", row(&m, 0));
        assert!(dot(row(&m, 1), row(&s, 2)) < -0.9999, "LED normal = −charge z (up), got {:?}", row(&m, 1));
        assert!(dot(row(&m, 2), row(&s, 1)) < -0.9999, "LED z = −charge y, got {:?}", row(&m, 2));
        // Translation, in the charge's frame: exactly the DoTrails offset [M swc1 0x12e47c-0x12e49c].
        let l = to_local(&s, pos, [m[3][0], m[3][1], m[3][2]]);
        let want = if kind == 4 { [0.093_693_3, -0.002_286_93, -0.122_736] } else { [0.032_180_1, -0.002_286_93, -0.122_736] };
        for c in 0..3 { assert!((l[c] - want[c]).abs() < 1e-5, "kind {kind} local offset {l:?} vs {want:?}"); }
        // On the real mesh: centred over the pads, 0..5 mm proud of the pad face (−z is outward).
        assert!(l[0] > pmin[0] && l[0] < pmax[0] && l[1] > pmin[1] && l[1] < pmax[1],
            "kind {kind} sits over the pad batch's rect {pmin:?}..{pmax:?}, got {l:?}");
        let proud = pmin[2] - l[2];
        assert!(proud > 0.0 && proud < 0.005, "kind {kind} floats {proud} above the pad face");
        // The glyph: the slot's vertices are the LED mesh's own (one batch, texture 11, the +y normal).
        let (lm, lo) = level.by_hash[&mesh];
        let led = &level.maps[lm].objects[lo];
        let lb: Vec<_> = led.gfx.leaves.iter().flat_map(|l| l.batches.iter()).collect();
        assert_eq!(lb.len(), 1, "OBJECT_LED_{digit} is one batch");
        assert_eq!(lb[0].texture, Some(11), "OBJECT_LED_{digit} draws texture 11");
        assert!(lb[0].verts.iter().all(|v| v.pos[1] == 0.0 && v.normal == [0, 127, 0]), "flat LED, +y normal");
        let nv = scene.char_verts.iter().filter(|v| v.attachment == Some(ai as u32)).count();
        assert_eq!(nv, lb[0].verts.len(), "the slot draws OBJECT_LED_{digit}'s {} verts", lb[0].verts.len());
        // Lit: the white tweak — multiplier 1.0 [M 0x134630:18-21, 0x1e5888:46-49].
        assert_eq!(attach_mult[ai], [1.0; 3], "kind {kind} keeps the white tweak");
        centres.push(l);
    }
    // Texture 11 is the LED colour: a green glow ramp — every texel green-dominant, the core
    // near full green [D 0700004a texture 11: (1,13,1)..(17,222,17)].
    let tex = level.maps[level.by_hash[&LED0].0].textures.iter().find(|t| t.index == 11).expect("texture 11");
    let pal8 = tex.palette_rgba8();
    let texels: Vec<[u8; 4]> = tex.frame(0).iter().map(|&i| pal8[i as usize]).collect();
    assert!(texels.iter().all(|c| c[1] as u32 > 4 * c[0] as u32 && c[1] as u32 > 4 * c[2] as u32), "texture 11 is green: {texels:?}");
    assert!(texels.iter().any(|c| c[1] > 200), "texture 11's core is bright green");
    assert!(centres[0][0] > centres[1][0], "tens on +x of units (reads left to right with x flipped)");

    // Sisters: the comparisons can fail.
    // (1) The previous placement — (±0.03, 0.032, 0.06) through the charge rows, no turn — lands
    //     the digits BELOW the pad face (inside the roof), outside the pad window.
    for (k, lx) in [(4u8, -0.03f32), (5, 0.03)] {
        let old = [lx, 0.032, 0.06];
        assert!(pmin[2] - old[2] < 0.0, "the old kind-{k} offset is not proud of the pad face");
    }
    // (2) The old multiplier (character 0's ambient ramp) is not the white tweak here.
    let char0 = sim.char_multiplier(0);
    assert_ne!(char0, [1.0; 3], "character 0's ramp differs from the white tweak at the charge");
    // (3) The old (un-turned) matrix fails the normal check: S's own y row is horizontal.
    assert!(dot(row(&s, 1), [0.0, 1.0, 0.0]).abs() < 0.01, "the charge's y row lies in the roof plane");
}

#[test]
fn a_burned_out_charge_shows_the_previous_ticks_redrawn_fuse() {
    let Ok(_) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set — skipping"); return };
    std::env::set_var("NF_SILENT", "1");
    let mut scene = ride_to_landed_charge();
    let bi = { let s = scene.sim.as_ref().unwrap(); s.drones.bullets.iter().position(|b| b.landed && b.family(&s.weapons) == 0x3b).unwrap() };
    // The burn-out state (the laser walk's end state [M 0x12e2f0-0x12e390]; the walk itself is
    // pinned by walkreal_0700004a_satchel_live): BU+0x102 at 0.
    { let b = &mut scene.sim.as_mut().unwrap().drones.bullets[bi]; b.burn = 0; b.defused = true; }
    let mut prev: Option<f32> = None;
    let mut fuses = Vec::new();
    for _ in 0..8 {
        scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false);
        let b = &scene.sim.as_ref().unwrap().drones.bullets[bi];
        assert!(b.state < 2, "the burned-out charge never explodes");
        // The redraw: 0x3c + Rand_Rand(0x1518) ∈ [60, 5459] [M :425-432].
        assert!(b.fuse >= 60.0 && b.fuse <= 5459.0 && b.fuse.fract() == 0.0, "redrawn fuse {}", b.fuse);
        if let Some(p) = prev {
            // The digits come from the PREVIOUS redraw after this tick's decrement [M :422 then :424].
            let sec = ((p - 1.0) * 0.016_666_668) as i32;
            assert_eq!(b.digits, Some(((sec / 10) as u8, (sec % 10) as u8)), "digits from the previous fuse {p}");
        }
        prev = Some(b.fuse);
        fuses.push(b.fuse);
    }
    // Sister: a latched fuse (no redraw) would hold one value — the draws differ tick to tick.
    fuses.dedup();
    assert!(fuses.len() > 4, "the fuse is redrawn every tick, got {fuses:?}");
}
