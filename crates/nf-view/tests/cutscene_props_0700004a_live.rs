//! cutscene-props: 0700004a's −52 landing cut-scene (SP #289, pack 0x060006cf, 575 ticks) — the
//! charge in the guard's hands and the lift roof under Bond, pinned on what the renderer DRAWS.
//!
//! [D 0780004a.bin 0x060006cf, `nf-extract nis`]:
//! - s1 op 0xA: the red-beret guard (skin 0x05000006, script 0x060006d2); s3 op 0xA: the
//!   capped guard (skin 0x05000004, script 0x060006d0) — s4 t0 event 0xB param 0x18 hands HIM
//!   weapon 0x18 (`Script_EventHandler` 0x1d94c0 case 0xb, the previous stream's anim object [M]).
//! - s7 op 7: "OBJECT_DemoCharge" 0x02000758 (no parent), keyed 133..594 — t0 (3.429, 1.488,
//!   −0.836), t182 (3.538, 0.880, −0.650), t575 (3.226, −0.776, −2.002): the charge itself, a keyed
//!   prop that travels with the red-beret guard's hands (the PS2 frame of this scene shows the
//!   charge in the red-beret guard's hands [L audits live-4a-cast2 ride_1850]).
//! - s2 op 0xA: Bond (skin 0x05000007, script 0x060006d1).
//! - s10 op 7: "NIS_LIFTHATCH" 0x020006e1 t0 (3.05, −7.990, −3.08) → t575 (3.05, −14.314, −3.08);
//!   s13 op 7: "NIS_lift" 0x020006ee t0 (0, 0.990, 0) → t575 (0, −5.334, 0) — the car (glist box
//!   x 0.89..5.21, z −5.58..−1.3 [D 0700004a.bin]) and its roof hatch, descending together
//!   (hatch − car = −8.98 on every key). The roof Bond kneels on is the hatch's plane.
//! - s0 t0 op 0x0d (`Script_CameraStart` 0x1d9140 → identity on si+0x00 [M 0x1d916c]) before any
//!   entity keys, so these entity keys are the drawn matrices; composing SP #289's placement
//!   (0, −53.534, −3.377) [D] threw the car ~53 units under a cast drawn on its raw tracks — the
//!   "Bond on nothing" state the second test's sister rejects.
//!
//! Opt-in via `NF_ISO`.

use nf_game::player::Input;

const PACK: u32 = 0x0600_06cf;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

struct Sample {
    t: u32,
    /// The s7 entity's translation.
    satchel: Option<[f32; 3]>,
    /// Bond's (s2) lowest posed bone, world.
    bond_feet: Option<[f32; 3]>,
    hatch: Option<[f32; 3]>,
    car: Option<[f32; 3]>,
    car_slot_hidden: Option<bool>,
    charge_slot_hidden: Option<bool>,
    /// Per pack character (stream): the nearest posed bone to the s7 charge (distance, bone).
    near: Vec<(u8, f32, usize)>,
}

fn run_scene(sample_at: &[u32]) -> (Vec<Sample>, u32) {
    std::env::set_var("NF_SILENT", "1");
    let mut scene = nf_view::load(&iso().unwrap(), "0700004a.bin", false).expect("load 0700004a");
    let mut out = Vec::new();
    let mut t = 0u32;
    // The lift starts on its own (SP #42 mode 1 [D]); Bond rides it from the start spot and the
    // −52 landing scene disables him (event 4 [D s9 t0]).
    for _ in 0..6800u32 {
        scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false);
        let sim = scene.sim.as_ref().unwrap();
        if !sim.disabled { if t > 0 { break } else { continue } }
        t += 1;
        if !sample_at.contains(&t) { continue }
        let tr = |m: [[f32; 4]; 4]| [m[3][0], m[3][1], m[3][2]];
        let fs = scene.frame_state_for(None, 0.0, nf_view::scene::ViewOpts::default(), false);
        let mut s = Sample { t, satchel: sim.script_entity((PACK, 7), 0x0200_0758).map(tr),
            bond_feet: None, hatch: sim.script_entity((PACK, 10), 0x0200_06e1).map(tr), car: sim.script_entity((PACK, 13), 0x0200_06ee).map(tr),
            car_slot_hidden: None, charge_slot_hidden: None, near: Vec::new() };
        for (ci, c) in scene.characters.iter().enumerate() {
            if let (Some((h, si)), Some(sat), Some(Some((_, world, m)))) = (c.script, s.satchel, fs.posed.get(ci)) {
                if h == PACK {
                    let mut best = (f32::MAX, 0usize);
                    for (bi, b) in world.iter().enumerate() {
                        let p = [b[3][0], b[3][1], b[3][2]];
                        let w = [0, 1, 2].map(|k| m[0][k] * p[0] + m[1][k] * p[1] + m[2][k] * p[2] + m[3][k]);
                        let d = dist(w, sat);
                        if d < best.0 { best = (d, bi); }
                    }
                    eprintln!("   s{si} skin {:08x}: nearest bone {} at {:.3} from the s7 charge", c.skin.hashcode, best.1, best.0);
                    s.near.push((si, best.0, best.1));
                }
            }
            match c.script {
                Some((h, 2)) if h == PACK => {
                    if let Some(Some((_, world, m))) = fs.posed.get(ci) {
                        let mut low: Option<[f32; 3]> = None;
                        for b in world.iter() {
                            let p = [b[3][0], b[3][1], b[3][2]];
                            let w = [0, 1, 2].map(|k| m[0][k] * p[0] + m[1][k] * p[1] + m[2][k] * p[2] + m[3][k]);
                            if low.map_or(true, |l| w[1] < l[1]) { low = Some(w); }
                        }
                        s.bond_feet = low;
                    }
                }
                _ => {}
            }
        }
        for (ai, a) in scene.attachments.iter().enumerate() {
            if a.script.is_some_and(|(h, si, hash, _)| h == PACK && si == 13 && hash == 0x0200_06ee) {
                let hid = fs.hidden.get(ai).copied().unwrap_or(false);
                s.car_slot_hidden = Some(s.car_slot_hidden.map_or(hid, |p| p && hid));
            }
            if a.script.is_some_and(|(h, si, hash, _)| h == PACK && si == 7 && hash == 0x0200_0758) {
                let hid = fs.hidden.get(ai).copied().unwrap_or(false);
                s.charge_slot_hidden = Some(s.charge_slot_hidden.map_or(hid, |p| p && hid));
            }
        }
        eprintln!("t {t}: charge {:?} feet {:?} hatch {:?} car {:?} car hidden {:?}",
            s.satchel, s.bond_feet, s.hatch, s.car, s.car_slot_hidden);
        out.push(s);
    }
    (out, t)
}

