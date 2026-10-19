//! net-desync-detail: when two machines' state hashes disagree we only learn the tick — this
//! module makes every desync diagnosable.
//!
//! Each machine keeps the last [`HIST_CAP`] ticks of the named component hashes
//! ([`crate::scene::net_component_hashes`], fixed order `NET_HASH_COMPONENT_NAMES`) plus, at the
//! exchange cadence, each component's per-item breakdown. On a desync at tick T the machines
//! exchange the components for T (`PT_HASHCOMPS`, protocol 8) and then the first diverging
//! component's items (`PT_HASHDETAIL`); each writes a desync report zip (the bug-report format,
//! `kind: desync`) holding the first diverging component and slot, the match settings + seed,
//! the full input log since the match start (so `--bug-replay` can replay it — every machine's
//! view is the same lockstep transcript) and the component hash history.
//!
//! The whole layer is the port's own online infrastructure ([S port protocol] — the PS2 game has
//! no network mode); the determinism it diagnoses is the shared RNG + fixed 60 Hz tick.

use crate::scene::{fold_component_hashes, NET_HASH_COMPONENTS, NET_HASH_COMPONENT_NAMES};

/// How many ticks of component hashes each machine keeps (~10 s at 60 Hz — the desync is
/// detected within one exchange cadence, 15 ticks, so 600 is ample).
pub const HIST_CAP: usize = 600;

/// One machine's component-hash history. `entries` holds every tick's 7 component hashes (the
/// last [`HIST_CAP`] ticks); `detail` holds the per-item breakdowns at the exchange cadence only
/// (a desync is only ever detected at an exchanged tick, so only those need items).
#[derive(Default)]
pub struct HashHistory {
    pub entries: Vec<(u32, [u64; NET_HASH_COMPONENTS])>,
    pub detail: Vec<(u32, Detail)>,
}

/// The per-item breakdown of one tick: item hashes per component, in
/// `NET_HASH_COMPONENT_NAMES` order (the layout each vector's items follow is documented on
/// `Sim::net_component_items`).
pub type Detail = [Vec<u64>; NET_HASH_COMPONENTS];

impl HashHistory {
    pub fn push(&mut self, tick: u32, comps: [u64; NET_HASH_COMPONENTS]) {
        if self.entries.last().is_some_and(|(t, _)| *t >= tick) {
            return; // a re-run tick: the latest hash won at the source, keep the first push
        }
        if self.entries.len() == HIST_CAP {
            self.entries.remove(0);
        }
        self.entries.push((tick, comps));
    }

    pub fn push_detail(&mut self, tick: u32, items: Detail) {
        if self.detail.last().is_some_and(|(t, _)| *t >= tick) {
            return;
        }
        if self.detail.len() == HIST_CAP {
            self.detail.remove(0);
        }
        self.detail.push((tick, items));
    }

    pub fn comps_at(&self, tick: u32) -> Option<[u64; NET_HASH_COMPONENTS]> {
        self.entries.iter().rev().find(|(t, _)| *t == tick).map(|(_, c)| *c)
    }

    pub fn detail_at(&self, tick: u32) -> Option<&Detail> {
        self.detail.iter().rev().find(|(t, _)| *t == tick).map(|(_, d)| d)
    }

    /// The history as `tick c0 .. c6` lines (the report's `hash-history.txt`).
    pub fn to_text(&self) -> String {
        let mut o = format!("# tick {}\n", NET_HASH_COMPONENT_NAMES.join(" "));
        for (t, c) in &self.entries {
            o.push_str(&format!("{t} {:016x} {:016x} {:016x} {:016x} {:016x} {:016x} {:016x}\n", c[0], c[1], c[2], c[3], c[4], c[5], c[6]));
        }
        o
    }

    /// Parses `to_text`'s output (the `--bug-replay` side of a desync report).
    pub fn from_text(s: &str) -> HashHistory {
        let mut h = HashHistory::default();
        for l in s.lines().skip(1) {
            let f: Vec<&str> = l.split_whitespace().collect();
            if f.len() != 8 { continue; }
            let (Ok(t), v) = (f[0].parse::<u32>(), f[1..8].iter().map(|x| u64::from_str_radix(x, 16).unwrap_or(0)).collect::<Vec<_>>()) else { continue };
            h.entries.push((t, [v[0], v[1], v[2], v[3], v[4], v[5], v[6]]));
        }
        h
    }
}

