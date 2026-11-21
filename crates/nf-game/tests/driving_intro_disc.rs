//! Disc proof for the driving missions' scripted control and camera layer
//! (`nf_game::driving::intro`, `nf_formats::drivepath`, docs/systems/driving-intros.md): every
//! spline's path animation resolves, and every track's opening, run through the game's own
//! triggers, rules and events, reaches the state its data gives it.
//!
//! Reads the full retail image named by `NF_ISO`; skips cleanly when it is unset or the image
//! has no `DRIVING/` extents.

use nf_formats::crp::Crp;
use nf_formats::drivepath::{map_paths, MapPaths};
use nf_formats::iso9660::IsoImage;
use nf_formats::trackstart::{ground_height, map_script, track_spawn, MapScript};
use nf_game::driving::intro::{CameraMode, IntroOut, IntroState, LetterboxState, Owner};
use nf_game::driving::script::{DriveScript, ElementFacts, Facts, Output};

const TRACKS: [&str; 8] = ["MIS01", "MIS11", "MIS13A", "MIS13B", "MIS13C", "MIS3", "MIS4", "RACE"];

fn track_crp(iso: &IsoImage, viv: &str) -> Option<Vec<u8>> {
    let e = iso.find(&format!("/DRIVING/{viv}.VIV"))?;
    let g = nf_formats::disc::read_bigf(iso, e).ok()?;
    let t = g.entries.iter().find(|x| {
        let n = x.name.to_ascii_lowercase();
        n.starts_with("data\\track\\") && n.ends_with(".crp")
    })?;
    let raw = iso.read_bytes(e.lba as u64 * 2048 + t.offset as u64, t.size as u64).ok()?;
    if raw.iter().take(64).all(|&b| b == 0) {
        return None;
    }
    Some(if nf_formats::refpack::is_refpack(&raw) { nf_formats::refpack::decompress(&raw).unwrap() } else { raw })
}

fn tracks() -> Option<Vec<(&'static str, Vec<u8>)>> {
    let Ok(path) = std::env::var("NF_ISO") else {
        eprintln!("skip: NF_ISO not set");
        return None;
    };
    let iso = IsoImage::open(&path).expect("open NF_ISO");
    if track_crp(&iso, "MIS01").is_none() {
        eprintln!("skip: no /DRIVING/ extents in this image");
        return None;
    }
    Some(TRACKS.iter().map(|&v| (v, track_crp(&iso, v).unwrap())).collect())
}

#[test]
fn every_spline_names_its_own_path_animation() {
    let Some(tracks) = tracks() else { return };
    let mut unresolved = Vec::new();
    for (viv, dec) in &tracks {
        let crp = Crp::parse(dec).unwrap();
        let m = map_script(&crp).unwrap();
        let p = map_paths(&crp).unwrap();
        assert!(!p.splines.is_empty(), "{viv}");
        for s in &p.splines {
            let Some(a) = p.spline_anim(&m, s.index) else {
                // [D] MIS13C's "path_(JC_freighter)" names no `pt` record.
                eprintln!("  {viv} spline {} {}: anim ref {:#x} = {:?}", s.index, s.name, s.anim_ref, m.ref_path(s.anim_ref));
                unresolved.push((*viv, s.index));
                continue;
            };
            assert_eq!(a.name, s.name, "{viv} spline {}", s.index);
            assert_eq!(s.matrix[3][3], 1.0, "{viv}");
            let cubic = a.pos.is_some_and(|c| a.channels[c].cubic);
            assert!(a.duration > 0.0 && (a.position(0.0).is_some() || cubic), "{viv} {}", a.name);

        }
        // Every spline / camera argument the script passes resolves.
        let mut n = 0;
        for a in m.triggers.iter().flat_map(|t| &t.actions).chain(m.rules.iter().flat_map(|r| &r.actions)) {
            match a.id {
                0xca | 0xd1 => assert!(p.spline_ref(&m, a.word(2)).is_some(), "{viv} {:#x}", a.id),
                0xcc => assert!(p.spline_ref(&m, a.word(1)).is_some(), "{viv} cam spline"),
                0x3f if a.word(1) != 0 => assert!(p.cam_ref(&m, a.word(1)).is_some(), "{viv} cam"),
                _ => continue,
            }
            n += 1;
        }
        eprintln!("{viv}: {} splines, {} cams, {} pt, {n} refs", p.splines.len(), p.cams.len(), p.anims.len());
    }
    assert_eq!(unresolved, vec![("MIS13C", 25)]);
}

