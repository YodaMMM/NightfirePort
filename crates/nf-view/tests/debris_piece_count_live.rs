//! debris-piece-count: the MGL 'Grenade' contact blast on the 07000011 storeroom's metal shelf
//! (the level's MGL pickup spot), through the real fire path.
//!
//! The game's rules [M]:
//! - `Explode_Update` 0x17c350's debris arm: while the blast's age is ≤ lifetime >> 2 (165 >> 2 =
//!   41 for script 0x6000052), one `Rand_Rand(5)` per tick and a burst of 3 pieces
//!   (`Debris_CreateEx(.., 180, 3, .., MetalHash_132, 4)`) when `frame % (r + 10) == 0` — a
//!   random 0..6 bursts, ≈ 3.4 on average. The port already ran that rule; the piece count of one
//!   shot is that roll (pinned below over 25 start ticks: 3..18 pieces, mean ≈ 9.5).
//! - `Explode_Create` 0x17cbd0 stores the spread axis EXPLODEINFO+0x10 = a + b of
//!   `Vec_MakeOrthonormalBasis` 0x1e4b78 over the unit blast→player vector — perpendicular to
//!   it. The port stored align2dir's x row + the to-player row, so the pieces flew toward or away
//!   from the player and faster sideways, never up.
//! - `Debris_Update` 0x1aac00: a piece of life L lives through L updates (the one that stores
//!   life 0 keeps it) and dies on update L + 1 [M disasm 0x1aac0c-0x1aac4c]. The port dropped it
//!   one update early.
//!
//! Opt-in: `NF_ISO`. NF_SILENT everywhere.

use nf_game::player::Input;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

struct Shot {
    blast_pos: [f32; 3],
    bond_pos: [f32; 3],
    jitter: [f32; 3],
    /// Per burst: (blast age, pieces added).
    bursts: Vec<(usize, usize)>,
    /// Per piece of the first burst: (life at create, ticks it stayed in the list after create).
    first_lives: Vec<(i32, usize)>,
}

/// Loads 07000011, idles `30 + wait` ticks, stands on the MGL pickup facing the shelf and fires
/// one 0x2A round; follows the blast and its debris for 260 ticks.
fn shoot(iso: &str, wait: usize) -> Shot {
    let mut scene = nf_view::load_with_carry(iso, "07000011.bin", false, None).expect("load 07000011");
    for _ in 0..(30 + wait) { scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false); }
    // The MGL pickup's spot [D: the level's 0x2A pickup], facing the shelf 5.3 units away.
    let at = scene.sim.as_ref().unwrap().drones.pickups.iter().find(|p| p.weapon == 0x2a)
        .expect("07000011 places an MGL").start;
    scene.sim.as_mut().unwrap().place_bond(at, 0.35);
    let w = 0x2au16;
    {
        let sim = scene.sim.as_mut().unwrap();
        let owned = sim.drones.inventory.owned.len();
        if (w as usize) < owned { sim.drones.inventory.owned[w as usize] = true; }
        sim.drones.inventory.clip[w as usize] = 6;
        sim.gun.weapon = w as u8;
        sim.gun.switch_request = w as u8;
        sim.gun.switch_state = 0;
    }
    for _ in 0..10 { scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false); }
    let sim = scene.sim.as_mut().unwrap();
    sim.gun.reload_state = 0;
    sim.gun.fire_state = false;
    sim.gun.clip = 6;
    sim.gun.pending = 0;
    let mut input = Input::default();
    input.fire = true;
    let mut t0: Option<usize> = None;
    let mut shot = Shot { blast_pos: [0.0; 3], bond_pos: [0.0; 3], jitter: [0.0; 3], bursts: Vec::new(), first_lives: Vec::new() };
    let mut last = 0usize;
    // `blast_debris` appends: the first burst is indices 0..3 on its create tick.
    let mut first_seen: Vec<i32> = Vec::new();
    let mut first_alive = [0usize; 3];
    for t in 0..400 {
        let sim = scene.sim.as_mut().unwrap();
        let n0 = sim.drones.blasts.len();
        sim.step_bond(if t == 0 { input } else { Input::default() }, t == 0, false, false);
        let sim = scene.sim.as_ref().unwrap();
        if t0.is_none() && sim.drones.blasts.len() > n0 {
            let b = sim.drones.blasts.iter().rev().find(|b| b.weapon_type == w).expect("the 0x2A blast");
            shot.blast_pos = b.pos;
            shot.jitter = b.debris_jitter;
            shot.bond_pos = sim.bond.pos;
            t0 = Some(t);
        }
        let Some(a) = t0 else { continue };
        let n = sim.drones.blast_debris.len();
        if n > last { shot.bursts.push((t - a, n - last)); }
        last = n;
        // Track the first burst by its create life (each piece's life counts down by one per tick).
        if first_seen.is_empty() && n >= 3 {
            first_seen = sim.drones.blast_debris[..3].iter().map(|d| d.life).collect();
        }
        if !first_seen.is_empty() {
            let since = shot.bursts[0].0;
            let age = (t - a) - since; // ticks since the first burst's create tick
            for (k, &l0) in first_seen.iter().enumerate() {
                if sim.drones.blast_debris.iter().any(|d| d.life == l0 - age as i32 && d.fade as i32 == 0xff - age as i32) {
                    first_alive[k] = age + 1;
                }
            }
        }
        if t > a + 260 { break; }
    }
    shot.first_lives = first_seen.iter().zip(first_alive).map(|(&l, n)| (l, n)).collect();
    shot
}

