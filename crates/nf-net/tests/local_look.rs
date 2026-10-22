//! mp-local-look: `Session::pending_local_look` — the local turn/look that is scheduled but
//! has not run yet (the view adds it so the camera answers the mouse without the lockstep's
//! input delay). A solo host on the shim: every sample lands `input_delay` ticks ahead.

use std::net::SocketAddr;

use nf_net::session::{Config, Session};
use nf_net::shim::Shim;
use nf_net::{MatchSettings, NEUTRAL};

#[test]
fn pending_local_look_sums_the_ticks_not_yet_run() {
    let shim = Shim::new(1);
    let a: SocketAddr = "127.0.0.1:7300".parse().unwrap();
    let cfg = Config { input_delay: 3, ..Default::default() };
    let mut host = Session::new_host(cfg, shim.endpoint(a), MatchSettings::default(), 7);
    host.start_match(0).expect("a solo host starts");
    let _ = host.poll(shim.now_ms());
    let mut inp = NEUTRAL;
    inp.turn = 0.1;
    inp.look = -0.02;
    // Sample on run ticks 0..=4: each lands 3 ticks later (ticks 3..=7).
    for t in 0..5u32 { host.submit_local(t, inp); }
    // Ticks 3..=7 are scheduled; after tick 2, all five are pending.
    let (t5, _) = host.pending_local_look(2);
    assert!((t5 - 0.5).abs() < 1e-5, "five pending turns after tick 2, got {t5}");
    // After tick 4: the samples for ticks 5, 6, 7 are scheduled but not run.
    let (t, l) = host.pending_local_look(4);
    assert!((t - 0.3).abs() < 1e-5, "three pending turns of 0.1, got {t}");
    assert!((l + 0.06).abs() < 1e-5, "three pending looks of -0.02, got {l}");
    // Sister: nothing pending after the last scheduled tick.
    assert_eq!(host.pending_local_look(7), (0.0, 0.0));
}
