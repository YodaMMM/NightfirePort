//! boot-loadout-qworm: the per-level START LOADOUT rows decoded FROM THE DISC and pinned against
//! the port's boot path, every on-foot level.
//!
//! The disc has no loadout table in the level archives — the level-start inventory is built by
//! `Player_InitWeapon` (0x1a7ef8), a compiled per-level switch on the level id: `lw a0, 0x3774(v1)`
//! reads the level id (GameState+0xC, 0x2a3774), `sltiu v1, a0, 0x4a` bounds it and the 74-word jump
//! table at 0x2FC870 (`lui v0, 0x30` + `addiu v0, v0, -0x3790` + `sll v1, a0, 2` + `jr`) dispatches to
//! the case arms [M 001a825c-001a8284; the table bytes read back per level below]. Each arm calls
//! `Player_EquipWeapon(BL, weapon, rounds)` 0x1a8a58 with the weapon/rounds as immediates (or as the
//! s-registers the prologue precomputed: `s0 = Upgrade_Weapon(6)` = 2, `s2 = Upg(0x4a)`, `s5 =
//! Upg(0x43)`, `s6 = Upg(0x1e)`, `s7 = Upg(0x56)`, `s4 = 0x5b` on the 60 Hz NTSC build (the
//! `movz s4, v0, a0` PAL-50 Hz pick at 001a8250 does not fire — the disc is NTSC [D
//! docs/build-identity.md])), and the shared tail equips `Upgrade_Weapon(0x4e)` @999 = 0x4e and the
//! fists 1 @0 [M 001a7ef8:318-319; Upgrade_Weapon 0x1a8f30 with the boot tables UpgradedHandguns
//! 0x2bee08 = [2,4,6,8] .. Upgrades 0x2beea8 = 0 [D ELF bytes]] — so the "rows" this test reads are
//! the EquipWeapon immediates themselves, straight out of /ACTION.ELF on the disc.
//!
//! The table indexing was verified entry by entry: entry k serves the case the decompile labels
//! 0x700000(k+1) (e.g. k=9 → the 0a/0b RamLoad arm at 0x1a8418 falling into the 09 kit at 0x1a8448;
//! k=0x13 → the fresh 14 arm at 0x1a8618; k=0x49 → case 0x700004a sharing 12/13's arm 0x1a84dc) —
//! the decompile's `case 0x7000015: case 0x7000016: case 0x7000017:` grouping matches k=0x14/15/16 →
//! 0x1a85e8 [M the switchD labels in 001a7ef8's decompile].
//!
//! Test 1 boots EVERY on-foot archive through the real viewer load (`nf_view::load` — the path the
//! player runs) and pins the owned set and the switch request against the disc rows, applied through
//! the game's own refusal rules (`Player_EquipWeapon` 0x1a8a58:20 refuses an equip whose first-person
//! model is not loaded — the port's `sim.skins` is that hashtable; `Player_CheckWeaponsLoaded`
//! 0x1a97c8 revokes a selectable weapon whose skin is missing).
//!
//! Test 2 (the sister) calls the port's `init_level_loadout` directly per level with the refusal
//! disabled (`loaded = |_| true` — the raw kit list, the same direct call the dev panel's levelkit
//! runs) and asserts each disc GADGET row (id ≥ 0x48) individually — when the port's loadout drops a
//! disc gadget row, this test fails naming the level and the gadget. The headline row is the
//! Q-Worm: the 09/0a/0b kits equip 0x58 @0 (the island's `EquipWeapon(BL, 0x58, 0)` at 001a8494),
//! so 0700000b must boot owning it.
//!
//! Opt-in via `NF_ISO`.

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

// ---------------------------------------------------------------------------
// The disc-row decoder: /ACTION.ELF bytes → per-level EquipWeapon rows
// ---------------------------------------------------------------------------

