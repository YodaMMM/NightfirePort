# net-soak-faults — the long faulty-network soak

Status: **in progress** (2026-10-19). The soak
exists and runs; phase 0 is verified end-to-end; phase 1 uncovered a real
readiness deadlock that is not yet root-caused (see §4).

## 1. What the soak is

A 16-participant match on the REAL session + transport — raw
`Session<UdpTransport>` pairs over `127.0.0.1` UDP (the `net_e2e_loopback_live`
session shape, no menu), with nf-net's deterministic fault injection
(`UdpTransport::inject_link_fault` / `inject_link_delay`, SplitMix64 on the
send path [M net.rs Fault]):

- **4 machines**: the host (session slot 0), one plain client (slot 1, the
  host's lowest-free assignment [M session.rs join]), and two late joiners
  that reclaim slots **5** and **11** through the codename-reclaim path
  [M session.rs `begin_late_join`: a reconnect with a departed human's
  codename takes that slot back]. The reclaim handles are made by 10 pad
  sessions that join slots in order and leave before the match arms — a pad
  that left before the next dialed would hand the freed lowest slot to it
  (the freed slot is reused; the test asserts every pad's slot).
- **12 bots** (`bot_count = 12`, humans 0..3, bots 4..15 [S mp-16-slots] —
  the port's layout) on both sims.
- **One phased match** (8 phases; the suite runs 10 game-minutes = 36,000
  ticks; `NF_SOAK_LONG=1` runs 30 game-minutes = 108,000):
  clean → 5% loss + 8% reorder (every machine's outgoing half) → 20% loss +
  8% reorder → 150 ms one-way ± 80 ms jitter → a 2 s total stall of slot 5's
  sends then recovery → slot 11 leaves and rejoins (the late-join path) →
  the host's link at 20% loss → clean to the time-limit end
  (`frag_limit = -1`, the time limit is the only ender).

Every phase asserts: no `Event::Desync` on any machine, the two simmed
machines' per-tick hashes agree (`net_tick`'s hash — the full real path:
`step_bond` + the bots inside + `step_net_combat`), and the match reaches its
end (both sims' `game.state == 5` + `request_debriefing`). Slots 5/11 run
session-only — a small machine cannot hold four full arena sims (2 loads = 1.75
GB fit, 4 = OOM-killed [D mp_16_slots_live.rs:119-127]) — they still feed
real inputs every tick and gate the lockstep (the host requires every human
slot's input [M session.rs `ready_inputs`]); the sim-hash agreement is
asserted on the simmed pair.

Per phase the test prints each machine's **sent/received bytes/s** (the
session's payload counters — `sent_totals()` [M session.rs] and the new
`recv_totals()`) and the **worst input-wait stall** per machine (the logical
ms a machine spent waiting for the row) plus the count of ticks that waited
>100 ms and the session-level `Stalled` events.

## 2. Additions to nf-net

- `UdpTransport::stall_sends_for(ms)` + `Session::inject_send_stall(ms)`
  [S net-soak-faults]: swallow every send for `ms` of the session's logical
  clock (the stalled datagrams are LOST — the recovery is the protocol's own
  retransmit, which is what the soak proves). Field `stall_until_ms` on
  `Fault`, own line (merge rule #4).
- `Session::recv_totals() -> (u64, u64)` [S net-soak-faults]: the receive-side
  payload counter, bumped on every datagram `poll` drains (the same
  accounting as `sent_totals`).
- Tests: `crates/nf-net/tests/send_stall.rs` — a 2 s total uplink stall on
  one machine stalls the lockstep (the never-guess rule) and then RECOVERS
  through the unacked-window retransmit; equal hashes to tick 400, no
  desync, both counters alive; sister: the clean run never blocks. 2/2 green.

## 3. Evidence tags

- [M] session.rs `begin_late_join` (the codename reclaim), `drop_slot`
  (a pre-match departure sets `left_at = start_tick` — forever-NEUTRAL, no
  poison), `ready_inputs`/`tick_input` (the never-guess rule + the host's
  NEUTRAL materialization), the client Input decoder (protocol 5: a slot the
  relay left out is NEUTRAL), `plan_inputs`/`build_input_range` (≤64 ticks
  and ≤ MAX_PACKET per packet; the RTO retransmit from the oldest unacked),
  `relay_horizon` (the walk stops at the first tick with a missing human
  input).
- [S] the stall primitive, the recv counter, the soak's slot choreography
  and phase script (port choices; the faults are synthetic).
- [D] the memory figures are the measured ones recorded in
  mp_16_slots_live.rs.

## 4. Open defect: a rare readiness deadlock under 5% loss + reorder (phase 1)

Symptom (two runs, `--nocapture`): ticks 4500+ (the first fault phase) —
the host's `ready_inputs(t)` stays None for the fault phase's wall budget
(5 s / 30 s in the two runs) with the host's missing slots **[1, 5, 11]** —
every client — while the logical clock ran ~108,000 ms past the hang with no
`HostTimeout` and no recovery.

Ruled out by minimal repros over real UDP with the SAME fault recipe
(all green, `crates/nf-net/tests/`):

- `loss_reorder_repro.rs` — 2 machines, faults at tick 100, 400 ticks: PASS.
- `loss_reorder_repro4.rs` — 4 machines (host + 3 clients), faults at tick
  100, 600 ticks: PASS.
- `loss_reorder_repro5.rs` — the soak's full session shape (4 machines + 10
  join-and-leave pads + two late joiners reclaiming slots 5/11 mid-match +
  `record_run_tick` per tick), faults at tick 100, 600 ticks: PASS.

Also verified by reading [M]: packets are size-bounded (≤64 ticks,
≤ MAX_PACKET 1400 B — no EMSGSIZE silent drop), the retransmit clock
(`progress_ms`) only resets on ack advance or a retransmit, the client
materializes relay-absent slots as NEUTRAL, a pre-match pad departure cannot
poison `host_slot_required`.

The trigger therefore involves the full app shape the repros lack: the two
full arena sims running `net_tick` in the pump loop (real CPU work between
polls), the 4500-tick clean lead-in before the faults, and the real input
rows (fire/turn/look). Next steps for the continuation:

1. Instrument the hang: on the pump deadline, dump `host.peer_acks()`,
   `peer_rtts()`, each client's own next tick, and `relay_horizon`'s stop
   tick — that pinpoints whether the hole is on the send side (a client's
   input never re-sent) or the relay side (the horizon frozen on an earlier
   hole than `t`).
2. Bisect the sims: run the soak's session shape with the sims' `net_tick`
   replaced by the movement-only `machine_tick` (mp_16_slots' shape).
3. The deterministic-fault caveat: the fault dice apply per SEND in poll
   order, so UDP arrival timing makes runs non-deterministic — the repros
   passing does not clear the protocol; they only bound where the bug lives.
