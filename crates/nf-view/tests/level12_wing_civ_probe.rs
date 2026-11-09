//! level-12-wing-guard-civilian probe: the east wing symptom, reproduced through real input with a
//! whole-run ledger. The route is common/tower2.rs's walk_12 (the proven front half) then the exit
//! tail's east-wing legs driven by the same steering walk_to uses, with every tick observed.
//! No position writes; the walker's top-up keeps Bond alive (the fights still run).
//!
//! The ledger logs: every guard shot (shooter placement, aim point), every bullet's owner while in
//! flight, every drone death (placement, tick), the wing guard placement 9's sight/firing fold, the
//! office civilian's (death channel 0x63) position/health, and the mission fail label
//! (0x0400_0034 "Killed Civilians"). Runs the campaign difficulty (1) and the walkreal difficulty
//! (3) sisters.

#[path = "common/walker_b2.rs"]
mod walker;
#[path = "common/tower2.rs"]
mod tower2;

use walker::{dxz, wrap_pi, Walker};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn tables(iso: &str) -> Option<nf_formats::drone::PropertyTables> {
    let img = nf_formats::iso9660::IsoImage::open(iso).expect("open iso");
    let entry = img.find("/ACTION.ELF")?;
    let elf = img.read_file(entry).ok()?;
    nf_formats::drone::PropertyTables::from_action_elf(&elf).ok()
}

struct Ledger {
    shots: Vec<usize>,
    alive: Vec<bool>,
    hp: Vec<f32>,
    out: Vec<String>,
    tables: Option<nf_formats::drone::PropertyTables>,
    /// The wing guard's (placement 9) highest shots count seen — his slots' vecs may be cleared on
    /// death, so the end-of-run read is not the run's evidence.
    max_shots_g9: usize,
    /// The wing guard's prop 0x17, captured the first tick he is live (the props block arms with
    /// the drone's first-tick setup; the load-state read is 0) [L this run].
    p17_g9: Option<u32>,
}

impl Ledger {
    fn new(tables: Option<nf_formats::drone::PropertyTables>) -> Ledger {
        Ledger { shots: Vec::new(), alive: Vec::new(), hp: Vec::new(), out: Vec::new(), tables, max_shots_g9: 0, p17_g9: None }
    }

    fn observe(&mut self, w: &Walker) {
        let sim = w.scene.sim.as_ref().unwrap();
        if self.alive.len() != sim.drones.list.len() {
            self.alive = sim.drones.list.iter().map(|d| d.present && d.health > 0.0 && !d.corpse).collect();
            self.shots = sim.drones.list.iter().map(|d| d.shots.len()).collect();
            self.hp = sim.drones.list.iter().map(|d| d.health).collect();
        }
        let civ = sim.drones.list.iter().position(|d| d.death_channel == 0x63);
        let g9 = sim.drones.list.iter().position(|d| d.placement == 9);
        let mut notes: Vec<String> = Vec::new();
        for (i, d) in sim.drones.list.iter().enumerate() {
            let now_alive = d.present && d.health > 0.0 && !d.corpse;
            if i < self.alive.len() && self.alive[i] && !now_alive {
                notes.push(format!("DEATH idx{i} plc{} tick{}", d.placement, w.total_ticks));
                self.alive[i] = false;
            }
            if i < self.hp.len() && d.health != self.hp[i] {
                notes.push(format!("DMG idx{i} plc{} {:.0}->{:.0}", d.placement, self.hp[i], d.health));
                self.hp[i] = d.health;
            }
            if i < self.shots.len() && d.shots.len() > self.shots[i] {
                for k in self.shots[i]..d.shots.len() {
                    let aim = d.shot_aims.get(k).map(|a| (a.1, a.2));
                    let civ_in_line = civ.map_or(false, |ci| {
                        let c = &sim.drones.list[ci];
                        aim.map_or(false, |(m, t)| dist_point_seg(c.pos, m, t) < 0.6)
                    });
                    notes.push(format!("SHOT idx{i} plc{} tick{} aim{:.1?} civline{}",
                        d.placement, w.total_ticks, aim.map(|a| a.1), civ_in_line as u8));
                }
                self.shots[i] = d.shots.len();
            }
        }
        // Bullets near the civilian (she holds a spot in these runs): owner + distance.
        if let Some(ci) = civ {
            let c = &sim.drones.list[ci];
            if c.present && c.health > 0.0 {
                for b in &sim.drones.bullets {
                    let d = ((b.pos[0] - c.pos[0]).powi(2) + (b.pos[1] - c.pos[1]).powi(2) + (b.pos[2] - c.pos[2]).powi(2)).sqrt();
                    if d < 4.0 {
                        let owner = sim.drones.list.get(b.owner).map_or(999, |o| o.placement);
                        notes.push(format!("NEAR-BUL idx{} plc{} ({:.1},{:.1},{:.1}) d{:.1}",
                            b.owner, owner, b.pos[0], b.pos[1], b.pos[2], d));
                    }
                }
            }
        }
        if let Some(i) = g9 {
            let g = &sim.drones.list[i];
            self.max_shots_g9 = self.max_shots_g9.max(g.shots.len());
            if g.present && g.health > 0.0 {
                let p17 = self.tables.as_ref().map_or(0, |t| g.prop_value(t, 0x17));
                if self.p17_g9.is_none() { self.p17_g9 = Some(p17); }
                notes.push(format!("g9 st{:#x} los{} un{} fire{} ({:.1},{:.1}) p17={p17}",
                    g.state, g.los as u8, g.unseen_ticks, g.firing as u8, g.pos[0], g.pos[2]));
            }
        }
        if let Some(i) = civ {
            let c = &sim.drones.list[i];
            if c.present && c.health > 0.0 && !c.corpse {
                notes.push(format!("civ hp{:.0} ({:.2},{:.2})", c.health, c.pos[0], c.pos[2]));
            } else {
                notes.push(format!("civ DEAD hp{:.0} present{} fail{:?}", c.health, c.present, c.fail_reason.map(|r| r.label)));
            }
        }
        if sim.drones.mission.fail_label != 0 {
            notes.push(format!("FAIL {:#x}", sim.drones.mission.fail_label));
        }
        if !notes.is_empty() {
            self.out.push(format!("t{} {}", w.total_ticks, notes.join(" | ")));
        }
    }
}