/// The first component index where the two sides' component hashes differ (None = they agree).
pub fn first_diff_component(local: &[u64; NET_HASH_COMPONENTS], peer: &[u64; NET_HASH_COMPONENTS]) -> Option<usize> {
    (0..NET_HASH_COMPONENTS).find(|&i| local[i] != peer[i])
}

/// The first item index where two sides' item breakdowns of one component differ
/// (`None` = the items agree — the component hash difference came from the fold, not an item).
pub fn first_diff_item(local: &[u64], peer: &[u64]) -> Option<usize> {
    let n = local.len().min(peer.len());
    for i in 0..n {
        if local[i] != peer[i] {
            return Some(i);
        }
    }
    if local.len() != peer.len() { Some(n) } else { None }
}

/// What one machine concluded about the desync at `tick`.
#[derive(Clone, Debug)]
pub struct Diagnosis {
    pub tick: u32,
    /// The first component whose hashes disagree (index into `NET_HASH_COMPONENT_NAMES`).
    pub component: usize,
    /// The first item of that component that differs — the agent slot / bot index / world
    /// element index per the component's item layout (None: the items agreed, only the fold
    /// differed).
    pub item: Option<usize>,
    pub local_comps: [u64; NET_HASH_COMPONENTS],
    pub peer_comps: [u64; NET_HASH_COMPONENTS],
    /// The exchanged item breakdowns of the diverging component (local, peer).
    pub local_items: Vec<u64>,
    pub peer_items: Vec<u64>,
}

impl Diagnosis {
    pub fn component_name(&self) -> &'static str {
        NET_HASH_COMPONENT_NAMES[self.component]
    }

    /// The one-line summary the fail box shows (the draw code renders it; menu.rs holds a copy
    /// the page reads).
    pub fn ui_line(&self) -> String {
        let item = self.item.map(|i| format!(", slot/item {i}")).unwrap_or_default();
        format!("desync at tick {} — first diverging component {}{item}", self.tick, self.component_name())
    }
}

/// One machine's desync state machine. The driver (the app's net event loop, or a test) feeds it
/// the ticks it runs, the desync the session reported, and the packets the peer sent; it says
/// what to send and, once the exchange completes, what the diagnosis is.
#[derive(Default)]
pub struct Watcher {
    pub hist: HashHistory,
    /// The desync tick the session reported (both machines see the same one).
    desync: Option<u32>,
    /// Whether our `PT_HASHCOMPS` for the desync tick went out.
    comps_sent: bool,
    /// The peer's `PT_HASHCOMPS` (tick, components).
    peer_comps: Option<(u32, [u64; NET_HASH_COMPONENTS])>,
    /// Which component's detail we asked for / sent.
    detail_component: Option<usize>,
    detail_sent: bool,
    /// The peer's `PT_HASHDETAIL` (tick, component, items).
    peer_detail: Option<(u32, usize, Vec<u64>)>,
    /// Set once the exchange completes; the report writes from it.
    pub diagnosis: Option<Diagnosis>,
}

/// What the driver must send after a step (`send_comps` → `PT_HASHCOMPS`,
/// `send_detail` → `PT_HASHDETAIL`).
pub enum Action {
    SendComps(u32, [u64; NET_HASH_COMPONENTS]),
    SendDetail(u32, u8, Vec<u64>),
}

impl Watcher {
    /// The driver calls this every executed net tick (the app: beside `submit_hash`).
    pub fn on_tick(&mut self, tick: u32, comps: [u64; NET_HASH_COMPONENTS]) {
        self.hist.push(tick, comps);
    }

    /// The driver calls this at the exchange cadence (every `hash_every` ticks — the only ticks
    /// a desync is detected at) with the per-item breakdown.
    pub fn on_detail_tick(&mut self, tick: u32, items: Detail) {
        self.hist.push_detail(tick, items);
    }

    /// The session reported a desync at `tick`. Returns the `PT_HASHCOMPS` to send (the local
    /// components for that tick, from the history).
    pub fn on_desync(&mut self, tick: u32) -> Option<Action> {
        self.desync = Some(tick);
        self.step()
    }

