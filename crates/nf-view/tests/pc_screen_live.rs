//! pc-screen: what the hacked computers' screens do after the install, pinned through the real
//! loader (NF_ISO) with sisters.
//!
//! 1. `Control_NextLOD` 0x1349b0 swaps obj+0xD8 to `*(glist+0x40)` only when that link is non-null
//!    [M :8-12]; `PCQWorm_Update` 0x1aee20 state 1 calls it for a PC with param 0 != 0 [M :24-25].
//!    The PC's obj+0xD8 is its placement glist (`PCQWorm_Create` 0x1b07a8:12 → `Control_CreateObjEx`
//!    0x1343c8:29; `parsemap_block_map_data_dynamic` 0x1d1b68:77-80 = pCelList + index·0x50) and the
//!    only writer of glist+0x40 is `parsemap_block_LOD` 0x1d1f10 (block 0x27). Every PCQWorm glist on
//!    0a/0b/11/12 carries −1 there [D], so the call returns 0 and the model never changes — on this
//!    disc there is no LOD swap to draw.
//! 2. The visible change is the Q-Worm screen overlay: a script entity (0a SCREEN_Q 0x02000A22,
//!    12 qworm2b 0x02000A28, …) whose ScriptPlayer the install's channel unpauses; `Script_Interp`
//!    0x1d9a08:104 stores the stream key's +0x24 byte in obj+0x106 and `View_SetupRenderModes_Tweak`
//!    0x1e5888:12,47-49 draws with it as the alpha factor. The keys hold 0 until the install, then
//!    pulse 0 → 1 → 0. The port drew the overlay at alpha 1 from the level start (a "Q" on the screen
//!    before any install) — now the palette slot carries the key alpha.

use nf_formats::iso9660::IsoImage;
use nf_formats::{disc, level::Level};
use nf_view::scene::{Scene, Sim, ViewOpts};

type V3 = [f32; 3];

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

fn load(level: &str) -> Option<Scene> {
    let iso = iso()?;
    Some(nf_view::load_with_carry_weapons_difficulty(&iso, level, false, None, &[], 3).unwrap_or_else(|e| panic!("load {level}: {e}")))
}

fn opts() -> ViewOpts { ViewOpts { reinforcements: false, firing: false, right: [1.0, 0.0, 0.0], up: [0.0, 1.0, 0.0] } }

/// (palette alpha, palette translation) of the script-entity slot drawing `hash`.
fn slot(scene: &Scene, hash: u32) -> (f32, [f32; 3]) {
    let ai = scene.attachments.iter().position(|a| a.script.is_some_and(|(_, _, h, _)| h == hash))
        .unwrap_or_else(|| panic!("a draw slot for {hash:#010x}"));
    let (layout, _) = scene.skin_layout();
    let (pal, cols) = scene.skin_palette(&layout, None, 0.0, opts(), None);
    let m = pal[(layout.attach_base + ai as u32) as usize];
    (cols[(layout.col_att + ai as u32) as usize][3], [m[3][0], m[3][1], m[3][2]])
}

fn idle(sim: &mut Sim, n: usize) { for _ in 0..n { sim.step_bond(Default::default(), false, false, false); } }

fn dist(a: V3, b: V3) -> f32 { ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt() }

// ---------------------------------------------------------------------------------------------
// 1. No LOD link on any PCQWorm glist.
// ---------------------------------------------------------------------------------------------

#[test]
fn pcqworm_glists_carry_no_lod_link_so_next_lod_is_a_no_op() {
    let Some(iso) = iso() else { eprintln!("NF_ISO unset — skipping"); return };
    let image = IsoImage::open(&iso).expect("iso");
    let (entries, bin) = disc::files_bin_entries(&image).expect("entries");
    let mut pcs = Vec::new();
    let mut keypad_link = None;
    for name in ["0700000a.bin", "0700000b.bin", "07000011.bin", "07000012.bin"] {
        let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case(name)).expect("level");
        let data = image.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read");
        let lvl = Level::parse(&data).expect("parse");
        for (m, map) in lvl.maps.iter().enumerate() {
            for (pi, p) in map.placements.iter().enumerate().filter(|(_, p)| p.flags & 0xff == 0x35) {
                // parsemap_block_map_data_dynamic 0x1d1b68:77-80: hash −1 → pCelList[index].
                assert!(!p.by_hash(), "{name} #{pi}: the PCs index their own map's cel list [D]");
                let o = p.index as usize;
                pcs.push((name, pi, map.objects[o].name.trim().to_string(), map.lod.get(o).copied(), map.lod_target(o)));
            }
            // Sister: the shared gadget map's keypad lock DOES link (Locked → Unlocked), so a link
            // on a PC glist would be seen by the same read.
            if let Some(k) = map.objects.iter().position(|o| o.name == "Lock - KeypadLocked") {
                if keypad_link.is_none() { keypad_link = map.lod_target(k).map(|t| (m, map.objects[t].name.clone())); }
            }
        }
    }
    let ids: Vec<_> = pcs.iter().map(|(n, pi, _, _, _)| format!("{}#{pi}", &n[6..8])).collect();
    assert_eq!(ids, ["0a#683", "0a#684", "0a#688", "0b#297", "11#216", "12#684"], "the disc's PCQWorm placements [D]");
    for (name, pi, obj, word, target) in &pcs {
        assert_eq!(*word, Some(-1), "{name} #{pi} '{obj}': block-0x27 word (glist+0x40) is −1 [D]");
        assert!(target.is_none(), "{name} #{pi} '{obj}': no Control_NextLOD target");
    }
    let (_, unlocked) = keypad_link.expect("sister: the keypad lock's glist links to its LOD variant [D 010000cf]");
    assert_eq!(unlocked, "Lock - KeypadUnlocked");
}

