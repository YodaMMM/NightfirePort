//! `UpdateMusicalEvents` 0x114e70's own writes to the attack gate (`MusicVars` 0x2707a0, the
//! number of attacking guards that starts the combat music), on the real levels:
//! - e[10], the music zones with channel 10 (on 01, 06, 07 and 12), sets the count to 1 every
//!   tick it is posted [M 0x114e70:125-127];
//! - e[2] == 2, the alarm slot, seats the table's -99 row (count 1, window 10, distance 50 [D
//!   MapDroneData]) [M :130-152];
//! - while the bonus count is negative, and when the alert count runs out, the level's own row
//!   goes back; a level with no row takes count 2, floor 0, window 8, distance 25 [M :102-123,
//!   :166-188].
//! Headless there is no IOP, so `Run::tick` reports back the section last requested. The guards
//! are held in a combat state, not seeing Bond, so only the count decides. Opt-in: `NF_ISO`.

use nf_game::music::Cmd;
use nf_view::scene::Scene;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

struct Run { scene: Scene, section: i32 }

impl Run {
    fn load(level: &str) -> Option<Run> {
        let path = iso()?;
        Some(Run { scene: nf_view::load(&path, level, false).expect("load"), section: 0 })
    }

    fn sim(&mut self) -> &mut nf_view::scene::Sim { self.scene.sim.as_mut().unwrap() }

    /// One tick; the section jumps it queued.
    fn tick(&mut self) -> Vec<i32> {
        let section = self.section;
        let sim = self.sim();
        sim.drones.iop_section = section;
        sim.step_bond(Default::default(), false, false, false);
        let mut got = Vec::new();
        for c in sim.drones.music.out.drain(..).collect::<Vec<_>>() {
            match c {
                Cmd::Start { section, .. } => self.section = section,
                Cmd::Jump { section, .. } => { self.section = section; got.push(section); }
                Cmd::FadeDown { .. } => {}
            }
        }
        got
    }

    fn place(&mut self, pos: [f32; 3]) {
        let sim = self.sim();
        sim.bond.pos = pos;
        sim.bond.prev_pos = pos;
    }

    /// The first `n` live guards.
    fn guards(&mut self, n: usize) -> Vec<usize> {
        let sim = self.sim();
        let g: Vec<usize> = sim.drones.list.iter().enumerate()
            .filter(|(_, d)| d.present && d.enabled && d.team == 1 && !d.corpse).map(|(i, _)| i).take(n).collect();
        assert_eq!(g.len(), n);
        g
    }

    /// `ticks` ticks with `guards` held attacking: in a combat state, not seeing Bond, last seen
    /// a tick ago. Returns the jumps and whether the gate ever posted `Music_Event(2, 5)`.
    fn attack(&mut self, guards: &[usize], ticks: u32) -> (Vec<i32>, bool) {
        let (mut got, mut fired) = (Vec::new(), false);
        for _ in 0..ticks {
            for &k in guards {
                let d = &mut self.sim().drones.list[k];
                d.state = nf_game::drone::state::COMBAT;
                d.seen = false;
                d.unseen_ticks = 0;
            }
            got.extend(self.tick());
            let sim = self.sim();
            assert!(guards.iter().all(|&k| !sim.drones.list[k].seen), "a held guard saw Bond");
            fired |= sim.drones.prev_event2 == 5;
        }
        (got, fired)
    }

    fn vars(&mut self) -> (u16, u16, u32, f32) {
        let v = self.sim().drones.attack_gate.vars.expect("vars");
        (v.count, v.floor, v.window, v.distance)
    }
}

/// 07000006's combat sections [D MapMusic].
const COMBAT_06: [i32; 3] = [2, 15, 17];

/// 07000006 has no `MapDroneData` row, so two attacking guards start the combat music. Inside
/// the channel-10 zone (the cel at x 27.9..37.5, z -25..-16.25) one is enough: e[10] sets the
/// count to 1, the gate posts `Music_Event(2, 5)`, and after the level's 10 alert ticks the
/// alarm turns on and `UpdateMFX_01CastleB` 0x113060 picks a combat section. The alert count
/// spent re-seats the default row: count 2, floor 0, window 8, distance 25 [M 0x114e70:185-188].
#[test]
fn in_06s_channel_10_zone_one_guard_starts_the_combat_music() {
    let Some(mut r) = Run::load("07000006.bin") else { eprintln!("NF_ISO not set — skipping"); return };
    for _ in 0..40 { r.tick(); }
    assert_eq!(r.vars(), (2, 0, 8, 25.0), "no row: the default");
    let g = r.guards(1);
    r.place([32.7, 40.5, -20.6]);
    let (got, fired) = r.attack(&g, 30);
    assert!(fired, "one attacker in the zone posts e[2] = 5");
    assert!(got.iter().any(|s| COMBAT_06.contains(s)), "combat music: {got:?}");
    assert_eq!(r.vars(), (2, 0, 8, 25.0), "the alert count spent: the default row back");
}

/// Sister: where Bond starts, outside the zone, one attacking guard is not enough; a second
/// one starts the combat music.
#[test]
fn outside_06s_zone_it_takes_two_guards() {
    let Some(mut r) = Run::load("07000006.bin") else { eprintln!("NF_ISO not set — skipping"); return };
    for _ in 0..40 { r.tick(); }
    let g = r.guards(2);
    let (got, fired) = r.attack(&g[..1], 60);
    assert!(!fired && got.iter().all(|s| !COMBAT_06.contains(s)), "one guard: {got:?}");
    let (got, fired) = r.attack(&g, 30);
    assert!(fired, "two attackers post e[2] = 5");
    assert!(got.iter().any(|s| COMBAT_06.contains(s)), "combat music: {got:?}");
}

/// 07000003: the alarm slot reading 2 seats the -99 row, count 1, window 10, distance 50; the
/// alarm's 600 bonus ticks spent, the level's row (count 2, window 8, distance 10) goes back.
#[test]
fn the_03_alarm_seats_the_minus_99_row_and_its_end_restores_the_level_row() {
    let Some(mut r) = Run::load("07000003.bin") else { eprintln!("NF_ISO not set — skipping"); return };
    for _ in 0..40 { r.tick(); }
    assert_eq!(r.vars(), (2, 0, 8, 10.0), "03's row [D]");
    r.sim().drones.music.event(2, 2);
    r.tick();
    assert_eq!(r.vars(), (1, 0, 10, 50.0), "the -99 row");
    for _ in 0..590 { r.tick(); }
    assert_eq!(r.vars(), (1, 0, 10, 50.0), "held while the bonus counts down");
    for _ in 0..20 { r.tick(); }
    assert_eq!(r.vars(), (2, 0, 8, 10.0), "the level's row back");
}