const EQUIP: u32 = 0x1a8a58; // Player_EquipWeapon
const INIT_AMMO: u32 = 0x1a17f8; // Player_InitAmmoWeapons
const UPG: u32 = 0x1a8f30; // Upgrade_Weapon
const TABLE: u32 = 0x2fc870; // the 74-word switch table [M 001a8270-001a8284]
const DISPATCH: u32 = 0x1a8254; // the dispatch's first instruction (the prologue stops here)
const TAIL: u32 = 0x1a8994; // the shared tail label [M 001a86b4's b target]
const DEFAULT_ARM: u32 = 0x1a86f0; // the dispatch's out-of-range target [M 001a826c beq]
/// The shared tail's own equips [M 001a7ef8:318-319]: Upgrade_Weapon(0x4e) @999 and the fists @0.
const TAIL_ROWS: [(u32, u32); 2] = [(0x4e, 999), (1, 0)];

struct Elf {
    data: Vec<u8>,
    segs: Vec<(u32, u32, u32)>, // (vaddr, file offset, len)
}

impl Elf {
    fn new(data: Vec<u8>) -> Option<Elf> {
        let phoff = u32::from_le_bytes(data.get(0x1c..0x20)?.try_into().ok()?) as usize;
        let phentsize = u16::from_le_bytes(data.get(0x2a..0x2c)?.try_into().ok()?) as usize;
        let phnum = u16::from_le_bytes(data.get(0x2c..0x2e)?.try_into().ok()?) as usize;
        let mut segs = Vec::new();
        for i in 0..phnum {
            let o = phoff + i * phentsize;
            let p_type = u32::from_le_bytes(data.get(o..o + 4)?.try_into().ok()?);
            if p_type != 1 { continue; }
            let p_offset = u32::from_le_bytes(data.get(o + 4..o + 8)?.try_into().ok()?);
            let p_vaddr = u32::from_le_bytes(data.get(o + 8..o + 12)?.try_into().ok()?);
            let p_filesz = u32::from_le_bytes(data.get(o + 16..o + 20)?.try_into().ok()?);
            segs.push((p_vaddr, p_offset, p_filesz));
        }
        Some(Elf { data, segs })
    }
    fn word(&self, a: u32) -> u32 {
        for &(v, o, sz) in &self.segs {
            if v <= a && a + 4 <= v + sz {
                let i = (o + (a - v)) as usize;
                return u32::from_le_bytes(self.data[i..i + 4].try_into().unwrap());
            }
        }
        panic!("ELF word read at unmapped {a:#x}");
    }
    /// `Upgrade_Weapon` (0x1a8f30) at the boot `Upgrades` = 0 [L ee_start_05.bin: 0x2beea8 = 0]:
    /// the tables' slot 0, read straight from the ELF [D 0x2bee08..]; case 0xa maps to 0xc whenever
    /// the game clock is running (GameState[0x52] != 0 — always in play) [M 0x1a8f30 case 0xa].
    fn up(&self, id: u32) -> u32 {
        match id {
            6 => self.word(0x2bee08),
            0x1e => self.word(0x2bee28),
            0x24 => self.word(0x2bee38),
            0x43 => self.word(0x2bee68),
            0x4a => self.word(0x2bee48),
            0x4e => self.word(0x2bee18),
            0x56 => self.word(0x2bee58),
            0xa => 0xc,
            _ => id,
        }
    }
}

#[derive(Clone)]
struct St {
    r: [Option<u32>; 32],
    stk: std::collections::HashMap<u32, Option<u32>>,
}
impl St {
    fn new() -> St {
        let mut r = [None; 32];
        r[0] = Some(0); // $zero
        St { r, stk: Default::default() }
    }
}