// ---------------------------------------------------------------------------------------------
// 2. The Q-Worm screen overlay takes the key alpha.
// ---------------------------------------------------------------------------------------------

const SCREEN_Q: u32 = 0x0200_0a22; // 0700000a obj 574 "SCREEN_Q" [D]
const QWORM2B: u32 = 0x0200_0a28; // 07000012 obj 592 "qworm2b" [D]

#[test]
fn qworm_pc_screen_overlay_is_hidden_until_the_install_then_shows() {
    let Some(mut sc) = load("0700000a.bin") else { eprintln!("NF_ISO unset — skipping"); return };
    assert!(sc.script_tweak_alpha.contains(&SCREEN_Q), "SCREEN_Q's batch blends (PRIM ABE) — the key alpha reaches it");
    let pi = sc.sim.as_ref().unwrap().drones.useobjs.pcs.iter().position(|p| p.ch == 0x48).expect("Q-WORM_PC_1 #688 [D]");
    let pc = sc.sim.as_ref().unwrap().drones.useobjs.pcs[pi].sph_centre;
    idle(sc.sim.as_mut().unwrap(), 30);
    let (a0, at) = slot(&sc, SCREEN_Q);
    assert!(dist(at, pc) < 0.5, "the overlay is posed at the PC (not parked): {at:?} vs {pc:?}");
    assert_eq!(a0, 0.0, "before the install the overlay's key alpha is 0 — no Q on the screen");
    // The install: obj+0xF4 = 1 is the state the Q-Worm bullet leaves [M 0x1aee20:49-82 → LAB_001aefec];
    // the bullet path itself is pinned by qworm_fire_mode_live.
    sc.sim.as_mut().unwrap().drones.useobjs.pcs[pi].state = 1;
    let mut peak = 0.0f32;
    for _ in 0..90 {
        idle(sc.sim.as_mut().unwrap(), 1);
        peak = peak.max(slot(&sc, SCREEN_Q).0);
    }
    assert!(sc.sim.as_ref().unwrap().drones.channels.on(0x48), "the install wrote ch 0x48 [M 0x1aee20:34-36]");
    assert!(peak > 0.9, "after the install the overlay fades in (key alpha → 1): peak {peak}");
}

/// One E press at a time while aiming at `target`, until `done`.
fn aim_use(sim: &mut Sim, target: V3, done: &dyn Fn(&Sim) -> bool) -> bool {
    for k in 0..360 {
        let eye = [sim.bond.pos[0], sim.bond.pos[1] + sim.bond.eye - sim.bond.crouch_drop, sim.bond.pos[2]];
        let (dx, dz, dy) = (target[0] - eye[0], target[2] - eye[2], target[1] - eye[1]);
        let err = dx.atan2(dz) - sim.bond.yaw;
        let want_pitch = (dy / (dx * dx + dz * dz).sqrt().max(0.5)).atan() / std::f32::consts::FRAC_PI_2;
        let input = nf_game::player::Input { turn: err.clamp(-0.08, 0.08), look: (want_pitch - sim.bond.pitch).clamp(-0.08, 0.08), ..Default::default() };
        sim.step_bond(input, false, false, k % 6 == 0);
        if done(sim) { return true; }
    }
    false
}

#[test]
fn hack_computer_e_install_brings_up_the_qworm_screen_on_12() {
    let Some(mut sc) = load("07000012.bin") else { eprintln!("NF_ISO unset — skipping"); return };
    let sim = sc.sim.as_mut().unwrap();
    let pi = sim.drones.useobjs.pcs.iter().position(|p| p.ch == 0x34 && p.e_usable != 0).expect("Computer to hack #684 [D]");
    let target = sim.drones.useobjs.pcs[pi].sph_centre;
    // Placed once in front of the screen (the place-once stand-in of the hud2 tests; the walked
    // approach is interactables_07000012_live's): the "Press Action" prompt shows from here.
    sim.bond.pos = [-17.5, 0.0, -33.6];
    idle(sim, 60);
    let (a0, at) = slot(&sc, QWORM2B);
    assert!(dist(at, target) < 0.5, "qworm2b is posed at the PC: {at:?} vs {target:?}");
    assert_eq!(a0, 0.0, "sister: no overlay before the E");
    let sim = sc.sim.as_mut().unwrap();
    assert!(aim_use(sim, target, &|s| s.drones.useobjs.pcs[pi].state >= 2),
        "E on the PC installs [M PCQWorm_Activate 0x1b0840 → 0x1aee20 state 1]");
    let mut peak = 0.0f32;
    for _ in 0..90 {
        idle(sc.sim.as_mut().unwrap(), 1);
        peak = peak.max(slot(&sc, QWORM2B).0);
    }
    assert!(peak > 0.9, "the E install fades the Q-Worm screen in: peak {peak}");
}
