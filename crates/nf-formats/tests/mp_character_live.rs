//! mp-character-data: the MP character tables against the disc.
//!
//! - `MP_skins` (ACTION.ELF 0x26d488, ELF symbol, 29 × 0x10) — the character index → skin table
//!   `Player_Init` 0x1910f8 reads with `MPSettings+slot*0x30+0x24` [M 0x1910f8:108-117]. The
//!   expected bytes were read straight from the ELF file at its symbol [D].
//! - The 16 animation-set descriptors @0x2b91b0..0x2b9430 — the id tables `PlayerAnimSetInitNormal`
//!   0x18fcd8 / `PlayerAnimSetInitCrouch` 0x18fea0 hand to `AnimSetAppend` per anim-set class [M].
//! - `mp_body_pose`: the pure movement-state → script pick, [M] `Player_ChangeSubState` 0x190630 +
//!   `AnimSetUpdate` 0x1c0938.
//!
//! Opt-in on `NF_ISO` (the ACTION.ELF read).

use nf_formats::iso9660::IsoImage;
use nf_formats::mpanim::{
    anim_set_index, crouch2stand_script, mp_body_pose, mp_body_strafe, stand2crouch_script,
    weapon_anim_class, MpAnimSets, MpBodyState, MpSkins, DEATH_SCRIPT, DEATH_SCRIPT_WATER_ZERO,
    MP_SKINS_COUNT, MP_SPAWN_SCRIPT,
};
use nf_formats::weapon::WeaponData;

fn action_elf() -> Option<Vec<u8>> {
    let path = std::env::var_os("NF_ISO")?;
    let iso = IsoImage::open(&path).unwrap();
    Some(iso.read_file(iso.find("/ACTION.ELF").unwrap()).unwrap())
}

#[test]
fn the_mp_skins_table_is_the_elf_bytes() {
    let Some(elf) = action_elf() else { eprintln!("skip: NF_ISO not set"); return };
    let skins = MpSkins::from_action_elf(&elf).unwrap();
    assert_eq!(skins.entries.len(), MP_SKINS_COUNT);
    // [D] the raw table at the symbol: entry 0, 9, 22 and 28, every field.
    let expect: [(usize, u32, u32, u32); 4] = [
        (0, 0x0500_0089, 0x0100_01b4, 4),
        (9, 0x0500_00c6, 0x0100_021b, 4),
        (22, 0x0500_0081, 0x0100_01aa, 7),
        (28, 0x0500_00b9, 0x0100_0208, 4),
    ];
    for (i, skin, file, sleeve) in expect {
        let e = &skins.entries[i];
        assert_eq!((e.skin, e.file, e.sleeve), (skin, file, sleeve), "MP_skins[{i}] [D 0x26d488+{:#x}]", i * 0x10);
        assert_eq!(e.selected, 0, "the disc's +0x0c flag is 0 (runtime-only) [D]");
    }
    // Every skin hash is 0x0500xxxx and every file hash 0x0100xxxx [D the full table].
    for e in &skins.entries {
        assert_eq!(e.skin >> 24, 0x05, "skin family");
        assert_eq!(e.file >> 24, 0x01, "file family");
        assert!((4..=7).contains(&e.sleeve), "MP sleeve range 4..7 [D]");
    }
}

#[test]
fn the_anim_set_descriptors_are_the_elf_bytes_and_the_switch_picks_them() {
    let Some(elf) = action_elf() else { eprintln!("skip: NF_ISO not set"); return };
    let sets = MpAnimSets::from_action_elf(&elf).unwrap();
    // [D] blob contents: SP (999) normal = all 0x6000a00 @0x2b91b0; class 3 normal @0x2b9200;
    // class 3 crouch @0x2b9228 (walk == run); class 1 normal @0x2b92a0.
    let s999 = sets.set(999, false);
    assert_eq!((s999.idle, s999.walk, s999.run), (0x0600_0a00, 0x0600_0a00, 0x0600_0a00), "[D 0x2b91b0]");
    let s3 = sets.set(3, false);
    assert_eq!((s3.idle, s3.walk, s3.run), (0x0600_01d4, 0x0600_01db, 0x0600_01d9), "[D 0x2b9200]");
    assert_eq!((s3.strafe_left, s3.strafe_right), (0x0600_097c, 0x0600_097d), "[D 0x2b9200 words 4/6 — AnimSetUpdate's dir-1/dir-2 reads 0x1c0f2c/0x1c0f48]");
    let s3c = sets.set(3, true);
    assert_eq!((s3c.idle, s3c.walk, s3c.run), (0x0600_01d7, 0x0600_08f9, 0x0600_08f9), "[D 0x2b9228 — crouch walk == run]");
    let s1 = sets.set(1, false);
    assert_eq!((s1.idle, s1.walk, s1.run), (0x0600_09dc, 0x0600_09de, 0x0600_09db), "[D 0x2b92a0]");
    // [M] the switch mapping picks the blob addresses the decompile names:
    // 999→0x2b91b0, 1→0x2b92a0, 2→0x2b93e0, 3→0x2b9200, 4→0x2b9340, 5→0x2b92f0, 6→0x2b9390,
    // default→0x2b9250; crouch = normal + 0x28 [M 0x18fcd8 / 0x18fea0].
    for (class, va) in [(999u32, 0x2b91b0u32), (1, 0x2b92a0), (2, 0x2b93e0), (3, 0x2b9200), (4, 0x2b9340), (5, 0x2b92f0), (6, 0x2b9390), (0, 0x2b9250), (7, 0x2b9250)] {
        let idx = anim_set_index(class, false);
        assert_eq!(0x2b91b0 + (idx as u32) * 0x28, va, "class {class} → descriptor @0x{va:06x} [M 0x18fcd8]");
        assert_eq!(anim_set_index(class, true), idx + 1, "the crouch descriptor is the next 0x28 block [M 0x18fea0]");
    }
}