fn unit(v: [f32; 3]) -> [f32; 3] {
    let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    [v[0] / l, v[1] / l, v[2] / l]
}

/// The spread axis is `a + b` of `Vec_MakeOrthonormalBasis` over the unit blast→Bond vector —
/// perpendicular to it [M 0x17cbd0, 0x1e4b78]. The old axis (align2dir x row + the to-player row)
/// had a dot of 1 with that vector.
#[test]
fn mgl_shelf_blast_spreads_its_pieces_across_the_view() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    std::env::set_var("NF_SILENT", "1");
    let s = shoot(&iso, 0);
    let n = unit([s.bond_pos[0] - s.blast_pos[0], s.bond_pos[1] - s.blast_pos[1], s.bond_pos[2] - s.blast_pos[2]]);
    let j = s.jitter;
    let dot = j[0] * n[0] + j[1] * n[1] + j[2] * n[2];
    eprintln!("blast {:?} bond {:?} to-player {n:?} jitter {j:?} dot {dot}", s.blast_pos, s.bond_pos);
    assert!(dot.abs() < 1e-3, "the spread axis is perpendicular to the to-player vector [M 0x1e4b78], dot {dot}");
    let want = nf_game::explode::debris_jitter(n);
    for k in 0..3 {
        assert!((j[k] - want[k]).abs() < 1e-3, "EXPLODEINFO+0x10 = a + b [M 0x17cbd0]: {j:?} vs {want:?}");
    }
    // The shelf faces Bond horizontally, so b is near world up: the pieces spread vertically.
    assert!(j[1] > 0.9, "b ≈ up for a horizontal to-player vector [M 0x1e4b78 second branch], {j:?}");
}

/// A piece of life L stays in the list L + 1 ticks after its create tick (drawn on the create
/// tick and through the update that stores life 0) [M 0x1aac0c-0x1aac4c].
#[test]
fn mgl_shelf_pieces_live_their_full_count() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    std::env::set_var("NF_SILENT", "1");
    let s = shoot(&iso, 0);
    eprintln!("first burst lives {:?}", s.first_lives);
    assert_eq!(s.first_lives.len(), 3, "a burst is 3 pieces [M 0x17c350:63-67]");
    for &(l, ticks) in &s.first_lives {
        assert!((90..180).contains(&l), "life = Rand(90) + 90 [M 0x1ac808:29-31], got {l}");
        assert_eq!(ticks, l as usize + 1, "life {l}: in the list on the create tick and {l} updates [M 0x1aac00]");
    }
}

/// The count is the game's random burst gate: 3 pieces per burst, bursts only at ages 1..=41,
/// one shot ranging 3..18 pieces over 25 start ticks (mean ≈ 9.5, the rule's expectation ≈ 10.3).
/// The 6-piece shot is wait 0's roll; other start ticks give 12 and 15 (as the game's runs gave
/// 6, 9 and 12). Pins the rule, not one roll.
#[test]
fn mgl_shelf_piece_count_follows_the_burst_gate() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    std::env::set_var("NF_SILENT", "1");
    let mut total = 0usize;
    let mut counts = Vec::new();
    for wait in [0usize, 2, 5, 10, 23] {
        let s = shoot(&iso, wait);
        let pieces: usize = s.bursts.iter().map(|b| b.1).sum();
        eprintln!("wait {wait}: bursts {:?} → {pieces} pieces", s.bursts);
        for &(age, k) in &s.bursts {
            assert_eq!(k, 3, "Debris_CreateEx count 3 [M 0x17c350:63-67]");
            assert!((1..=41).contains(&age), "bursts only while age ≤ 165 >> 2 [M 0x17c350:31], age {age}");
        }
        total += pieces;
        counts.push(pieces);
    }
    assert!(counts.iter().any(|&c| c >= 12), "some start ticks throw 12+ pieces: {counts:?}");
    assert!(total >= 30, "five shots average ≥ 6 pieces: {counts:?}");
}