/// Distance from point p to segment a-b (the civ-line observation aid, [S]).
fn dist_point_seg(p: [f32; 3], a: [f32; 3], b: [f32; 3]) -> f32 {
    let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let ap = [p[0] - a[0], p[1] - a[1], p[2] - a[2]];
    let ab2 = ab[0] * ab[0] + ab[1] * ab[1] + ab[2] * ab[2];
    let t = if ab2 < 1e-9 { 0.0 } else { (ap[0] * ab[0] + ap[1] * ab[1] + ap[2] * ab[2]) / ab2 };
    let t = t.clamp(0.0, 1.0);
    let c = [a[0] + ab[0] * t, a[1] + ab[1] * t, a[2] + ab[2] * t];
    ((p[0] - c[0]).powi(2) + (p[1] - c[1]).powi(2) + (p[2] - c[2]).powi(2)).sqrt()
}

/// The steering walk (walk_to's core), with the ledger observing every tick.
fn leg(w: &mut Walker, x: f32, z: f32, label: &str, led: &mut Ledger) {
    let start = w.total_ticks;
    loop {
        led.observe(w);
        {
            let sim = w.scene.sim.as_ref().unwrap();
            if sim.disabled { w.tick(Default::default(), false); continue; }
            if sim.drones.level_exit.is_some() {
                eprintln!("[probe] {label}: level_exit at +{} (fail {:#x} restart {})",
                    w.total_ticks - start, sim.drones.mission.fail_label, sim.drones.level_exit_restart);
                return;
            }
        }
        let p = w.scene.sim.as_ref().unwrap().bond.pos;
        let d = dxz(p, [x, p[1], z]);
        if d < 1.0 {
            eprintln!("[probe] {label}: arrived +{}", w.total_ticks - start);
            return;
        }
        if w.total_ticks - start > 4000 {
            panic!("{label}: 4000-tick cap at ({:.2},{:.2})", p[0], p[2]);
        }
        let sim = w.scene.sim.as_ref().unwrap();
        let desired = (x - p[0]).atan2(z - p[2]);
        let err = wrap_pi(desired - sim.bond.yaw);
        let input = nf_game::player::Input {
            turn: err.clamp(-0.07, 0.07),
            forward: if d > 2.2 { 1.0 } else { 0.45 },
            ..Default::default()
        };
        w.tick(input, false);
    }
}