#[test]
fn the_weapon_class_byte_is_weapon_data_plus_0x84() {
    let Some(elf) = action_elf() else { eprintln!("skip: NF_ISO not set"); return };
    let wd = WeaponData::from_action_elf(&elf).unwrap();
    // [L ee_start_05.bin @ weapon_data 0x2bf150 + w*0x10C + 0x84] (the boot initialiser's output,
    // the same run WeaponData::from_action_elf replays over the disc ELF):
    // fists 0x01 → 0, P2K 0x02 → 1, the class-2 SMGs 0x12 → 2, the rifle class 0x11 → 3,
    // 0x0e → 4, 0x2c → 5, 0x0d → 6.
    for (w, class) in [(0x01u8, 0u32), (0x02, 1), (0x12, 2), (0x11, 3), (0x0e, 4), (0x2c, 5), (0x0d, 6)] {
        assert_eq!(weapon_anim_class(&wd, w), class, "weapon {w:#04x} anim-set class [L]");
    }
}

#[test]
fn mp_body_pose_picks_the_game_script_per_movement_state() {
    let Some(elf) = action_elf() else { eprintln!("skip: NF_ISO not set"); return };
    let sets = MpAnimSets::from_action_elf(&elf).unwrap();
    let st = |f: fn(&mut MpBodyState)| {
        let mut s = MpBodyState {
            mp: true, weapon: 0x02, substate: 0, fwd: 0.0, strafe: 0.0,
            phase: 0.37, hold: 0, one_shot_time: 1.0, gravity_y: -9.8,
        };
        f(&mut s);
        s
    };
    // [M 0x18fcd8] class 1 (P2K), stand, still → the class-1 idle 0x2b92a0 word 0.
    let class1 = 1u32;
    let p = mp_body_pose(&sets, class1, &st(|_| {}));
    assert_eq!(p.script, 0x0600_09dc, "idle = set word 0 [M 0x1c0938: t <= 0.0002 → arr[0]]");
    assert_eq!(p.blend, 0.0);
    // Walk: t = |fwd|/0.1 [M 0x1c0938:99-112]; the walk script plays with the run weight t.
    let p = mp_body_pose(&sets, class1, &st(|s| s.fwd = 0.05));
    assert_eq!(p.script, 0x0600_09de, "half-speed forward = walk [M 0x1c0938]");
    assert!((p.blend - 0.5).abs() < 1e-5, "run-blend 0.5 [M 0x1c0938], got {}", p.blend);
    let p = mp_body_pose(&sets, class1, &st(|s| s.fwd = 0.1));
    assert_eq!(p.script, 0x0600_09de, "full-speed forward = walk [M]");
    assert!((p.blend - 1.0).abs() < 1e-5, "t clamps to 1 [M], got {}", p.blend);
    let p = mp_body_pose(&sets, class1, &st(|s| s.fwd = -0.04));
    assert_eq!(p.script, 0x0600_09de, "backward walks [M ABS(param_2)]");
    assert!((p.blend - 0.4).abs() < 1e-5, "by |fwd| [M], got {}", p.blend);
    // Crouch: substate 4 → the crouch descriptor [M 0x195b90: obj+0xf6 == 4].
    let p = mp_body_pose(&sets, class1, &st(|s| s.substate = 4));
    assert_eq!(p.script, 0x0600_09d9, "crouch idle = the crouch set's word 0 [D 0x2b92c8]");
    let p = mp_body_pose(&sets, class1, &st(|s| { s.substate = 4; s.fwd = 0.05; }));
    assert_eq!(p.script, 0x0600_09f5, "crouch move: walk == run [D 0x2b92c8]");
    assert!((p.blend - 0.5).abs() < 1e-5, "blend still t [M], got {}", p.blend);
    // The transitions fire while the hold counter runs [M 0x190630 case 0/4 + 0x192310's skip];
    // the direction comes from the NEW stance [M: substate set first, then the transition].
    let p = mp_body_pose(&sets, class1, &st(|s| { s.substate = 4; s.hold = 60; }));
    assert_eq!(p.script, stand2crouch_script(class1), "0x190198 case 1");
    assert_eq!(p.script, 0x0600_09dd, "[M 0x190198 switch]");
    let p = mp_body_pose(&sets, class1, &st(|s| { s.substate = 0; s.hold = 30; }));
    assert_eq!(p.script, crouch2stand_script(class1), "0x190068 case 1");
    assert_eq!(p.script, 0x0600_09da, "[M 0x190068 switch]");
    // SP (mp == false) forces the 999 selector regardless of the weapon [M 0x18fd08-18: movz → 999].
    let sp = mp_body_pose(&sets, 999, &st(|s| { s.mp = false; s.weapon = 0x02; }));
    assert_eq!(sp.script, 0x0600_0a00, "SP stand = the 999 set [D 0x2b91b0]");
    let sp = mp_body_pose(&sets, 999, &st(|s| { s.mp = false; s.substate = 4; }));
    assert_eq!(sp.script, 0x0600_09ff, "SP crouch = the 999 crouch set [D 0x2b91d8]");
    // Death: substates 0xd/0xe pick the death script by the water level [M 0x190630 cases 0xd/0xe].
    let p = mp_body_pose(&sets, class1, &st(|s| s.substate = 0xd));
    assert_eq!(p.script, DEATH_SCRIPT, "gravity Y −9.8 (Player_Init's WldGravity.y [M 0x1910f8:153]) → 0x6000151");
    let p = mp_body_pose(&sets, class1, &st(|s| { s.substate = 0xe; s.gravity_y = 0.0; }));
    assert_eq!(p.script, DEATH_SCRIPT_WATER_ZERO, "gravity Y == 0.0 (zero-g) → 0x60008e0 [M 0x190630]");
    // The MP spawn script is class 3's idle [M 0x1910f8:155-158].
    assert_eq!(MP_SPAWN_SCRIPT, sets.set(3, false).idle);
}