/// One opening: the script, the layer, the car held at the spawn or carried by its spline.
struct Run {
    s: DriveScript,
    f: Facts,
    i: IntroState,
    frame: u32,
    log: Vec<(u32, String)>,
}

impl Run {
    fn new(m: &MapScript, spawn: [f32; 3]) -> Run {
        let mut f = Facts { player_pos: spawn, player_radius: 2.5, difficulty: 1, ..Default::default() };
        for r in &m.rules {
            if r.word(0x10) == 4 {
                f.elements.insert(r.word(0x14), ElementFacts { health: 1000.0, speed: 20.0, pos: [1e5, 0.0, 1e5], ..Default::default() });
            }
        }
        let mut r = Run { s: DriveScript::new(m.clone()), f, i: IntroState::new(60), frame: 0, log: Vec::new() };
        r.s.drain(&r.f.clone());
        r
    }
    fn absorb(&mut self, m: &MapScript, p: &MapPaths) {
        for o in self.s.take_output() {
            match o {
                Output::StartMission { .. } => self.i.start_mission(),
                Output::Effect { action, .. } => {
                    let before = (self.i.player_drives(), self.i.pov, self.i.spline, self.i.stop, self.i.camera.clone(), self.i.letterbox.state);
                    self.i.apply(&action, m, p);
                    let after = (self.i.player_drives(), self.i.pov, self.i.spline, self.i.stop, self.i.camera.clone(), self.i.letterbox.state);
                    if before != after {
                        self.log.push((self.frame, format!("{:#x} -> drives {} pov {} spline {:?} stop {:?} cam {:?} lb {:?}", action.id, after.0, after.1, after.2, after.3, after.4, after.5)));
                    }
                }
                _ => {}
            }
        }
        for o in self.i.out.drain(..) {
            if let IntroOut::Text { id, secs } = o {
                self.log.push((self.frame, format!("text {id:#x} {secs}s")));
            }
        }
    }
    fn step(&mut self, m: &MapScript, p: &MapPaths) {
        self.frame += 1;
        self.s.step(&self.f.clone());
        self.absorb(m, p);
        self.i.step();
        if let Some(q) = self.i.spline_position(p) {
            let o = self.f.player_pos;
            self.f.player_vel = [(q[0] - o[0]) * 60.0, (q[1] - o[1]) * 60.0, (q[2] - o[2]) * 60.0];
            self.f.player_pos = q;
        } else {
            self.f.player_vel = [0.0; 3];
        }
    }
}

/// The opening every track's data gives [D: the start lists and the triggers beside the start,
/// run by the ported rules; M for each event's effect]. `frames`: how long the opening is run.
struct Expect {
    viv: &'static str,
    frames: u32,
    /// The player steers at the end (owner player, no spline, no stop, no POV).
    drives: bool,
    owner: Owner,
    pov: bool,
    spline: Option<usize>,
    /// The frame the last change of the layer happened (None: nothing changed after frame 1).
    last_change: Option<u32>,
}

const EXPECT: [Expect; 8] = [
    // The sniper section: on the helicopter's spline 0, POV mode; the letterbox the start list
    // put up comes down when the path passes trigger 596 (frame 10).
    Expect { viv: "MIS01", frames: 600, drives: false, owner: Owner::Player, pov: true, spline: Some(0), last_change: Some(10) },
    Expect { viv: "MIS11", frames: 600, drives: true, owner: Owner::Player, pov: false, spline: None, last_change: None },
    Expect { viv: "MIS13A", frames: 600, drives: true, owner: Owner::Player, pov: false, spline: None, last_change: None },
    // On spline 0 the whole drive; trigger 78 starts camera 0 + letterbox, trigger 89 (173 u
    // down the path) ends them and turns the POV mode on.
    Expect { viv: "MIS13B", frames: 1200, drives: false, owner: Owner::Player, pov: true, spline: Some(0), last_change: Some(JEX2_CAM_END) },
    // POV from the start list; trigger 17 puts the car on spline 0 ("path_nomovement") at
    // throttle 0.
    Expect { viv: "MIS13C", frames: 600, drives: false, owner: Owner::Player, pov: true, spline: Some(0), last_change: None },
    // The garage: CPU control and POV (180°, 0°) at speed 0; the next step (trigger 223)
    // needs the CPU driver to move the car, which is not modelled here.
    Expect { viv: "MIS3", frames: 600, drives: false, owner: Owner::Cpu, pov: true, spline: None, last_change: None },
    Expect { viv: "MIS4", frames: 600, drives: true, owner: Owner::Player, pov: false, spline: None, last_change: None },
    // The countdown: stopped from the start list, released at "GO".
    Expect { viv: "RACE", frames: 600, drives: true, owner: Owner::Player, pov: false, spline: None, last_change: Some(RACE_GO) },
];

