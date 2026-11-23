//! Native Rust macOS Metal CLI verification harness (`nf-macos-harness`).
//!
//! Usage:
//!   nf-macos-harness <iso-or-carved-dir> [options]
//!
//! Options:
//!   --out <dir>            Output directory for rendered PNGs (default: work/macos_rust_harness)
//!   --only <substr>        Run only suite cases whose name contains <substr>
//!   --level <archive>      Run a single level archive (e.g. 07000001.bin, 09000001.bin, MIS01)
//!   --steps <N>            Override simulation ticks (default: 30 on-foot/FE, 60 driving/MP)
//!   --size <WxH>           Render resolution (default: 1280x720)
//!   --difficulty <1..4>    Game difficulty (default: 2)
//!   --mp-match <H,B>       Run MP arena with H humans and B bots
//!   --throttle <f32>       Override forward/throttle input (-1.0..1.0)
//!   --steer <f32>          Override horizontal/steer input (-1.0..1.0)
//!   --fire                 Hold fire button during steps
//!   --audio                Enable live CoreAudio output
//!   --json                 Print NDJSON telemetry lines for each case
//!   --list-levels          List all discovered level archives and exit
//!   --help                 Show this usage text

use std::io::{self, BufRead, Write};
use std::path::PathBuf;
use std::time::Instant;

use anyhow::{anyhow, Context, Result};
use nf_view::menu::Action as MenuAction;
use nfmacos::controls::{parse_key, verify_controls, VirtualKbm};
use nfmacos::engine::{
    discover_levels, InputState, LoadOptions, MacEngine, BTN_AIM, BTN_CROUCH, BTN_FIRE, BTN_JUMP,
    BTN_PAUSE, BTN_RELOAD, BTN_USE, BTN_WEAP_NEXT, BTN_WEAP_PREV,
};
use nfmacos::inspect::{diff_png_files, inspect_json};

struct Cli {
    data_path: PathBuf,
    out_dir: PathBuf,
    only: Option<String>,
    level: Option<String>,
    steps: Option<u32>,
    width: u32,
    height: u32,
    difficulty: u32,
    mp_match: Option<(u32, u32)>,
    throttle: Option<f32>,
    steer: Option<f32>,
    fire: bool,
    audio: bool,
    json: bool,
    list_levels: bool,
    repl: bool,
    verify_controls: bool,
}

fn parse_size(s: &str) -> Result<(u32, u32)> {
    let (w, h) = s
        .split_once('x')
        .or_else(|| s.split_once('X'))
        .ok_or_else(|| anyhow!("--size expects WxH (e.g. 1280x720), got {s}"))?;
    Ok((w.parse()?, h.parse()?))
}

fn parse_mp_match(s: &str) -> Result<(u32, u32)> {
    let (h, b) = s
        .split_once(',')
        .ok_or_else(|| anyhow!("--mp-match expects H,B (e.g. 1,3), got {s}"))?;
    Ok((h.trim().parse()?, b.trim().parse()?))
}

fn print_help() {
    println!(
        "nf-macos-harness <iso-or-carved-dir> [options]\n\
         \n\
         Options:\n\
           --out <dir>            Output PNG directory (default: work/macos_rust_harness)\n\
           --only <substr>        Filter suite cases by substring\n\
           --level <archive>      Run a single archive (e.g. 07000001.bin, 09000001.bin, MIS01)\n\
           --steps <N>            Override tick count\n\
           --size <WxH>           Render resolution (default: 1280x720)\n\
           --difficulty <1..4>    Game difficulty (default: 2)\n\
           --mp-match <H,B>       Multiplayer match (H humans, B bots)\n\
           --throttle <f32>       Forward / throttle axis (-1.0..1.0)\n\
           --steer <f32>          Strafe / steer axis (-1.0..1.0)\n\
           --fire                 Hold fire button\n\
           --audio                Enable live CoreAudio output\n\
           --json                 Emit NDJSON telemetry lines\n\
           --list-levels          Discover and print all levels from ISO or carved dir\n\
           --verify-controls      Verify all keyboard/mouse/directional mappings (FE, On-Foot, Driving)\n\
           --repl                 Interactive persistent Metal control REPL (stdin -> JSON stdout)\n\
           --help                 Show this help"
    );
}