fn run(difficulty: u32) {
    let iso = iso();
    let tables = tables(&iso);
    let mut w = Walker::new(nf_view::load_with_carry_difficulty(&iso, "07000012.bin", false, None, difficulty).unwrap());
    let mut led = Ledger::new(tables.clone());
    {
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("[probe d{difficulty}] the armed roster:");
        for (i, d) in sim.drones.list.iter().enumerate() {
            if d.team == 1 || d.death_channel != 0 {
                eprintln!("  idx{i} plc{} team{} wpn{:#x} death{:#x} ({:.2},{:.2},{:.2})",
                    d.placement, d.team, d.weapon, d.death_channel, d.pos[0], d.pos[1], d.pos[2]);
            }
        }
    }
    // The data pins ([D] placements), read at load while the slots are fresh: the wing guard
    // placement 9 (weapon 0x19, no spawner group, behaviour prop 0x17 = fire without fresh sight)
    // and the office civilian placement 1 (team 3, death channel 0x63, spawner group 2 =
    // AI_DroneSpawner #716's group [D audit rows 1/9/716]).
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let g9 = sim.drones.list.iter().find(|d| d.placement == 9).expect("placement 9 exists [D]");
        assert_eq!(g9.weapon, 0x19, "the wing guard's weapon [D audit row 9]");
        assert_eq!(g9.group, 0, "the wing guard is in no spawner group [D row 9 param 4]");
        let civ = sim.drones.list.iter().find(|d| d.death_channel == 0x63).expect("the death-channel-0x63 civilian [D row 1]");
        assert_eq!(civ.team, 3, "the office civilian is team 3 [D row 1]");
        assert_eq!(civ.group, 2, "she is spawner group 2 — AI_DroneSpawner #716's group (p1=2, kill ch 0x6) [D rows 1/716]");
    }
    tower2::walk_12(&mut w);
    led.observe(&mut w);
    // walk_12_exit's proven door[19] legs, then the observed wing steering.
    w.walk_to(walker::Wp::new(4.0, 1.1, 5.0).r(0.6), "back across the security room");
    w.walk_to(walker::Wp::new(6.0, 1.1, -0.86).r(0.6), "door[19], the west side");
    w.walk_to(walker::Wp::new(8.4, 1.1, -0.6).r(0.6), "back through door[19]");
    w.walk_to(walker::Wp::new(10.75, 1.1, -0.50).r(1.0), "the corridor");
    w.walk_to(walker::Wp::new(11.25, 1.1, 1.25).r(1.0), "the corridor north");
    w.walk_to(walker::Wp::new(13.25, 1.1, 3.25).r(1.0), "the east lobby");
    let legs: &[(f32, f32, &str)] = &[
        (21.00, 3.50, "the east lobby east"), (24.25, 2.75, "the east wing corner"), (24.00, -1.00, "the east wing"),
        (23.50, -4.50, "the east wing south"), (22.25, -7.25, "the south-east bend"), (20.75, -9.75, "the south-east bend 2"),
        (19.00, -12.00, "the south corridor east"), (15.00, -12.00, "the south corridor"), (11.00, -12.00, "the south corridor west"),
    ];
    for &(x, z, label) in legs { leg(&mut w, x, z, label, &mut led); }
    for l in &led.out { eprintln!("[d{difficulty}] {l}"); }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        // The fire-gate question, on the real route: the wing guard fires at Bond's back down the
        // wing — no game rule refuses the shot (the fire request arms from the sight ack +
        // GunThroughWall alone [M 0x148338:110-126, 0x166328:121-151]) and the port must not
        // invent one. He carried prop 0x17 in both difficulties [L this run's ledger].
        assert!(led.max_shots_g9 >= 1,
            "d{difficulty}: the wing guard fired down the wing during the real walk ({} shots)", led.max_shots_g9);
        // Prop 0x17 rides the ALERT behaviour block (the normal/alert words differ at 19/20 [D row 9]),
        // so the capture needs him alive mid-alert; the d3 roster's early fights can take him before
        // the observed window — pin it when captured, report otherwise.
        if let Some(p17) = led.p17_g9 {
            assert_eq!(p17, 1,
                "d{difficulty}: the wing guard carries prop 0x17 once alert [D behaviour block, L this run]");
        } else {
            eprintln!("[probe d{difficulty}] the wing guard never lived in the observed window — prop 0x17 unpinned this run");
        }
        // Her fate is timing/RNG-dependent and game-legal either way (the spawner #716 cleanup
        // removes an undrawn spawned copy on the wing's ch 0x6 [M 0x137e68:235-288 — 0x12 is in
        // the keep-gated level split]); report it, pin nothing.
        if let Some(i) = sim.drones.list.iter().position(|d| d.death_channel == 0x63) {
            let c = &sim.drones.list[i];
            eprintln!("[probe d{difficulty}] end: civilian idx{i} present{} health {:.1} corpse {} fail{:?}",
                c.present, c.health, c.corpse, c.fail_reason.map(|r| r.label));
        }
        eprintln!("[probe d{difficulty}] end: guard9 max shots {} fail_label {:#x}",
            led.max_shots_g9, sim.drones.mission.fail_label);
    }
}

#[test]
fn probe_the_wing_guard_and_the_office_civilian_operative() { run(1); }

#[test]
fn probe_the_wing_guard_and_the_office_civilian_agent() { run(3); }