fn apply(i: u32, st: &mut St, held: &mut Vec<(u32, u32)>, pc: u32) {
    let op = i >> 26;
    let rs = ((i >> 21) & 31) as usize;
    let rt = ((i >> 16) & 31) as usize;
    let rd = ((i >> 11) & 31) as usize;
    let imm = i & 0xffff;
    let simm = (imm as u16 as i16) as i32 as u32; // sign-extended, then as bits
    let fn_ = i & 0x3f;
    match op {
        9 => { if rs != 0 { st.r[rt] = None; } else { st.r[rt] = Some(simm); } } // addiu/li
        0xd => st.r[rt] = if rs == 0 { Some(imm) } else { st.r[rs].map(|v| v | imm) }, // ori
        0xf => st.r[rt] = Some(imm << 16), // lui
        0 => match fn_ {
            0 => st.r[rd] = st.r[rt], // sll (the sll a1, sN, 16 / sra a1, a1, 16 sign-extend pair)
            0x21 | 0x2d => { // addu/daddu — the compiler's move
                st.r[rd] = if rs == 0 { st.r[rt] } else if rt == 0 { st.r[rs] } else { None };
            }
            3 => st.r[rd] = st.r[rt], // sra (the pair's second half; values stay < 0x8000)
            0xa => { if st.r[rt] == Some(0) { st.r[rd] = st.r[rs]; } } // movz (the 0x59/0x5b pick)
            _ => {}
        },
        0x28 => { // sb
            let off = simm as u16 as i32;
            if off == 0x63 {
                if let Some(v) = st.r[rt] { held.push((pc, v)); } // the held weapon, M_CONTROL+0x63
            }
            if rs == 29 { st.stk.insert(simm, st.r[rt]); }
        }
        0x20 | 0x24 | 0x23 => st.r[rt] = if rs == 29 { st.stk.get(&simm).copied().flatten() } else { None }, // lb/lbu/lw
        0x2b => { if rs == 29 { st.stk.insert(simm, st.r[rt]); } } // sw (the 0(sp) upgrade spill)
        0xb | 0xa => st.r[rt] = None, // sltiu/slti — the comparison result is not tracked
        _ => {}
    }
}

/// The abstract walk: collects the `Player_EquipWeapon` rows on every explored path (the cold path
/// carries the kit; the warm path jumps straight to the tail), the held-weapon writes (`sb ?, 0x63`),
/// and the `Player_InitAmmoWeapons` wipes. Branches explore both arms — the owned-gate's fall-through
/// IS the cold kit [M 001a8610's bne skips to the tail when the P2K is already owned].
fn walk(
    elf: &Elf,
    mut pc: u32,
    st: &mut St,
    rows: &mut Vec<(u32, u32)>,
    held: &mut Vec<(u32, u32)>,
    wipe: &mut bool,
    seen: &mut std::collections::HashSet<u32>,
    stop: u32,
    ends: &mut Vec<(u32, St)>,
) {
    loop {
        if pc >= stop { ends.push((pc, st.clone())); return; }
        if !(0x1a7000..0x1a8b00).contains(&pc) || seen.contains(&pc) { return; }
        seen.insert(pc);
        let i = elf.word(pc);
        let op = i >> 26;
        let rs = ((i >> 21) & 31) as usize;
        let rt = ((i >> 16) & 31) as usize;
        let fn_ = i & 0x3f;
        let imm = i & 0xffff;
        let simm = (imm as u16 as i16) as i32;
        let jt = (pc + 4) & 0xf000_0000 | ((i & 0x03ff_ffff) << 2); // j/jal absolute
        let bt = (pc as i64 + 4 + (simm as i64 * 4)) as u32; // branch pc-relative
        if op == 3 {
            apply(elf.word(pc + 4), st, held, pc + 4); // the delay slot first
            let arg = st.r[4];
            if jt == EQUIP {
                if let (Some(a1), Some(a2)) = (st.r[5], st.r[6]) { rows.push((a1, a2)); }
            } else if jt == INIT_AMMO {
                *wipe = true;
            } else if jt == UPG {
                st.r[2] = arg.map(|a| elf.up(a));
            } else {
                st.r[2] = None;
            }
            pc += 8;
            continue;
        }
        if op == 2 {
            apply(elf.word(pc + 4), st, held, pc + 4);
            pc = jt;
            continue;
        }
        if op == 4 && rs == 0 && rt == 0 {
            apply(elf.word(pc + 4), st, held, pc + 4);
            pc = bt;
            continue;
        }
        if op == 4 || op == 5 || op == 1 {
            apply(elf.word(pc + 4), st, held, pc + 4); // the delay slot: both paths see it
            let mut s2 = st.clone();
            walk(elf, bt, &mut s2, rows, held, wipe, seen, stop, ends);
            pc += 8;
            continue;
        }
        if op == 0 && fn_ == 8 { return; } // jr (the dispatch's table jump)
        apply(i, st, held, pc);
        pc += 4;
    }
}