/// RACE: timer 5 set to 5 s on frame 1; the texts at 4, 3, 2 s left and the release below 1 s.
const RACE_TEXTS: [(u32, u32); 4] = [(63, 0x8ff), (123, 0x8fe), (183, 0x8fd), (RACE_GO, 0x227c)];
const RACE_GO: u32 = 243;
const JEX2_CAM_END: u32 = 130;

#[test]
fn every_track_opening_runs_to_its_hand_over() {
    let Some(tracks) = tracks() else { return };
    for ((viv, dec), e) in tracks.iter().zip(EXPECT.iter()) {
        assert_eq!(*viv, e.viv);
        let crp = Crp::parse(dec).unwrap();
        let m = map_script(&crp).unwrap();
        let p = map_paths(&crp).unwrap();
        let tris = crp.extract_collision_tris().unwrap();
        let spawn = track_spawn(&crp, &tris).unwrap();
        // MIS13B: the ground found under the marker is 169 u below it, but the trigger beside
        // the start (78) and the start of the path it hands out (path_BondMaster01) are both
        // at the marker's height; the car is put at the marker so the opening can run
        // (trackstart.md §3 keeps the placement question open).
        let start = if *viv == "MIS13B" { spawn.marker } else { spawn.pos };
        let mut r = Run::new(&m, start);
        r.absorb(&m, &p);
        let mut texts = Vec::new();
        for _ in 0..e.frames {
            r.step(&m, &p);
            texts.extend(r.log.iter().filter(|(f, l)| *f == r.frame && l.starts_with("text")).map(|(f, l)| (*f, l.clone())));
        }
        let pos = r.f.player_pos;
        let g = ground_height(&tris, pos[0], pos[1], pos[2]);
        eprintln!("== {viv}: drives {} owner {:?} pov {} spline {:?} stop {:?} cam {:?} lb {:?} pos {pos:?} ground {g:?} path t {}", r.i.player_drives(), r.i.owner, r.i.pov, r.i.spline, r.i.stop, r.i.camera, r.i.letterbox.state, r.i.path.time);
        for (fr, l) in &r.log {
            eprintln!("   {fr:4} {l}");
        }
        assert_eq!((r.i.player_drives(), r.i.owner, r.i.pov, r.i.spline), (e.drives, e.owner, e.pov, e.spline), "{viv}");
        let last = r.log.iter().filter(|(f, l)| *f > 1 && !l.starts_with("text")).map(|(f, _)| *f).last();
        assert_eq!(last, e.last_change, "{viv}");
        // Nothing scripted is left on screen at the end of an opening.
        assert_eq!((r.i.camera.clone(), r.i.letterbox.state, r.i.stop), (CameraMode::Chase, LetterboxState::Off, None), "{viv}");
        if e.drives {
            // Control handed back with the car on the ground: the spawn lift above the surface.
            let g = g.unwrap_or_else(|| panic!("{viv}: no ground under {pos:?}"));
            assert!((pos[1] - g - 1.0).abs() < 1e-3, "{viv}: {pos:?} ground {g}");
            assert!(r.i.control_word, "{viv}");
        }
        if let Some(sp) = e.spline {
            // The car rides the path: where the path player says, at throttle × frames.
            let a = p.spline_anim(&m, sp).unwrap();
            assert_eq!(Some(pos), a.position(r.i.path.time).or(Some(start)), "{viv}");
        }
        if *viv == "RACE" {
            let got: Vec<(u32, u32)> = r
                .log
                .iter()
                .filter_map(|(f, l)| l.strip_prefix("text ").map(|t| (*f, u32::from_str_radix(t.split(' ').next().unwrap().trim_start_matches("0x"), 16).unwrap())))
                .collect();
            assert_eq!(got, RACE_TEXTS.to_vec());
            assert!(r.log.iter().any(|(f, l)| *f == RACE_GO && l.starts_with("0xdb")));
        }
    }
}
