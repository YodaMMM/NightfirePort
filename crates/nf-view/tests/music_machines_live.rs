//! The per-level music machines on the real levels: `UpdateMFX_03TowerA` 0x113558 (07000009),
//! `UpdateMFX_03TowerB` 0x1137b0 (0700000a), `UpdateMFX_PowerStationA1` 0x113c18 (0700000c) and
//! `UpdateMFX_PowerStationA2` 0x113fc0 (0700000d). The events come through the real producers:
//! the cel music zones [M Player_Update 0x193600:117-122], the MusicTrigger placements [M
//! 0x1b31a0] and the intro's own script. Headless there is no IOP, so `Run::tick` stands in for
//! its latch: the section last requested is the section reported back [M music.md §3.5].
//! Opt-in: `NF_ISO`.

use nf_game::music::Cmd;
use nf_view::scene::Scene;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

struct Run { scene: Scene, section: i32 }

impl Run {
    fn load(level: &str) -> Option<Run> {
        let path = iso()?;
        Some(Run { scene: nf_view::load(&path, level, false).expect("load"), section: 0 })
    }

    /// One tick; the section jumps it queued.
    fn tick(&mut self) -> Vec<(i32, bool)> {
        let sim = self.scene.sim.as_mut().unwrap();
        sim.drones.iop_section = self.section;
        sim.step_bond(Default::default(), false, false, false);
        let mut got = Vec::new();
        for c in sim.drones.music.out.drain(..) {
            match c {
                Cmd::Start { section, .. } => self.section = section,
                Cmd::Jump { section, instant } => { self.section = section; got.push((section, instant)); }
                Cmd::FadeDown { .. } => {}
            }
        }
        got
    }

    fn ticks(&mut self, n: u32) -> Vec<(i32, bool)> { (0..n).flat_map(|_| self.tick()).collect() }

    /// Runs the level's opening `n` ticks, skipping any cut-scene the way `Script_FFwd` does.
    fn skip_intro(&mut self, n: u32) -> Vec<(i32, bool)> {
        let mut got = Vec::new();
        for _ in 0..n {
            let sim = self.scene.sim.as_mut().unwrap();
            if sim.drones.scripts.cutscene_playing() { sim.drones.scripts.skip_request = true; }
            got.extend(self.tick());
        }
        got
    }

    fn place(&mut self, pos: [f32; 3]) {
        let sim = self.scene.sim.as_mut().unwrap();
        sim.bond.pos = pos;
        sim.bond.prev_pos = pos;
    }

    /// `n` ticks of the alarm slot the attack gate and the music triggers write.
    fn alarm(&mut self, n: u32) -> Vec<(i32, bool)> {
        (0..n).flat_map(|_| { self.scene.sim.as_mut().unwrap().drones.music.event(2, 2); self.tick() }).collect()
    }
}

/// The rows music.rs's tests mirror, off the disc.
#[test]
fn the_rows_the_machines_mirror_come_from_the_disc() {
    let Some(path) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let img = nf_formats::iso9660::IsoImage::open(&path).expect("open iso");
    let elf = img.read_file(img.find("/ACTION.ELF").expect("no ACTION.ELF")).expect("read elf");
    let (mm, _) = nf_formats::music::tables(&elf).expect("tables parse");
    let row = |lv: u32| mm.iter().find(|m| m.level == lv).expect("row").clone();
    let a = row(0x0700_0009);
    assert_eq!((a.track, a.start, a.combat, a.stealth), (23, 0, vec![10, 8, 6], vec![12, 2]));
    let b = row(0x0700_000a);
    assert_eq!((b.track, b.start, b.combat, b.stealth), (23, 26, vec![8, 6, 10], vec![19, 14]));
    let c = row(0x0700_000c);
    assert_eq!((c.track, c.start, c.combat, c.stealth), (36, 0, vec![7, 9, 10], vec![0, 15, 19, 0, 0]));
    let d = row(0x0700_000d);
    assert_eq!((d.track, d.start, d.combat, d.stealth), (36, 32, vec![41, 43], vec![]));
}

/// 07000009: the upper floor's zone 43 moves the score to section 2, and leaving it moves on
/// to 4 [M 0x113558:83-126]. An alarm takes the level's first combat section, and the calm after
/// it the 0x10 bridge. Sister: where Bond starts, outside the zone, nothing moves.
#[test]
fn the_09_zone_and_alarm_move_the_sections() {
    let Some(mut r) = Run::load("07000009.bin") else { eprintln!("NF_ISO not set — skipping"); return };
    r.ticks(40);
    assert!(r.ticks(40).is_empty(), "outside the zone the score stays on the start section");
    let start = r.scene.sim.as_ref().unwrap().bond.pos;
    r.place([0.0, -369.5, 29.0]);
    let into = r.ticks(30);
    assert_eq!(into.first(), Some(&(2, false)), "zone 43");
    r.place(start);
    assert_eq!(r.ticks(30).first(), Some(&(4, false)), "out of the zone");
    assert_eq!(r.alarm(30).first(), Some(&(10, false)), "the alarm");
    assert_eq!(r.ticks(1000).first(), Some(&(0x10, false)), "the calm after it");
}