/// The prologue's s-register map, captured at the dispatch entry.
fn prologue(elf: &Elf) -> St {
    let mut rows = Vec::new();
    let mut held = Vec::new();
    let mut wipe = false;
    let mut ends = Vec::new();
    walk(elf, 0x1a7ef8, &mut St::new(), &mut rows, &mut held, &mut wipe,
         &mut Default::default(), DISPATCH, &mut ends);
    ends.into_iter()
        .find(|(_, s)| s.r[16..24].iter().any(|v| v.is_some()))
        .map(|(_, s)| s)
        .expect("the prologue's dispatch-entry state (s0-s7 set by the Upgrade_Weapon block)")
}

struct Decoded {
    rows: Vec<(u32, u32)>,
    held: Option<u32>,
    wipe: bool,
}

/// One level's disc rows: the arm's EquipWeapon immediates + the shared tail rows [M 001a7ef8:318-319].
fn decode_level(elf: &Elf, pro: &St, level_id: u32) -> Decoded {
    let code = level_id & 0xff;
    let arm = if code == 0 || code > 0x4a {
        DEFAULT_ARM // out of the jump table's range: the dispatch's beq target [M 001a826c]
    } else {
        let k = code - 1;
        let a = elf.word(TABLE + 4 * k);
        if !(0x1a7ef8..0x1a8a60).contains(&a) { DEFAULT_ARM } else { a }
    };
    let mut rows = Vec::new();
    let mut held_writes = Vec::new();
    let mut wipe = false;
    let mut ends = Vec::new();
    let mut st = pro.clone();
    walk(elf, arm, &mut st, &mut rows, &mut held_writes, &mut wipe,
         &mut Default::default(), TAIL, &mut ends);
    rows.extend_from_slice(&TAIL_ROWS);
    Decoded { rows, held: held_writes.last().map(|&(_, v)| v), wipe }
}

/// The on-foot archive list, straight from FILES.BIN's index (the same set the weapon matrix
/// sweeps — `nf_view::matrix::on_foot_levels`).
fn levels(iso: &str) -> Vec<String> {
    let img = nf_formats::iso9660::IsoImage::open(iso).expect("ISO open");
    nf_view::matrix::on_foot_levels(&img)
}