fn parse_args() -> Result<Option<Cli>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "--help" || a == "-h") {
        print_help();
        return Ok(None);
    }

    let mut positional = Vec::new();
    let mut out_dir = PathBuf::from("work/macos_rust_harness");
    let mut only = None;
    let mut level = None;
    let mut steps = None;
    let mut width = 1280u32;
    let mut height = 720u32;
    let mut difficulty = 2u32;
    let mut mp_match = None;
    let mut throttle = None;
    let mut steer = None;
    let mut fire = false;
    let mut audio = false;
    let mut json = false;
    let mut list_levels = false;
    let mut repl = false;
    let mut verify_ctrls = false;

    let mut i = 0;
    while i < args.len() {
        let a = args[i].as_str();
        match a {
            "--out" => {
                i += 1;
                out_dir = PathBuf::from(args.get(i).context("--out requires a path")?);
            }
            "--only" => {
                i += 1;
                only = Some(args.get(i).context("--only requires a substring")?.clone());
            }
            "--level" => {
                i += 1;
                level = Some(args.get(i).context("--level requires an archive")?.clone());
            }
            "--steps" => {
                i += 1;
                steps = Some(args.get(i).context("--steps requires a number")?.parse()?);
            }
            "--size" => {
                i += 1;
                let (w, h) = parse_size(args.get(i).context("--size requires WxH")?)?;
                width = w;
                height = h;
            }
            "--difficulty" => {
                i += 1;
                difficulty = args.get(i).context("--difficulty requires 1..4")?.parse()?;
            }
            "--mp-match" => {
                i += 1;
                mp_match = Some(parse_mp_match(
                    args.get(i).context("--mp-match requires H,B")?,
                )?);
            }
            "--throttle" => {
                i += 1;
                throttle = Some(args.get(i).context("--throttle requires a float")?.parse()?);
            }
            "--steer" => {
                i += 1;
                steer = Some(args.get(i).context("--steer requires a float")?.parse()?);
            }
            "--fire" => fire = true,
            "--audio" => audio = true,
            "--json" => json = true,
            "--list-levels" => list_levels = true,
            "--verify-controls" => verify_ctrls = true,
            "--repl" => repl = true,
            _ if a.starts_with("--") => anyhow::bail!("unknown flag: {a} (try --help)"),
            _ => positional.push(a.to_string()),
        }
        i += 1;
    }

    let data_path = positional
        .first()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("../007 - Nightfire (USA).iso"));
    if let Some(second) = positional.get(1) {
        if out_dir == PathBuf::from("work/macos_rust_harness") {
            out_dir = PathBuf::from(second);
        }
    }

    Ok(Some(Cli {
        data_path,
        out_dir,
        only,
        level,
        steps,
        width,
        height,
        difficulty,
        mp_match,
        throttle,
        steer,
        fire,
        audio,
        json,
        list_levels,
        repl,
        verify_controls: verify_ctrls,
    }))
}

struct SuiteCase {
    name: String,
    archive: String,
    steps: u32,
    input: InputState,
    opts: LoadOptions,
    check_motion: bool,
}