/// 0700000a: zone 40 (the cel path) latches the second half of the machine [M 0x1137b0:84-129].
/// Sister: the alarm's calm leaves through 0x1c before the latch and 0xe after it.
#[test]
fn the_0a_zone_latch_changes_the_calm_section() {
    let Some(mut r) = Run::load("0700000a.bin") else { eprintln!("NF_ISO not set — skipping"); return };
    r.ticks(40);
    assert!(r.ticks(40).is_empty(), "the start section stands");
    assert_eq!(r.alarm(30).first(), Some(&(8, false)), "the alarm");
    assert_eq!(r.ticks(1000).first(), Some(&(0x1c, false)), "the calm before the latch");
    r.place([0.0, 1.0, -15.0]);
    assert_eq!(r.ticks(30).first(), Some(&(0x13, false)), "zone 40");
    assert_eq!(r.alarm(30).first(), Some(&(6, false)), "the alarm's next pick");
    assert_eq!(r.ticks(1000).first(), Some(&(0xe, false)), "the calm after the latch");
}

/// 0700000a's other e[40] source: the ch 48 music trigger [D placement] latches the same way.
#[test]
fn the_0a_channel_48_trigger_latches_too() {
    let Some(mut r) = Run::load("0700000a.bin") else { eprintln!("NF_ISO not set — skipping"); return };
    r.ticks(40);
    r.scene.sim.as_mut().unwrap().drones.set_channel(48);
    assert_eq!(r.ticks(30).first(), Some(&(0x13, false)), "the trigger's event 40");
}

/// 0700000c: the yard volume #501 raises ch 0x24, the music trigger posts e[13] and the score
/// goes to 0x22 [M 0x113c18:183-208]; the turbine hall's zone 45 ends the machine with the
/// instant 0x24 [M :87-91]. Sister: after that, the alarm moves nothing.
#[test]
fn the_0c_yard_trigger_and_turbine_zone() {
    let Some(mut r) = Run::load("0700000c.bin") else { eprintln!("NF_ISO not set — skipping"); return };
    r.ticks(40);
    assert!(r.ticks(40).is_empty(), "section 0 is in the stealth set");
    r.place([4.99, 13.27, -222.5]);
    assert_eq!(r.ticks(20).first(), Some(&(0x22, false)), "the yard volume");
    r.place([60.0, 1.0, -222.0]);
    let hall = r.ticks(60);
    assert!(hall.contains(&(0x24, true)), "zone 45: {hall:?}");
    r.ticks(40);
    assert!(r.alarm(60).is_empty(), "the machine has ended");
}

/// 0700000d, the intro played through: its 0x172 sound jumps 0x2b, and the trigger is never
/// cleared [M 0x113fc0:42-47], so zone 45 does not move the score.
#[test]
fn the_0d_intro_sound_holds_section_43() {
    let Some(mut r) = Run::load("0700000d.bin") else { eprintln!("NF_ISO not set — skipping"); return };
    let got = r.ticks(1500);
    assert_eq!(got.first(), Some(&(0x2b, true)), "the cut-scene's sound");
    assert!(r.scene.sim.as_ref().unwrap().drones.music.nis_trigger);
    r.place([170.0, 5.0, -285.0]);
    assert!(r.ticks(800).iter().all(|j| *j == (0x2b, true)), "the arm stands");
}

/// 0700000d, the intro skipped: no 0x172, so the scene's end jumps 0x2b [M 0x113fc0:42] and,
/// the 600-tick hold spent, the calm takes 0xc [M :69-72]; the crane-yard volume #510 posts e[13]
/// [M :74-99] and zone 45 takes 0x3a [M :106-115]. Sister: during the hold nothing else moves.
#[test]
fn the_0d_skipped_intro_runs_the_machine() {
    let Some(mut r) = Run::load("0700000d.bin") else { eprintln!("NF_ISO not set — skipping"); return };
    let got = r.skip_intro(300);
    assert_eq!(got.iter().filter(|j| **j != (0x2b, true)).count(), 0, "only the scene's end: {got:?}");
    assert_eq!(got.first(), Some(&(0x2b, true)));
    assert!(!r.scene.sim.as_ref().unwrap().drones.music.nis_trigger);
    assert_eq!(r.ticks(500).first(), Some(&(0xc, false)), "the hold spent, calm in a combat section");
    r.place([206.18, 13.5, -274.26]);
    assert_eq!(r.ticks(20).first(), Some(&(0x22, false)), "the crane-yard volume");
    r.ticks(40);
    r.place([170.0, 5.0, -285.0]);
    let got = r.ticks(30);
    assert!(got.first().is_some_and(|j| j.0 == 0x3a), "zone 45: {got:?}");
}