/// Every on-foot level boots with the disc's loadout rows: the owned set and the switch request
/// from the real `nf_view::load` path, compared against the rows decoded from /ACTION.ELF.
#[test]
fn every_on_foot_level_boots_with_the_discs_loadout_rows() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let img = nf_formats::iso9660::IsoImage::open(&iso).expect("ISO open");
    let action = img.find("/ACTION.ELF").expect("no /ACTION.ELF on the disc");
    let elf = Elf::new(img.read_file(action).expect("read /ACTION.ELF")).expect("ELF parse");
    // The dispatch bytes this decode is built on [M 001a825c-001a8284] — a build drift fails here
    // instead of producing nonsense rows.
    assert_eq!(elf.word(0x1a825c), 0x8c64_3774, "lw a0, 0x3774(v1) — the level-id load");
    assert_eq!(elf.word(0x1a8268), 0x2c83_004a, "sltiu v1, a0, 0x4a — the jump-table bound");
    assert_eq!(elf.word(0x1a8284), 0x0080_0008, "jr a0 — the table dispatch");
    let pro = prologue(&elf);
    assert_eq!(pro.r[16], Some(2), "s0 = Upgrade_Weapon(6) = 2, the P2K [M 001a81d4; D 0x2bee08]");
    assert_eq!(pro.r[20], Some(0x5b), "s4 = 0x5b, the 60 Hz NTSC pick [M 001a81c8 + 001a8250]");
    assert_eq!(pro.stk.get(&0).copied().flatten(), Some(0x4e),
        "0(sp) = Upgrade_Weapon(0x4e), the tail's laser [M 001a81d8 + 0x1a7ef8:318]");

    let names = levels(&iso);
    let mut checked = 0usize;
    let mut gadget_levels = 0usize;
    for name in &names {
        let level_id = u32::from_str_radix(&name[..8], 16).unwrap_or(0);
        let d = decode_level(&elf, &pro, level_id);
        assert!(d.wipe, "{name}: the cold path must run Player_InitAmmoWeapons first (the disc arm wipes) [M 0x1a7ef8]");
        let Some(mut scene) = nf_view::load(&iso, name, false).ok() else {
            eprintln!("[rows] {name}: not loadable — skipped (not an on-foot sim archive)");
            continue;
        };
        let Some(sim) = scene.sim.as_mut() else {
            eprintln!("[rows] {name}: no sim — skipped");
            continue;
        };
        if sim.mp.is_some() {
            eprintln!("[rows] {name}: an MP map — MP_EquipPlayer 0x1855f8 arms it, not the SP switch — skipped");
            continue;
        }
        let wd = sim.weapons.clone();
        let loaded = |w: u8| sim.skins.contains_key(&wd.first_person(w as usize));
        // The rows folded through the game's own grant rule: `Player_EquipWeapon` 0x1a8a58:20 refuses
        // an equip whose first-person model is not in the hashtable; the equip owns the record's BASE
        // (+0x02) [M 0x1a8a58:48-55]; then `Player_CheckWeaponsLoaded` 0x1a97c8 revokes a selectable
        // weapon whose skin is missing.
        let mut expected = vec![false; sim.drones.inventory.owned.len()];
        for &(id, _rounds) in &d.rows {
            let (fp, base) = (wd.first_person(id as usize), wd.base(id as usize) as usize);
            if fp != 0 && loaded(id as u8) && base < expected.len() { expected[base] = true; }
        }
        for id in 0..expected.len().min(0x72) {
            if expected[id] && !loaded(id as u8) && wd.selectable(id as u8) { expected[id] = false; }
        }
        let owned: Vec<usize> = (0..sim.drones.inventory.owned.len())
            .filter(|&w| sim.drones.inventory.owned[w]).collect();
        let expected_set: Vec<usize> = (0..expected.len()).filter(|&w| expected[w]).collect();
        let rows_txt = d.rows.iter().map(|(a, b)| format!("{a:#04x}@{b}")).collect::<Vec<_>>().join(" ");
        eprintln!("[rows] {name}: disc rows [{rows_txt}] held {} | boot owned {owned:?} req {}",
            d.held.map(|v| format!("{v:#04x}")).unwrap_or_default(), sim.gun.switch_request);
        assert_eq!(owned, expected_set,
            "{name}: the boot owned set vs the disc rows (refusals applied) [M 0x1a7ef8's arm for this level; 0x1a8a58:20; 0x1a97c8]");
        // The request: the arm's held byte, then the tail's fists fallback when its base is unowned
        // [M 0x1a7ef8:321-324, disasm 001a89c4-001a89fc].
        let mut req = d.held.unwrap_or(2) as usize; // no write → the tail's s0 default [M 001a8990]
        if !expected.get(wd.base(req) as usize).copied().unwrap_or(false) { req = 1; }
        assert_eq!(sim.gun.switch_request, req as u8,
            "{name}: the boot switch request vs the disc arm's held byte (the fists fallback applied) [M 001a8990 + 001a89c4-001a89fc]");
        checked += 1;
        if d.rows.iter().any(|&(id, _)| id >= 0x48) { gadget_levels += 1; }
        drop(scene);
    }
    eprintln!("[rows] {checked} levels pinned ({gadget_levels} with gadget rows); archives {names:?}");
    assert!(checked >= 15, "the on-foot sweep must reach the story levels (checked {checked})");
}