    /// The peer's `PT_HASHCOMPS` arrived.
    pub fn on_peer_comps(&mut self, tick: u32, comps: [u64; NET_HASH_COMPONENTS]) -> Option<Action> {
        if self.peer_comps.is_none() {
            self.peer_comps = Some((tick, comps));
        }
        self.step()
    }

    /// The peer's `PT_HASHDETAIL` arrived (`component` is the wire's component index).
    pub fn on_peer_detail(&mut self, tick: u32, component: u8, items: Vec<u64>) -> Option<Action> {
        if self.peer_detail.is_none() {
            self.peer_detail = Some((tick, component as usize, items));
        }
        self.step()
    }

    /// The state machine: send our comps; when both sides' comps are known, name the first
    /// diverging component and send its items; when both sides' items are known, complete the
    /// diagnosis.
    fn step(&mut self) -> Option<Action> {
        let tick = self.desync?;
        if !self.comps_sent {
            if let Some(comps) = self.hist.comps_at(tick) {
                self.comps_sent = true;
                return Some(Action::SendComps(tick, comps));
            }
            return None; // the history does not cover the tick (older than ~600) — no report
        }
        if self.detail_component.is_none() {
            let (Some(local), Some((ptick, peer))) = (self.hist.comps_at(tick), self.peer_comps) else { return None };
            if ptick != tick {
                return None; // wait for the peer's comps for OUR desync tick
            }
            let Some(c) = first_diff_component(&local, &peer) else {
                self.finish(tick, local, peer, usize::MAX, Vec::new(), Vec::new());
                return None; // the components agree now (a late packet for an old desync)
            };
            self.detail_component = Some(c);
        }
        if !self.detail_sent {
            let c = self.detail_component.unwrap();
            if let Some(d) = self.hist.detail_at(tick) {
                let items = d[c].clone();
                self.detail_sent = true;
                return Some(Action::SendDetail(tick, c as u8, items));
            }
            return None; // no detail row for the tick (not an exchange tick on this machine)
        }
        if self.diagnosis.is_none() {
            let (Some(local), Some((ptick, peer))) = (self.hist.comps_at(tick), self.peer_comps) else { return None };
            if ptick != tick { return None; }
            let c = self.detail_component.unwrap();
            let Some(local_items) = self.hist.detail_at(tick).map(|d| d[c].clone()) else { return None };
            let Some((dtick, dcomp, peer_items)) = self.peer_detail.clone() else { return None };
            if dtick != tick || dcomp != c { return None; }
            let item = first_diff_item(&local_items, &peer_items);
            self.finish(tick, local, peer, c, local_items, peer_items);
            let _ = item;
        }
        None
    }

    fn finish(&mut self, tick: u32, local: [u64; NET_HASH_COMPONENTS], peer: [u64; NET_HASH_COMPONENTS], component: usize, local_items: Vec<u64>, peer_items: Vec<u64>) {
        if self.diagnosis.is_some() {
            return; // one diagnosis per desync
        }
        let item = if component == usize::MAX { None } else { first_diff_item(&local_items, &peer_items) };
        let component = component.min(NET_HASH_COMPONENTS - 1);
        self.diagnosis = Some(Diagnosis { tick, component, item, local_comps: local, peer_comps: peer, local_items, peer_items });
    }

    /// The combined hash of the history's view (what the session compared) — for the report.
    pub fn combined_at(&self, tick: u32) -> Option<u64> {
        self.hist.comps_at(tick).map(|c| fold_component_hashes(&c))
    }
}