fn default_cases(difficulty: u32) -> Vec<SuiteCase> {
    let base_opts = LoadOptions {
        difficulty,
        ..Default::default()
    };
    let mp_opts = LoadOptions {
        difficulty,
        mp_match: Some((1, 3)),
        ..Default::default()
    };
    let drive_in = InputState {
        move_y: 1.0,
        ..Default::default()
    };
    vec![
        SuiteCase {
            name: "fe_main".into(),
            archive: "07000048.bin".into(),
            steps: 30,
            input: InputState::default(),
            opts: base_opts,
            check_motion: false,
        },
        SuiteCase {
            name: "onfoot_01_exchange".into(),
            archive: "07000001.bin".into(),
            steps: 30,
            input: InputState {
                move_y: 0.5,
                ..Default::default()
            },
            opts: base_opts,
            check_motion: false,
        },
        SuiteCase {
            name: "onfoot_05_alpine".into(),
            archive: "07000005.bin".into(),
            steps: 30,
            input: InputState::default(),
            opts: base_opts,
            check_motion: false,
        },
        SuiteCase {
            name: "onfoot_08_nightshift".into(),
            archive: "07000008.bin".into(),
            steps: 30,
            input: InputState::default(),
            opts: base_opts,
            check_motion: false,
        },
        SuiteCase {
            name: "mp_23_skyrail".into(),
            archive: "07000023.bin".into(),
            steps: 60,
            input: InputState::default(),
            opts: mp_opts,
            check_motion: false,
        },
        SuiteCase {
            name: "driving_mis01_paris".into(),
            archive: "09000001.bin".into(),
            steps: 60,
            input: drive_in,
            opts: base_opts,
            check_motion: true,
        },
        SuiteCase {
            name: "driving_mis3_alpine".into(),
            archive: "09000005.bin".into(),
            steps: 60,
            input: drive_in,
            opts: base_opts,
            check_motion: true,
        },
        SuiteCase {
            name: "driving_mis4_vanquished".into(),
            archive: "09000006.bin".into(),
            steps: 60,
            input: drive_in,
            opts: base_opts,
            check_motion: true,
        },
        SuiteCase {
            name: "driving_mis11_sub".into(),
            archive: "09000002.bin".into(),
            steps: 60,
            input: drive_in,
            opts: base_opts,
            check_motion: true,
        },
        SuiteCase {
            name: "driving_mis13a_ultralight".into(),
            archive: "09000003.bin".into(),
            steps: 60,
            input: drive_in,
            opts: base_opts,
            check_motion: true,
        },
        SuiteCase {
            name: "driving_mis13b_island".into(),
            archive: "09000004.bin".into(),
            steps: 60,
            input: drive_in,
            opts: base_opts,
            check_motion: true,
        },
        SuiteCase {
            name: "driving_mis13c_boat".into(),
            archive: "09000007.bin".into(),
            steps: 60,
            input: drive_in,
            opts: base_opts,
            check_motion: true,
        },
        SuiteCase {
            name: "driving_race_bonus".into(),
            archive: "09000008.bin".into(),
            steps: 60,
            input: drive_in,
            opts: base_opts,
            check_motion: true,
        },
    ]
}