#[test]
fn mp_body_strafe_overlay_follows_the_angle_threshold() {
    let Some(elf) = action_elf() else { eprintln!("skip: NF_ISO not set"); return };
    let sets = MpAnimSets::from_action_elf(&elf).unwrap();
    let st = |f: fn(&mut MpBodyState)| {
        let mut s = MpBodyState {
            mp: true, weapon: 0x02, substate: 0, fwd: 0.0, strafe: 0.0,
            phase: 0.37, hold: 0, one_shot_time: 1.0, gravity_y: -9.8,
        };
        f(&mut s);
        s
    };
    // ratio = strafe/0.075; angle = atan2(ratio, |fwd|/0.1); the overlay shows at
    // |angle·2/π| >= 0.5 [M 0x1c0938 LAB_001c0dc8].
    let p = mp_body_strafe(&sets, 1, &st(|s| s.strafe = 0.05));
    let (script, speed, weight) = p.unwrap();
    assert_eq!(script, 0x0600_09d7, "class 1 strafe word [D 0x2b92a0 word 4]");
    assert!((speed - (0.05f32 / 0.075)).abs() < 1e-5, "overlay speed = |ratio| [M script+0x98]");
    assert!((weight - 1.0).abs() < 1e-5, "pure strafe: angle = π/2 → weight 1 [M script+0xa8]");
    assert!(mp_body_strafe(&sets, 1, &st(|s| s.strafe = -0.05)).unwrap().0 == 0x0600_09d8, "negative strafe → dir 2's word [M 0x1c0f48]");
    // Below the 30° cone: strafe 0.02 while walking 0.05 → ratio 0.267, |fwd|/0.1 = 0.5,
    // angle = atan2(0.267, 0.5) = 0.49 rad → weight 0.312 < 0.5 [M]. (Standing still the angle
    // is π/2 — a pure strafe always overlays.)
    assert!(mp_body_strafe(&sets, 1, &st(|s| { s.strafe = 0.02; s.fwd = 0.05; })).is_none(), "the 0.5 cone gate [M]");
    assert!(mp_body_strafe(&sets, 1, &st(|s| s.strafe = 0.0001)).is_none(), "|strafe| <= 0.0002 → none [M]");
    // SP never overlays [M the MPSettings._384_4_ == 0 early return].
    assert!(mp_body_strafe(&sets, 999, &st(|s| { s.mp = false; s.strafe = 0.05; })).is_none());
    // The crouch set's strafe words [D 0x2b92c8 words 4/6].
    let p = mp_body_strafe(&sets, 1, &st(|s| { s.substate = 4; s.strafe = 0.05; }));
    assert_eq!(p.unwrap().0, 0x0600_09f3, "crouch strafe left [D 0x2b92c8]");
}
