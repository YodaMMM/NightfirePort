# net-match-end — every machine reaches the debriefing

Status: implemented (ack protocol + the host's end wait + tests). Evidence tags: [M] code
(function + address), [D] disc, [L] EE snapshot, [S] suggested / port-only design.

## 1. The symptom (the soak, 2026-10-19)

Two-window checks on one PC (a 90 s match with 3 bots; a 30-minute 16-slot match): the HOST
reached the time limit, logged `mp: match over — the debriefing handover (16 rows)` and opened the
debriefing. The JOINER's last logged sim event was frame 107653 of the 108000-tick match; it then
logged `[net] The host left the match` and went back to the front end with no debriefing. Both
runs ended this way: **a client never gets the end-of-match screen** — the host tears the session
down at its own match end, before the clients have the last ticks' inputs.

## 2. The game's match-end machine

The match-end latches live in `MP_Update`'s end check [M `check_for_end_condition` port of
0x184268: the time-up latch 0xfe at :142-143, the frag latch 0xfd at :170-172, the final dispatch
:173-184 — state 1 (frag) else state 2 (time)]. The state machine walks 1 → 3 → 4 → 5; case 4 is
the handover [M `MP_Update` 0x184bd0:62-78]: `GameState+0x1C = 0x40000033` (P_MPDEBRIEFING — the
page only QUEUES [M the handler table 0x2029e0:434-437; G-048(3)]), `ResetMap_LevelToLoad`
(0x7000048, the front-end map), the fade push, the per-slot `Player_SetCamMode`/`GS_PausePlayer`.
The page opens at the load's `MenuManager_Create` [M 0x1d6360:160-163], and the board is
`P_MPDEBRIEFING_Handler` 0x212198 msg 0x4c's rows/ranks/result [M; the port's data builder is
`MpMatch::debriefing`].

In couch play all four "machines" are one console — the handover runs once. Online is a port
feature [S net-match-host.md §1 — the PS2 game has no online mode], so every machine runs its own
sim and must each reach the handover from its own final tick.

## 3. The bug

The app's handover (`App::mp_results`, crates/nf-view/src/lib.rs) moved the net session out of
`self.net` into `net_finished` at the HOST's own match end. The finished session polled only
while the front end sat on the debriefing page and CLOSED (a real `LEAVE`, slot 0) the moment the
page went away. A client still draining the last few hundred ticks of the input pipeline lost the
host's relay and then saw the leave: `Packet::Leave { slot: 0 }` fires `Event::HostLeft`
[M nf-net session.rs — the slot-0 LEAVE arm] → `net_fail("The host left the match")` → the front
end. No debriefing.

The session layer itself always supported the wait: the host's `relay_horizon` [M nf-net
session.rs] commits every tick whose inputs are all known and relays it from `poll()` alone — a
host that has stopped ticking its sim but keeps polling still feeds its clients every remaining
tick. The app just stopped polling the live session.

## 4. The fix (port protocol; [S] throughout — no online exists on the disc)

1. **The debriefing ack** — a new app-to-app DATA blob, tag 4
   (`MpSession::send_debrief_ack`): `[4, final_tick u32 LE, board_hash u64 LE]`. A machine sends
   it when its own handover runs (`App::mp_results`), before anything happens to its session.
   The receiver surfaces it as `CoreEvent::DebriefAck { slot, final_tick, board_hash }`.
2. **The board fold** — `nf_game::mp::debrief::board_hash(&MpDebriefing) -> u64`, FNV-1a over the
   numbers (slot, rank, character, points, victories, deaths, total, the result shape). The name
   cells are excluded: the wire's 208-byte MatchSettings carries no name bytes, so a client's own
   copy shows the empty-name rows (the same reason net_e2e_loopback_live pins the numbers).
3. **The host's end wait** — at the host's handover, with human peers still out, the session
   STAYS live (`self.net`) and the frame loop enters the wait: poll every frame (the relay keeps
   feeding the clients), collect `DebriefAck`s and departures, and compare each ack's board hash
   with the host's own (a mismatch logs and shows on the debriefing's notice line). The wait
   completes when every peer has acked or departed — or at a wall-clock deadline (15 s), so a
   dead peer can't hold the host. Only then does the host run the old handover tail
   (`net_finished = net.take()` + `finish_match()`).
4. **The leave paths.**
   - *A client quits during the debriefing*: its finished session closes when its debriefing page
     goes away — the `LEAVE` reaches the host (mid-wait: the wait counts the peer done and can
     complete; after the wait: the finished session's poll absorbs it). The host's debriefing
     shows the notice [S].
   - *The host quits during the debriefing*: its session closes — every client sees the slot-0
     `LEAVE` → `HostLeft` → `net_fail("The host left the match")` — a real departure, unchanged.
   - *A peer really leaves mid-match*: unchanged — the host relays the departure, the rest run on
     with NEUTRAL inputs (`DropPolicy::Neutral`), a slot-0 leave is a `HostLeft`.

## 5. Tests (crates/nf-view/tests/net_match_end_clients_live.rs)

The real session on loopback (ports per binary), `net_rng` set per slot, `nf_view::net_tick` on
every machine — the app's match-end policy modelled in the harness the way
`net_e2e_loopback_live` models the handover:

1. `three_machines_reach_the_debriefing_with_identical_boards` — host + 2 clients, the Duration
   row stepped to 1 minute (MP_RULES_DURATIONS[0] [M P_MPRULES 0x20f790:21-42]), all three sims
   run to their own final tick; the host stops ticking first, keeps polling, and completes its
   end wait on both acks. All three land on the debriefing with identical boards (numbers), the
   fold hashes agree, and no `ClientLeft`/`HostLeft`/`HostTimeout` fires on any machine.
2. Sister: `a_host_departure_mid_match_is_still_host_left` — the host's session closes mid-match;
   the client's session fires `CoreEvent::HostLeft` (the message path) and the input relay stops.
3. Sister: `a_client_departure_mid_match_still_runs_the_match_on` — the client's session closes
   mid-match; the host fires `CoreEvent::ClientLeft` (graceful) and the match runs on (the ticks
   after the departure are ready with the departed slot NEUTRAL).
4. Sister 2: `the_frag_limit_end_reaches_the_same_debriefing` — the same 3-machine match ended by
   the frag limit (scripted kills through `MpMatch::player_killed`, the game's scoring path [M
   0x188f70]) reaches the debriefing with identical boards and no leave events.