fn run_repl(engine: &mut MacEngine, cli: &Cli) -> Result<()> {
    let mut stdout = io::stdout().lock();
    let mut kbm = VirtualKbm::default();
    if let Some(ref archive) = cli.level {
        engine.load_level_with(
            archive,
            LoadOptions {
                difficulty: cli.difficulty,
                mp_match: cli.mp_match,
                ..Default::default()
            },
        )?;
    }
    writeln!(stdout, "{{\"ok\":true,\"ev\":\"ready\",\"telemetry\":{}}}", engine.telemetry().to_json())?;
    stdout.flush()?;

    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let line = line?;
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        let cmd = parts[0];
        let resp: Result<String> = match cmd {
            "quit" | "exit" => {
                writeln!(stdout, "{{\"ok\":true,\"ev\":\"bye\"}}")?;
                stdout.flush()?;
                break;
            }
            "status" => Ok(format!("{{\"ok\":true,\"telemetry\":{}}}", engine.telemetry().to_json())),
            "inspect" => {
                let topic = parts.get(1).copied().unwrap_or("all");
                Ok(format!("{{\"ok\":true,\"topic\":\"{topic}\",\"data\":{}}}", inspect_json(engine, topic)))
            }
            "bench" => (|| -> Result<String> {
                let n: u32 = parts.get(1).map(|s| s.parse()).transpose()?.unwrap_or(60);
                let (w, h) = match parts.get(2) {
                    Some(sz) => parse_size(sz)?,
                    None => (cli.width, cli.height),
                };
                let rep = engine.bench_frames(n, w, h)?;
                Ok(format!("{{\"ok\":true,\"bench\":{}}}", rep.to_json()))
            })(),
            "diff" => (|| -> Result<String> {
                let a = parts.get(1).copied().ok_or_else(|| anyhow!("diff <a.png> <b.png> [out_diff.png]"))?;
                let b = parts.get(2).copied().ok_or_else(|| anyhow!("diff <a.png> <b.png> [out_diff.png]"))?;
                let out = parts.get(3).map(std::path::Path::new);
                let d = diff_png_files(std::path::Path::new(a), std::path::Path::new(b), out)?;
                Ok(format!("{{\"ok\":true,\"diff\":{}}}", d.to_json()))
            })(),
            "levels" => {
                let lvls = engine.list_levels();
                let arr: Vec<String> = lvls.iter().map(|s| format!("\"{s}\"")).collect();
                Ok(format!("{{\"ok\":true,\"levels\":[{}]}}", arr.join(",")))
            }
            "auto_transition" => {
                let on = matches!(parts.get(1).copied(), Some("on" | "1" | "true"));
                engine.auto_transition = on;
                Ok(format!("{{\"ok\":true,\"auto_transition\":{on}}}"))
            }
            "load" => (|| -> Result<String> {
                let archive = parts.get(1).copied().ok_or_else(|| anyhow!("load <archive> [difficulty=N] [mp=H,B]"))?;
                let mut opts = LoadOptions {
                    difficulty: cli.difficulty,
                    ..Default::default()
                };
                for &kv in parts.iter().skip(2) {
                    if let Some(v) = kv.strip_prefix("difficulty=") {
                        opts.difficulty = v.parse()?;
                    } else if let Some(v) = kv.strip_prefix("mp=") {
                        opts.mp_match = Some(parse_mp_match(v)?);
                    }
                }
                let t0 = Instant::now();
                engine.load_level_with(archive, opts)?;
                let ms = t0.elapsed().as_secs_f64() * 1000.0;
                Ok(format!("{{\"ok\":true,\"ms\":{ms:.1},\"telemetry\":{}}}", engine.telemetry().to_json()))
            })(),
            "input" => (|| -> Result<String> {
                let mut inp = engine.input;
                for &tok in parts.iter().skip(1) {
                    if tok == "reset" {
                        inp = InputState::default();
                        continue;
                    }
                    let Some((k, v)) = tok.split_once('=') else {
                        anyhow::bail!("bad input token: {tok}");
                    };
                    let set_btn = |buttons: &mut i32, bit: i32, val: &str| {
                        if matches!(val, "1" | "true" | "on") {
                            *buttons |= bit;
                        } else {
                            *buttons &= !bit;
                        }
                    };
                    match k {
                        "move_x" | "steer" => inp.move_x = v.parse()?,
                        "move_y" | "throttle" => inp.move_y = v.parse()?,
                        "look_dx" => inp.look_dx = v.parse()?,
                        "look_dy" => inp.look_dy = v.parse()?,
                        "buttons" => inp.buttons = v.parse()?,
                        "fire" => set_btn(&mut inp.buttons, BTN_FIRE, v),
                        "aim" => set_btn(&mut inp.buttons, BTN_AIM, v),
                        "reload" => set_btn(&mut inp.buttons, BTN_RELOAD, v),
                        "jump" | "handbrake" => set_btn(&mut inp.buttons, BTN_JUMP, v),
                        "crouch" => set_btn(&mut inp.buttons, BTN_CROUCH, v),
                        "use" => set_btn(&mut inp.buttons, BTN_USE, v),
                        "weap_next" => set_btn(&mut inp.buttons, BTN_WEAP_NEXT, v),
                        "weap_prev" => set_btn(&mut inp.buttons, BTN_WEAP_PREV, v),
                        "pause" => set_btn(&mut inp.buttons, BTN_PAUSE, v),
                        _ => anyhow::bail!("unknown input field: {k}"),
                    }
                }
                engine.set_input(inp);
                Ok(format!(
                    "{{\"ok\":true,\"input\":{{\"move_x\":{:.3},\"move_y\":{:.3},\"look_dx\":{:.4},\"look_dy\":{:.4},\"buttons\":{}}}}}",
                    inp.move_x, inp.move_y, inp.look_dx, inp.look_dy, inp.buttons
                ))
            })(),
            "menu" => (|| -> Result<String> {
                let act = match parts.get(1).copied().unwrap_or("") {
                    "up" => MenuAction::Up,
                    "down" => MenuAction::Down,
                    "left" => MenuAction::Left,
                    "right" => MenuAction::Right,
                    "confirm" | "enter" => MenuAction::Confirm,
                    "back" | "esc" => MenuAction::Back,
                    "side" => MenuAction::Side,
                    "extra" => MenuAction::Extra,
                    other => anyhow::bail!("unknown menu action: {other}"),
                };
                engine.menu_action(act);
                Ok(format!("{{\"ok\":true,\"menu\":\"{}\"}}", parts[1]))
            })(),
            "step" => (|| -> Result<String> {
                let n: u32 = parts.get(1).map(|s| s.parse()).transpose()?.unwrap_or(1);
                let t0 = Instant::now();
                for _ in 0..n {
                    engine.step(1.0 / 60.0);
                }
                let ms = t0.elapsed().as_secs_f64() * 1000.0;
                Ok(format!("{{\"ok\":true,\"steps\":{n},\"ms\":{ms:.2},\"telemetry\":{}}}", engine.telemetry().to_json()))
            })(),
            "shot" => (|| -> Result<String> {
                let path = parts.get(1).copied().ok_or_else(|| anyhow!("shot <out.png> [WxH]"))?;
                let (w, h) = match parts.get(2) {
                    Some(sz) => parse_size(sz)?,
                    None => (cli.width, cli.height),
                };
                let t0 = Instant::now();
                let st = engine.render_png(w, h, path)?;
                let ms = t0.elapsed().as_secs_f64() * 1000.0;
                Ok(format!(
                    "{{\"ok\":true,\"path\":\"{}\",\"width\":{},\"height\":{},\"mean_luma\":{:.2},\"std_luma\":{:.2},\"ms\":{ms:.1}}}",
                    path.replace('\\', "\\\\").replace('"', "\\\""),
                    st.width,
                    st.height,
                    st.mean_luma,
                    st.std_luma
                ))
            })(),
            "pos" => (|| -> Result<String> {
                let x: f32 = parts.get(1).ok_or_else(|| anyhow!("pos <x> <y> <z> [yaw] [pitch]"))?.parse()?;
                let y: f32 = parts.get(2).ok_or_else(|| anyhow!("pos <x> <y> <z> [yaw] [pitch]"))?.parse()?;
                let z: f32 = parts.get(3).ok_or_else(|| anyhow!("pos <x> <y> <z> [yaw] [pitch]"))?.parse()?;
                let yaw = parts.get(4).map(|s| s.parse()).transpose()?;
                let pitch = parts.get(5).map(|s| s.parse()).transpose()?;
                engine.teleport([x, y, z], yaw, pitch);
                Ok(format!("{{\"ok\":true,\"telemetry\":{}}}", engine.telemetry().to_json()))
            })(),
            "cam" => (|| -> Result<String> {
                let x: f32 = parts.get(1).ok_or_else(|| anyhow!("cam <x> <y> <z> <yaw> <pitch>"))?.parse()?;
                let y: f32 = parts.get(2).ok_or_else(|| anyhow!("cam <x> <y> <z> <yaw> <pitch>"))?.parse()?;
                let z: f32 = parts.get(3).ok_or_else(|| anyhow!("cam <x> <y> <z> <yaw> <pitch>"))?.parse()?;
                let yaw: f32 = parts.get(4).ok_or_else(|| anyhow!("cam <x> <y> <z> <yaw> <pitch>"))?.parse()?;
                let pitch: f32 = parts.get(5).ok_or_else(|| anyhow!("cam <x> <y> <z> <yaw> <pitch>"))?.parse()?;
                engine.set_camera([x, y, z], yaw, pitch);
                Ok(format!("{{\"ok\":true,\"telemetry\":{}}}", engine.telemetry().to_json()))
            })(),
            "aim_at" => (|| -> Result<String> {
                let x: f32 = parts.get(1).ok_or_else(|| anyhow!("aim_at <x> <y> <z>"))?.parse()?;
                let y: f32 = parts.get(2).ok_or_else(|| anyhow!("aim_at <x> <y> <z>"))?.parse()?;
                let z: f32 = parts.get(3).ok_or_else(|| anyhow!("aim_at <x> <y> <z>"))?.parse()?;
                let (yaw, pitch, dist) = engine.aim_at([x, y, z])?;
                Ok(format!("{{\"ok\":true,\"yaw\":{yaw:.4},\"pitch\":{pitch:.4},\"dist\":{dist:.2},\"telemetry\":{}}}", engine.telemetry().to_json()))
            })(),
            "aim_drone" => (|| -> Result<String> {
                let idx: usize = parts.get(1).ok_or_else(|| anyhow!("aim_drone <idx>"))?.parse()?;
                let (yaw, pitch, dist) = engine.aim_drone(idx)?;
                Ok(format!("{{\"ok\":true,\"drone\":{idx},\"yaw\":{yaw:.4},\"pitch\":{pitch:.4},\"dist\":{dist:.2},\"telemetry\":{}}}", engine.telemetry().to_json()))
            })(),
            "walk_to" => (|| -> Result<String> {
                let x: f32 = parts.get(1).ok_or_else(|| anyhow!("walk_to <x> <y> <z> [max_ticks] [tol]"))?.parse()?;
                let y: f32 = parts.get(2).ok_or_else(|| anyhow!("walk_to <x> <y> <z> [max_ticks] [tol]"))?.parse()?;
                let z: f32 = parts.get(3).ok_or_else(|| anyhow!("walk_to <x> <y> <z> [max_ticks] [tol]"))?.parse()?;
                let max_ticks: u32 = parts.get(4).map(|s| s.parse()).transpose()?.unwrap_or(180);
                let tol: f32 = parts.get(5).map(|s| s.parse()).transpose()?.unwrap_or(1.0);
                let (ticks, dist, reached) = engine.walk_to([x, y, z], max_ticks, tol)?;
                Ok(format!("{{\"ok\":true,\"ticks\":{ticks},\"dist\":{dist:.2},\"reached\":{reached},\"telemetry\":{}}}", engine.telemetry().to_json()))
            })(),
            "skip_cutscene" => {
                let sent = engine.skip_cutscene();
                Ok(format!("{{\"ok\":true,\"skip_sent\":{sent}}}"))
            }
            "key_down" => (|| -> Result<String> {
                let k = parse_key(parts.get(1).copied().ok_or_else(|| anyhow!("key_down <key>"))?)?;
                if engine.is_in_menu() {
                    if let Some(act) = nf_view::menu::key_action(k) {
                        engine.menu_action(act);
                    }
                }
                kbm.key_down(k);
                let inp = kbm.to_input_state(engine.driving.is_some());
                engine.set_input(inp);
                Ok(format!("{{\"ok\":true,\"key_down\":\"{k:?}\"}}"))
            })(),
            "key_up" => (|| -> Result<String> {
                let k = parse_key(parts.get(1).copied().ok_or_else(|| anyhow!("key_up <key>"))?)?;
                kbm.key_up(k);
                let inp = kbm.to_input_state(engine.driving.is_some());
                engine.set_input(inp);
                Ok(format!("{{\"ok\":true,\"key_up\":\"{k:?}\"}}"))
            })(),
            "key_tap" => (|| -> Result<String> {
                let k = parse_key(parts.get(1).copied().ok_or_else(|| anyhow!("key_tap <key> [ticks]"))?)?;
                let n: u32 = parts.get(2).map(|s| s.parse()).transpose()?.unwrap_or(1);
                if engine.is_in_menu() {
                    if let Some(act) = nf_view::menu::key_action(k) {
                        engine.menu_action(act);
                    }
                }
                kbm.key_down(k);
                for _ in 0..n {
                    let inp = kbm.to_input_state(engine.driving.is_some());
                    engine.set_input(inp);
                    engine.step(1.0 / 60.0);
                }
                kbm.key_up(k);
                let inp = kbm.to_input_state(engine.driving.is_some());
                engine.set_input(inp);
                Ok(format!("{{\"ok\":true,\"key_tap\":\"{k:?}\",\"ticks\":{n},\"telemetry\":{}}}", engine.telemetry().to_json()))
            })(),
            "mouse_move" => (|| -> Result<String> {
                let dx: f64 = parts.get(1).ok_or_else(|| anyhow!("mouse_move <dx> <dy>"))?.parse()?;
                let dy: f64 = parts.get(2).ok_or_else(|| anyhow!("mouse_move <dx> <dy>"))?.parse()?;
                kbm.mouse_move(dx, dy, 1.0);
                let inp = kbm.to_input_state(engine.driving.is_some());
                engine.set_input(inp);
                Ok(format!("{{\"ok\":true,\"mouse_move\":[{dx:.2},{dy:.2}]}}"))
            })(),
            "mouse_btn" => (|| -> Result<String> {
                let btn = parts.get(1).copied().ok_or_else(|| anyhow!("mouse_btn <left|right> <down|up>"))?;
                let down = matches!(parts.get(2).copied(), Some("down" | "1" | "true"));
                match btn {
                    "left" => kbm.mouse_left = down,
                    "right" => kbm.mouse_right = down,
                    other => anyhow::bail!("unknown mouse button: {other}"),
                }
                let inp = kbm.to_input_state(engine.driving.is_some());
                engine.set_input(inp);
                Ok(format!("{{\"ok\":true,\"mouse_btn\":\"{btn}\",\"down\":{down}}}"))
            })(),
            "keys_reset" => {
                kbm.reset();
                engine.set_input(InputState::default());
                Ok("{\"ok\":true,\"keys_reset\":true}".to_string())
            }
            "verify_controls" => (|| -> Result<String> {
                let checks = verify_controls(engine)?;
                let passed = checks.iter().filter(|c| c.passed).count();
                let total = checks.len();
                let items: Vec<String> = checks.iter().map(|c| c.to_json()).collect();
                Ok(format!("{{\"ok\":{},\"passed\":{passed},\"total\":{total},\"checks\":[{}]}}", passed == total, items.join(",")))
            })(),
            _ => Err(anyhow!("unknown command: {cmd}")),
        };
        match resp {
            Ok(json) => writeln!(stdout, "{json}")?,
            Err(e) => {
                let msg = format!("{e:#}").replace('\\', "\\\\").replace('"', "\\\"");
                writeln!(stdout, "{{\"ok\":false,\"err\":\"{msg}\"}}")?;
            }
        }
        stdout.flush()?;
    }
    Ok(())
}