fn dist(a: [f32; 3], b: [f32; 3]) -> f32 { ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt() }

#[test]
fn the_red_beret_guard_holds_the_charge() {
    let Some(_) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let (samples, ticks) = run_scene(&[100, 182, 300]);
    assert!(ticks >= 500, "the landing scene played (~575 ticks [D]), got {ticks}");
    for s in &samples {
        s.satchel.unwrap_or_else(|| panic!("t {}: the s7 charge entity 0x02000758 is not live [D]", s.t));
        assert_eq!(s.charge_slot_hidden, Some(false), "t {}: the charge has no visible draw slot", s.t);
        let near = |si: u8| s.near.iter().find(|n| n.0 == si).map(|n| n.1)
            .unwrap_or_else(|| panic!("t {}: the s{si} character is not posed", s.t));
        // The charge sits in the red-beret guard's (s1) hands: one of his posed bones within 0.3.
        assert!(near(1) < 0.3, "t {}: the s7 charge is {:.2} from the red-beret guard's nearest bone — not in his hands", s.t, near(1));
        // Sister: the same check rejects a character that does not hold it (Bond, ~3 units away).
        assert!(near(2) > 1.0, "sister: Bond's nearest bone must be far from the charge, got {:.2}", near(2));
    }
}

#[test]
fn bond_kneels_on_the_lift_roof_through_the_scene() {
    let Some(_) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let (samples, ticks) = run_scene(&[1, 100, 200, 300, 400, 500, 570]);
    assert!(ticks >= 500, "the landing scene played (~575 ticks [D]), got {ticks}");
    for s in &samples {
        let feet = s.bond_feet.unwrap_or_else(|| panic!("t {}: Bond (s2) is not posed", s.t));
        let hatch = s.hatch.expect("the hatch s10 is live [D]");
        let car = s.car.expect("the car s13 is live [D]");
        assert_eq!(s.car_slot_hidden, Some(false), "t {}: the car NIS_lift has a visible draw slot", s.t);
        // On the roof: the feet at the hatch plane, and over the car's glist box [D].
        assert!((feet[1] - hatch[1]).abs() < 0.35, "t {}: Bond's feet y {:.2} vs the roof hatch plane {:.2} — he must kneel on the roof",
            s.t, feet[1], hatch[1]);
        let (lx, lz) = (feet[0] - car[0], feet[2] - car[2]);
        assert!((0.89..=5.21).contains(&lx) && (-5.58..=-1.3).contains(&lz),
            "t {}: Bond's feet ({:.2}, {:.2}) are off the car's footprint x 0.89..5.21 z −5.58..−1.3 [D glist]", s.t, lx, lz);
        // Sister: the composed state (SP #289's placement on the car, the hatch with it) leaves
        // Bond ~53 units above the roof — the check rejects it.
        let composed_hatch = hatch[1] - 53.534;
        assert!((feet[1] - composed_hatch).abs() > 0.35, "sister: the roof check must reject the composed car");
    }
}