/// The sister: the port's `init_level_loadout` — called directly per level with the skin refusal
/// disabled (`loaded = |_| true`, the raw kit list, the dev panel's levelkit call) — must carry EVERY
/// disc gadget row (id ≥ 0x48). When the port's loadout drops one, this fails naming the level and
/// the gadget. The headline: 0700000b's disc kit equips the Q-Worm 0x58 @0 (the island arm's
/// `EquipWeapon(BL, 0x58, 0)` at 001a8494), so it must be in the port's 0b kit.
#[test]
fn the_discs_gadget_rows_survive_into_the_ports_kit() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let img = nf_formats::iso9660::IsoImage::open(&iso).expect("ISO open");
    let action = img.find("/ACTION.ELF").expect("no /ACTION.ELF on the disc");
    let elf_bytes = img.read_file(action).expect("read /ACTION.ELF");
    let elf = Elf::new(elf_bytes.clone()).expect("ELF parse");
    let pro = prologue(&elf);
    let wd = nf_formats::weapon::WeaponData::from_action_elf(&elf_bytes).expect("weapon_data");
    let names = levels(&iso);
    let mut checked = 0usize;
    for name in &names {
        let level_id = u32::from_str_radix(&name[..8], 16).unwrap_or(0);
        let dec = decode_level(&elf, &pro, level_id);
        let mut health = nf_game::bullet::PlayerHealth::default();
        let mut inv = nf_game::pickup::Inventory::empty(0x72, 34);
        let mut ch = nf_game::triggers::Channels::default();
        let loaded = |_: u8| true; // the raw list: the refusal is test 1's pin
        let _req = nf_game::player::init_level_loadout(level_id, None, &mut health, &mut inv, &mut ch, &wd, loaded);
        let rows_txt = dec.rows.iter().map(|(a, b)| format!("{a:#04x}@{b}")).collect::<Vec<_>>().join(" ");
        eprintln!("[sister] {name}: kit rows [{rows_txt}] held {:?} wipe {}", dec.held.map(|v| format!("{v:#04x}")).unwrap_or_default(), dec.wipe);
        let mut gadget_rows = 0usize;
        for &(id, rounds) in &dec.rows {
            if id < 0x48 { continue; } // the gadget families [S the sim's own ≥ 0x48 convention]
            gadget_rows += 1;
            let base = wd.base(id as usize) as usize;
            assert!(inv.owned[base],
                "{name}: the disc's gadget row {id:#04x} ({rounds} rounds) is NOT in the port's kit — \
                 the loadout dropped a disc row [M 0x1a7ef8's arm for this level]");
        }
        if level_id == 0x0700_0009 || level_id == 0x0700_000a || level_id == 0x0700_000b {
            assert!(inv.owned[0x58],
                "{name}: the Q-Worm (0x58) must be in the port's kit — the disc's island arm equips it \
                 at 001a8494 [M 0x1a7ef8 cases 0x7000009/0a/0b]");
        }
        if gadget_rows > 0 {
            checked += 1;
            eprintln!("[sister] {name}: {gadget_rows} disc gadget rows all present");
        }
    }
    eprintln!("[sister] {checked} levels with gadget rows pinned");
    assert!(checked >= 8, "the gadget sweep must reach the kit levels (checked {checked})");
}