/// The desync report's zip contents (the bug-report format: `report.txt` `key: value` lines,
/// `kind: desync`). `settings_payload` is the tag-1 match-start payload
/// (`mpnet::encode_match_start`'s bytes — settings + seed) so `--bug-replay` rebuilds the match;
/// `run_log` is the full input transcript since the match start (base tick first), written as
/// `match-log.bin`. `world_counts` names the world items' ranges (pickups, doors, triggers,
/// channels, destroy, breaks, ss, hurts) so the report can point at the exact element.
pub fn desync_report_files(
    diag: &Diagnosis,
    level: u32,
    settings_payload: &[u8],
    run_log_base: u32,
    run_log: &[[nf_net::Input; nf_net::MAX_AGENTS]],
    world_counts: [usize; 8],
    hist_text: &str,
) -> Vec<(String, Vec<u8>)> {
    let id = crate::bugreport::report_id();
    let item = diag.item.map(|i| i.to_string()).unwrap_or_else(|| "(the fold, no item)".into());
    let (local, peer) = (diag.local_comps, diag.peer_comps);
    let mut comps = String::from("component                    local             peer\n");
    for (i, name) in NET_HASH_COMPONENT_NAMES.iter().enumerate() {
        let mark = if local[i] != peer[i] { "  <-- first diverging" } else { "" };
        comps.push_str(&format!("{:<12} {:016x} {:016x}{}\n", name, local[i], peer[i], mark));
    }
    let counts = format!(
        "world items: pickups 0..{}, doors {}..{}, triggers {}..{}, channels {}..{}, destroy {}..{}, breaks {}..{}, ss {}..{}, hurts {}..{}\n",
        world_counts[0], world_counts[0], world_counts[0] + world_counts[1],
        world_counts[0] + world_counts[1], world_counts[0] + world_counts[1] + world_counts[2],
        world_counts[0] + world_counts[1] + world_counts[2],
        world_counts[0] + world_counts[1] + world_counts[2] + world_counts[3],
        world_counts[0] + world_counts[1] + world_counts[2] + world_counts[3],
        world_counts[0] + world_counts[1] + world_counts[2] + world_counts[3] + world_counts[4],
        world_counts[0] + world_counts[1] + world_counts[2] + world_counts[3] + world_counts[4],
        world_counts[0] + world_counts[1] + world_counts[2] + world_counts[3] + world_counts[4] + world_counts[5],
        world_counts[0] + world_counts[1] + world_counts[2] + world_counts[3] + world_counts[4] + world_counts[5],
        world_counts[0] + world_counts[1] + world_counts[2] + world_counts[3] + world_counts[4] + world_counts[5] + world_counts[6],
        world_counts[0] + world_counts[1] + world_counts[2] + world_counts[3] + world_counts[4] + world_counts[5] + world_counts[6],
        world_counts[0] + world_counts[1] + world_counts[2] + world_counts[3] + world_counts[4] + world_counts[5] + world_counts[6] + world_counts[7],
    );
    let report = format!(
        "kind: desync\nreport_id: {id}\nversion: {}\nos: {}\nlevel: {level:08x}\ndesync_tick: {}\nfirst_component: {}\nfirst_item: {item}\ncombined_local: {:016x}\ncombined_peer: {:016x}\nprotocol: nf-net 8 (PT_HASHCOMPS/PT_HASHDETAIL)\nreplay: nf-view --bug-replay <this zip>  (replays match-log.bin and names the first tick/component that differs from hash-history.txt)\n{counts}\n",
        crate::bugreport::version_line(), crate::bugreport::os_line(), diag.tick, diag.component_name(),
        fold_component_hashes(&local), fold_component_hashes(&peer),
    );
    vec![
        ("report.txt".into(), crate::bugreport::scrub(&report).into_bytes()),
        ("components.txt".into(), comps.into_bytes()),
        ("hash-history.txt".into(), hist_text.as_bytes().to_vec()),
        ("match-settings.bin".into(), settings_payload.to_vec()),
        ("match-log.bin".into(), encode_match_log(run_log_base, run_log)),
    ]
}

/// `match-log.bin`: first_tick u32, tick_count u32, then tick_count × MAX_AGENTS wire inputs —
/// the full transcript since the match start (the late-join run log's format at rest).
pub fn encode_match_log(base: u32, log: &[[nf_net::Input; nf_net::MAX_AGENTS]]) -> Vec<u8> {
    let mut o = Vec::with_capacity(8 + log.len() * nf_net::MAX_AGENTS * nf_net::input::INPUT_WIRE_SIZE);
    o.extend_from_slice(&base.to_le_bytes());
    o.extend_from_slice(&(log.len() as u32).to_le_bytes());
    for row in log {
        for i in row {
            let mut b = [0u8; nf_net::input::INPUT_WIRE_SIZE];
            i.encode(&mut b, 0);
            o.extend_from_slice(&b);
        }
    }
    o
}

