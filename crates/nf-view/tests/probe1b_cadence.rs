//! WIP probe (real-health-00agent-1b-5): the missile p0 clock cadence and the run-to-run
//! determinism of the missile starts.
//!
//! - Cadence [M 0x1aa23c: `Script_Update` per tick in state 1]: the p0's owned-script clock must
//!   advance exactly +1.0 per sim tick while its missile is in state 1 (the isolate proved the
//!   [D] stream times already — t2/t10/t359 — so 1:1 is the port's law there).
//! - Determinism: two full loads in ONE process must produce byte-identical missile-start frames
//!   and clock traces (std HashMap iteration order differs between the two loads' fresh maps, so
//!   an order leak would show here without a second process).

#[path = "common/walker.rs"]
mod walker;

use walker::Walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

/// One run: load, wait for the arrival, hold the launch-order inputs on (the isolate drive of
/// AND #202's inputs — the mux writes ch 19 from them every tick [M triggers.rs case 3]), tick
/// `ticks` times and return the telemetry: for every tick (frame, per-missile (state, clock)).
fn run(tag: &str, ticks: u32) -> (u32, [u32; 8], Vec<(u32, u16, f32)>) {
    let mut w = Walker::new(nf_view::load(&iso(), "0700001b.bin", false).unwrap());
    w.wait_enabled(3000, "the shuttle arrival");
    let mut ch19 = 0u32;
    let mut starts = [0u32; 8];
    let mut trace: Vec<(u32, u16, f32)> = Vec::new();
    for _ in 0..ticks {
        {
            let sim = w.scene.sim.as_mut().unwrap();
            sim.drones.channels.set(193, 1, sim.drones.frame);
            sim.drones.channels.set(194, 1, sim.drones.frame);
        }
        w.tick(Default::default(), false);
        let sim = w.scene.sim.as_ref().unwrap();
        let f = sim.drones.frame;
        if ch19 == 0 && sim.drones.channels.on(19) { ch19 = f; }
        for m in &sim.drones.space_missiles {
            let i = (m.index - 1) as usize;
            if m.state == 1 && starts[i] == 0 { starts[i] = f; }
            if m.index == 5 {
                let clock = m.info.and_then(|h| sim.drones.scripts.owned(h)).map_or(-1.0, |s| s.clock);
                trace.push((f, m.state, clock));
            }
        }
    }
    eprintln!("[{tag}] ch19 {ch19} starts {starts:?} m5 trace len {}", trace.len());
    (ch19, starts, trace)
}

#[test]
fn p0_clock_cadence_and_two_run_determinism() {
    // 7000 ticks: the seed-105 missiles (m1/m2/m7) leave state 0 at ch19 + 6300 + 1 — the
    // window must cover them [D placement params 105/105/40/20/0/60/105/60].
    let (ch19_a, starts_a, trace_a) = run("run A", 7000);
    let (ch19_b, starts_b, trace_b) = run("run B", 7000);
    assert_eq!(ch19_a, ch19_b, "ch 19 raises on the same frame across loads");
    assert_eq!(starts_a, starts_b, "the missile starts are identical across loads (determinism)");
    assert_eq!(trace_a, trace_b, "m5's (state, clock) traces are identical across loads");
    // Cadence: m5 enters state 1 on the first gated tick (seed 0 [D index map]); from there the
    // clock advances exactly +1.0 per tick while state 1 stands [M 0x1aa23c].
    let mut rate_bad = Vec::new();
    for w in trace_a.windows(2) {
        let (f0, st0, c0) = w[0];
        let (f1, st1, c1) = w[1];
        if st0 == 1 && st1 == 1 && f1 == f0 + 1 && (c1 - c0 - 1.0).abs() > 1e-4 {
            rate_bad.push((f0, c0, f1, c1));
        }
    }
    assert!(rate_bad.is_empty(), "the p0 clock must advance +1.0 per sim tick [M 0x1aa23c], bad steps {rate_bad:?}");
    // The start law [L this run, both loads identical]: the exit tick is ch19 + 1 for seed 0
    // (the first gated tick — the trigger pass writes ch 19 after the missiles tick) and within
    // ±1 tick of ch19 + seed·60 for the others — the exact tick depends on the f32 countdown
    // accumulation (seed 105 exits at +6300, seed 40 at +2401, COUNTDOWN_STEP 1/60
    // [M 0x1aa1f0-208]). The walk's deadline model reads S_k straight off MissileDeploy, so the
    // deadline math is unaffected by the ±1.
    let seeds = [105.0, 105.0, 40.0, 20.0, 0.0, 60.0, 105.0, 60.0];
    for (i, s) in seeds.iter().enumerate() {
        let want = if *s == 0.0 { ch19_a + 1 } else { ch19_a + (*s * 60.0) as u32 };
        let got = starts_a[i] as i64 - want as i64;
        assert!(got.abs() <= 1, "missile {} starts within ±1 of ch19 + seed*60 [M state-0 countdown, L]: got {got:+}", i + 1);
    }
}