fn main() -> Result<()> {
    let Some(cli) = parse_args()? else {
        return Ok(());
    };

    if cli.list_levels {
        let levels = discover_levels(&cli.data_path);
        for lvl in &levels {
            println!("{lvl}");
        }
        eprintln!("[nf-macos-harness] {} level archive(s) discovered", levels.len());
        return Ok(());
    }

    let mut engine = MacEngine::init_with_audio(&cli.data_path, cli.audio)?;
    if cli.verify_controls {
        let checks = verify_controls(&mut engine)?;
        let mut failed = 0usize;
        for c in &checks {
            if !c.passed {
                failed += 1;
            }
            if cli.json {
                println!("{}", c.to_json());
            } else {
                let tag = if c.passed { "PASS" } else { "FAIL" };
                println!("[{tag}] {:8} {:46} ({})", c.mode, c.name, c.detail);
            }
        }
        if failed > 0 {
            anyhow::bail!("{failed}/{} control mapping check(s) failed", checks.len());
        }
        if !cli.json {
            println!("[nf-macos-harness] all {}/{} control mapping checks PASSED", checks.len(), checks.len());
        }
        return Ok(());
    }
    if cli.repl {
        return run_repl(&mut engine, &cli);
    }
    if !cli.json {
        println!("[nf-macos-harness] initialized: {}", engine.status_line());
    }

    let cases: Vec<SuiteCase> = if let Some(ref archive) = cli.level {
        let is_driving = nfmacos::engine::resolve_driving_viv(archive).is_some();
        let default_steps = if is_driving || cli.mp_match.is_some() { 60 } else { 30 };
        let input = InputState {
            move_y: cli.throttle.unwrap_or(if is_driving { 1.0 } else { 0.0 }),
            move_x: cli.steer.unwrap_or(0.0),
            buttons: if cli.fire { BTN_FIRE } else { 0 },
            ..Default::default()
        };
        let safe_name = archive.replace(['/', '\\', '.'], "_");
        vec![SuiteCase {
            name: safe_name,
            archive: archive.clone(),
            steps: cli.steps.unwrap_or(default_steps),
            input,
            opts: LoadOptions {
                difficulty: cli.difficulty,
                mp_match: cli.mp_match,
                ..Default::default()
            },
            check_motion: false,
        }]
    } else {
        default_cases(cli.difficulty)
            .into_iter()
            .filter(|c| {
                cli.only
                    .as_deref()
                    .is_none_or(|sub| c.name.contains(sub) || c.archive.contains(sub))
            })
            .map(|mut c| {
                if let Some(s) = cli.steps {
                    c.steps = s;
                }
                if let Some(t) = cli.throttle {
                    c.input.move_y = t;
                }
                if let Some(st) = cli.steer {
                    c.input.move_x = st;
                }
                if cli.fire {
                    c.input.buttons |= BTN_FIRE;
                }
                c
            })
            .collect()
    };

    if cases.is_empty() {
        anyhow::bail!("no harness cases matched filter");
    }

    let mut failed = 0usize;
    for c in &cases {
        let t0 = Instant::now();
        engine.load_level_with(&c.archive, c.opts)?;
        let start_pos = engine.telemetry().pos;
        engine.set_input(c.input);
        for _ in 0..c.steps {
            engine.step(1.0 / 60.0);
        }
        let end_tel = engine.telemetry();
        let moved = ((end_tel.pos[0] - start_pos[0]).powi(2)
            + (end_tel.pos[2] - start_pos[2]).powi(2))
        .sqrt();
        let png_path = cli.out_dir.join(format!("{}.png", c.name));
        let stats = engine.render_png(cli.width, cli.height, &png_path)?;
        let render_ok = stats.mean_luma > 3.0 && stats.std_luma > 4.0;
        let intro_held = end_tel.player_drives == Some(false);
        let motion_ok = !c.check_motion || intro_held || moved > 0.5;
        let ok = render_ok && motion_ok;
        let tag = if ok { "PASS" } else { "FAIL" };
        if !ok {
            failed += 1;
        }
        let hold_note = if intro_held { " [intro-hold]" } else { "" };
        let secs = t0.elapsed().as_secs_f64();
        if cli.json {
            println!(
                "{{\"case\":\"{}\",\"ok\":{},\"width\":{},\"height\":{},\"mean_luma\":{:.2},\"std_luma\":{:.2},\"moved_m\":{:.2},\"secs\":{:.3},\"telemetry\":{}}}",
                c.name,
                ok,
                stats.width,
                stats.height,
                stats.mean_luma,
                stats.std_luma,
                moved,
                secs,
                end_tel.to_json()
            );
        } else {
            println!(
                "[{tag}] {:26} {}x{} mean={:5.1} std={:5.1} moved={:5.1}m ({:.2}s){hold_note} | {}",
                c.name,
                stats.width,
                stats.height,
                stats.mean_luma,
                stats.std_luma,
                moved,
                secs,
                engine.status_line()
            );
        }
    }

    if failed > 0 {
        anyhow::bail!("{failed} harness case(s) failed sanity checks");
    }
    Ok(())
}