/// Decodes `encode_match_log` (the `--bug-replay` side).
pub fn decode_match_log(b: &[u8]) -> Result<(u32, Vec<[nf_net::Input; nf_net::MAX_AGENTS]>), String> {
    let mut o = 0usize;
    let u32le = |o: &mut usize| -> Result<u32, String> {
        if b.len() < *o + 4 { return Err("match-log.bin truncated".into()); }
        let v = u32::from_le_bytes(b[*o..*o + 4].try_into().unwrap());
        *o += 4;
        Ok(v)
    };
    let base = u32le(&mut o)?;
    let count = u32le(&mut o)? as usize;
    let mut log = Vec::with_capacity(count);
    for _ in 0..count {
        let mut row = [nf_net::NEUTRAL; nf_net::MAX_AGENTS];
        for i in row.iter_mut() {
            if b.len() < o + nf_net::input::INPUT_WIRE_SIZE { return Err("match-log.bin truncated".into()); }
            let raw = &b[o..o + nf_net::input::INPUT_WIRE_SIZE];
            o += nf_net::input::INPUT_WIRE_SIZE;
            *i = nf_net::Input::decode(raw, 0).ok_or("match-log.bin: bad input")?;
        }
        log.push(row);
    }
    Ok((base, log))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn history_caps_at_600_and_reads_back() {
        let mut h = HashHistory::default();
        for t in 0..700u32 {
            h.push(t, [t as u64; NET_HASH_COMPONENTS]);
        }
        assert_eq!(h.entries.len(), HIST_CAP);
        assert_eq!(h.entries[0].0, 100, "the oldest 100 fell off the ring");
        assert_eq!(h.comps_at(699).unwrap()[3], 699);
        assert!(h.comps_at(99).is_none(), "tick 99 is out of the ring");
        // Sister: pushes for an already-recorded tick are refused (a stale re-push must not
        // shadow the first view of the tick).
        h.push(699, [0; NET_HASH_COMPONENTS]);
        assert_eq!(h.comps_at(699).unwrap()[3], 699);
    }

    #[test]
    fn first_diff_names_the_component_then_the_item() {
        let mut a = [1u64; NET_HASH_COMPONENTS];
        let mut b = [1u64; NET_HASH_COMPONENTS];
        assert_eq!(first_diff_component(&a, &b), None, "equal states name nothing");
        b[4] = 9;
        assert_eq!(first_diff_component(&a, &b), Some(4), "the match component is the first diff");
        // Sister: an earlier component differing wins (the fixed order decides).
        b[1] = 9;
        assert_eq!(first_diff_component(&a, &b), Some(1));
        assert_eq!(first_diff_item(&[5, 6, 7], &[5, 9, 7]), Some(1));
        assert_eq!(first_diff_item(&[5, 6], &[5, 6, 7]), Some(2), "a length diff is an item diff");
        assert_eq!(first_diff_item(&[5, 6], &[5, 6]), None);
    }

    #[test]
    fn watcher_completes_the_exchange_and_names_the_slot() {
        // Machine A (slot 0) and B run the same ticks; at tick 30 B's world door 2 diverges.
        let mut a = Watcher::default();
        let mut b = Watcher::default();
        for t in 0..=30u32 {
            let mut ca = [t as u64; NET_HASH_COMPONENTS];
            let mut cb = ca;
            if t >= 30 { cb[6] += 1; } // the world component differs from tick 30 on
            a.on_tick(t, ca);
            b.on_tick(t, cb);
            if t % 15 == 0 {
                let mut ia: Detail = std::array::from_fn(|_| vec![t as u64; 3]);
                let mut ib = ia.clone();
                if t >= 30 { ib[6][2] += 1; } // door 2's item hash differs
                a.on_detail_tick(t, ia.clone());
                b.on_detail_tick(t, ib);
            }
        }
        // The sessions report the desync at tick 30 (the next exchange tick after the poke).
        let act_a = a.on_desync(30).expect("A sends its comps");
        let act_b = b.on_desync(30).expect("B sends its comps");
        let Action::SendComps(ta, ca) = &act_a else { panic!("comps") };
        let Action::SendComps(tb, cb) = &act_b else { panic!("comps") };
        assert_eq!((*ta, *tb), (30, 30));
        // The packets cross; each completes the diagnosis off the peer's packets.
        let act_a2 = a.on_peer_comps(*tb, *cb).expect("A names the component and sends its items");
        let act_b2 = b.on_peer_comps(*ta, *ca).expect("B names the component and sends its items");
        let Action::SendDetail(tick_a, comp_a, items_a) = &act_a2 else { panic!("detail") };
        let Action::SendDetail(tick_b, comp_b, items_b) = &act_b2 else { panic!("detail") };
        assert_eq!((*tick_a, *comp_a), (30, 6), "the world component is the first diverging one");
        assert_eq!((*tick_b, *comp_b), (30, 6));
        assert!(a.on_peer_detail(*tick_b, *comp_b, items_b.clone()).is_none(), "A's diagnosis completes with no further sends");
        assert!(b.on_peer_detail(*tick_a, *comp_a, items_a.clone()).is_none());
        let da = a.diagnosis.as_ref().expect("A diagnosed");
        let db = b.diagnosis.as_ref().expect("B diagnosed");
        assert_eq!(da.component_name(), "world");
        assert_eq!(da.item, Some(2), "door 2 is the first differing world item");
        assert_eq!(db.component_name(), "world");
        assert_eq!(db.item, Some(2));
        // Sister: an unperturbed exchange names nothing (no report is built).
        let mut w = Watcher::default();
        for t in 0..=30u32 {
            w.on_tick(t, [t as u64; NET_HASH_COMPONENTS]);
            if t % 15 == 0 { w.on_detail_tick(t, std::array::from_fn(|_| vec![t as u64; 3])); }
        }
        w.on_desync(0); // never fires on an honest match — no desync event, no diagnosis
        assert!(w.diagnosis.is_none() && w.peer_comps.is_none());
    }

    #[test]
    fn the_match_log_round_trips() {
        let mut row: [nf_net::Input; nf_net::MAX_AGENTS] = Default::default();
        row[0].forward = 0.5;
        row[1].turn = -0.25;
        row[1].trigger = true;
        let log = vec![row; 3];
        let b = encode_match_log(7, &log);
        let (base, back) = decode_match_log(&b).expect("decodes");
        assert_eq!(base, 7);
        assert_eq!(back.len(), 3);
        assert_eq!(back[2][1].turn, -0.25);
        assert!(back[2][1].trigger);
        // Sister: a truncated log is refused, not half-read.
        assert!(decode_match_log(&b[..b.len() - 5]).is_err());
    }

    #[test]
    fn the_report_files_hold_the_diagnosis_and_the_log() {
        let diag = Diagnosis {
            tick: 45, component: 6, item: Some(2),
            local_comps: [1; NET_HASH_COMPONENTS], peer_comps: [1; NET_HASH_COMPONENTS],
            local_items: vec![1, 2, 3], peer_items: vec![1, 2, 9],
        };
        let row: [nf_net::Input; nf_net::MAX_AGENTS] = Default::default();
        let files = desync_report_files(&diag, 0x0700_0023, &[1, 2, 3], 0, &[row], [4, 2, 0, 0, 0, 0, 0, 0], "# tick\n");
        let get = |n: &str| files.iter().find(|(f, _)| f == n).map(|(_, b)| b.clone());
        let report = String::from_utf8(get("report.txt").unwrap()).unwrap();
        assert!(report.contains("kind: desync"), "{report}");
        assert!(report.contains("desync_tick: 45"));
        assert!(report.contains("first_component: world"));
        assert!(report.contains("first_item: 2"));
        assert!(report.contains("level: 07000023"));
        assert!(report.contains("doors 4..6"), "the report maps world items to names: doors 4..6");
        assert!(get("hash-history.txt").is_some());
        assert!(get("match-settings.bin").is_some());
        let (base, log) = decode_match_log(&get("match-log.bin").unwrap()).unwrap();
        assert_eq!((base, log.len()), (0, 1), "the report holds the full input transcript");
    }
}
