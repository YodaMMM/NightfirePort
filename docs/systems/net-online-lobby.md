# net-online-lobby — the online Host / Join front end

Status: implemented (front-end model + session trait + fake; the real UDP session is 051's).
Evidence tags: [M] code (function + address), [D] disc, [L] EE snapshot, [S] suggested / port-only design.

## 1. What this is

The PS2 game has no online mode — every multiplayer path is couch play on four pads [D: the join
page joins agents through per-pad messages, P_MPJOIN 0x20ca00 msg 0x6a; there is no network page in
the menu-def (08000002 of 07000048.bin, parsed 2026-10-15)]. Online play is a **port feature**. The
plan is deterministic lockstep over UDP: every machine runs the same MP match from the same seed and
settings and exchanges only per-tick input (the plan is the main agent's [S]). This task adds the
**front-end model** — the pages, the state, the flow, the getters the draw code calls — and puts the
network session behind a trait so the real transport can land underneath it (051, net-lockstep-core)
without touching the pages again.

Nothing here renders. All drawing is the main agent's.

## 2. The page flow

The game's local flow is untouched where it stands [M mp-menu-flow, crates/nf-view/src/menu.rs]:

    Main row 1 "Multiplayer" → 0x40000019 Join Game → 0x4000001a Select Scenario → 0x40000013
    Select Map → 0x40000051 Setup → 0x40000012 Options → 0x40000049 Confirm → the match
    [M C_GOMULTIPLAYER 0x216a78 → P_MPJOIN 0x20ca00 → P_MPSCENARIO 0x217348 → P_MPMAP 0x217468
    → P_MPSETUP 0x20d7a8 → P_MPOPTIONS 0x217678 → P_MPCONFIRM 0x20ec00].

The port inserts ONE choice page between Main and the game's Join Game page [S — port-only]:

    Main row 1 → PAGE_MP_MODE 0x5000_0001 (Local / Host Online / Join Online)
      row 0 Local        → 0x40000019 (the game's couch flow, byte-identical to before)
      row 1 Host Online  → PAGE_MP_HOST 0x5000_0002 (port number + Start Listening)
                         → 0x40000019 (the Join Game page AS the host lobby)
                         → the usual Scenario → Map → Setup → Options → Confirm flow
      row 2 Join Online  → PAGE_MP_NETJOIN 0x5000_0003 (typed IP:PORT, Connect)
                         → PAGE_MP_LOBBY 0x5000_0004 (the host's choices, read-only)
                         → the match (the host's Confirm hands both machines over)

The port-only page ids sit in a spare `0x5000_0000` block so they can never collide with the
game's `0x4000_00xx` page ids [S].

### 2.1 Host

- `PAGE_MP_HOST` has two rows: **Port** (default 27420 [S]) and **Start Listening**. ◀ / ▶ on the
  port row steps it ±1 (clamped 1024..65535); the mouse's value columns step it too (§4).
- Start Listening calls `MpSession::listen(port)` [S §3]. On success the front end arms host mode:
  the host's own panel (slot 0) joins automatically as the active codename (or "Player 1" — the
  game's template name, Txt 0x1c3 + slot [M C_RBMPCNAME 0x20d448]), state 5, ready, and the page
  becomes the game's own 0x40000019. Remote agents arrive into **panels 2–4** (slots 1–3) as the
  session reports them; their codenames show in the panels through the existing panel getters
  (mp_join — the draw code needs nothing new).
- While hosting, the host walks the game's own Scenario → Map → Setup → Options flow unchanged.
  Every choice change is published to the session (`MpNetChoices`, §3) on the next `tick()`.
- Confirm on the host's own panel (already ready) proceeds to Select Scenario — the game's ready
  arm routes through Menu_MPAreWeReady → 0x4000001a [M P_MPJOIN 0x20ca00's msg-0x50 arm]; while
  hosting the port skips the panel's own ready toggle (the host is ready by definition [S]).
- P_MPCONFIRM's confirm (0x20ec00:53-94) builds `MpSettings` and hands the match over [M — the
  port's `mp_confirm_start`]; while hosting it also calls `MpSession::start_match(&settings)`.
- Back anywhere on the host path cancels: `MpSession::close()`, host mode off, the panels reset,
  Back to the mode page [S].

### 2.2 Join

- `PAGE_MP_NETJOIN` is one text row: the typed address. The keyboard types **digits, dots, colon**
  (an IPv4 `IP:PORT` needs nothing else), Backspace deletes, Ctrl+C / Ctrl+V copy and paste the
  page's own clipboard buffer [S — the system clipboard needs a crate, see the report]. Length cap
  22 (`255.255.255.255:65535`). Confirm with an unusable address keeps the page and reports it.
- Confirm (Connect) calls `MpSession::connect(addr)` and the page shows **Connecting…**. The front
  end gives the session 600 ticks (10 s at the fixed 60 Hz [M the menu runs on the game's frame
  tick]) to connect; past that it shows **Connection timed out** and closes the attempt [S].
  A refused handshake shows **Connection refused** [S]. Back cancels and returns to the mode page.
- Connected: the page becomes `PAGE_MP_LOBBY` — the mirror of the host's lobby, read-only: the
  agent panels (the same mp_join mirror as the host's page) and the host's current choices as
  `mp_lobby_lines()` (the confirm-page line labels 0x3d4 Map / 0x3d5 Scenario / 0x24d Weapon Set /
  0x3d8-0x3d7 limit / 0x3d6 Duration / 0x3da Friendly Fire [M P_MPCONFIRM's enter builds these
  pairs; the lobby reuses the labels]). The lobby updates as the host changes things, until the
  host starts the match.
- Match start: the session's `match_started()` yields the host's `MpSettings`; the client runs the
  same handover the host's confirm page runs (mp_match_settings + requested_level + the fade push)
  [M the port's mp_confirm_start tail]. The client's copy keeps the settings as published; the
  host's copy has `master = 1` [M +0x180 = 1] and the client's carries what the session gave it.
- Back on the lobby leaves cleanly (close, panels reset, mode page).

## 3. The session trait (`MpSession`, in crates/nf-view/src/menu.rs)

The front end owns **no sockets**. Everything it needs from the network sits behind one trait [S]:

```rust
pub enum MpNetLink { Idle, Listening, Dialing, Connected, Refused, TimedOut, Lost }
pub enum MpNetError { BadAddress, Busy, Denied }
pub struct MpNetAgent { pub slot: usize, pub codename: String, pub ready: bool }
pub struct MpNetChoices { scenario, level, weapon_set, radar, friendly_fire,
                          frag_limit, time_limit, player_mods: [u32; 2], envmods: [u32; 5] }
pub trait MpSession {
    fn poll(&mut self);                        // drive the transport once per menu tick
    fn link(&self) -> MpNetLink;
    fn listen(&mut self, port: u16) -> Result<(), MpNetError>;
    fn connect(&mut self, addr: &str) -> Result<(), MpNetError>;
    fn agents(&self) -> &[MpNetAgent];         // slot 0 = this machine's agent, 1..3 = remote
    fn host_choices(&self) -> Option<MpNetChoices>;   // the host's lobby choices (client side)
    fn publish_choices(&mut self, c: &MpNetChoices);  // the host pushes its choices
    fn start_match(&mut self, s: &nf_game::mp::MpSettings);
    fn match_started(&self) -> Option<nf_game::mp::MpSettings>;
    fn set_codename(&mut self, name: &str);
    fn close(&mut self);
}
```

Why a trait and not a crate: 051 (net-lockstep-core, in flight on `origin/s/net-lockstep-core`)
owns the wire (deterministic lockstep over `std::net::UdpSocket`, no crate downloads). Building the
crate twice in parallel would collide; the front end needs only the surface above. When 051 lands,
its session implements this trait (or a ≤30-line adapter wraps it); `FrontEnd::mp_session` takes
any `Box<dyn MpSession>`. **nf-net was NOT created** — the trait lives beside its only consumer.

The front end mirrors the session into the game's own structures (mp_join slots), so the existing
join-page getters, the start checks (C_SBMPOPTIONS 0x20e818:50-130's team/FFA gates) and the
confirm-page line builder all work over network agents unchanged [M].

Deviations from the couch flow (all [S], all for the report):
- Remote agents arrive joined + ready; the game's per-agent codename/team/skin picks at
  0x40000019/0x40000051 are the LOCAL agent's picks. Clients do not pick team/character in this
  first cut — the host's published `MpSettings.players[slot]` carries the defaults for remote slots
  (team 0, skin 0). A later task can carry per-client picks through the session.
- The client does not run the game's start checks; the host's confirm gate is the single gate.

## 4. Mouse support (the PC-mouse pattern)

`mouse_move` hovers via `list_rows_y()`; the new pages add their hit rows (640×480 space, the draw
contract): MpMode 138/164/190, MpHost 138/164, MpNetJoin 251 — the pages draw through
`page_rows_no_head`, whose rows sit at y 140+26k, and the Join page's one drawn row is the address
value label at (200,250,240×18), middle y 259 [M menu_overlay's MpMode / MpHost / MpNetJoin arms;
menu-mouse-parity moved the hits onto the drawn rows — the old 143/175/207/160 predate the
stand-in draw]. `mouse_click` treats every MpMode row, the MpHost Start row and the MpNetJoin
address row as action rows (a click confirms); the MpHost port row keeps the ◀ ▶ value columns
(x 340..400 left, 400..600 right — the option pages' own columns [M mouse_click's value columns]).

## 5. Tests

`crates/nf-view/tests/mp_net_lobby.rs` — headless (no ISO), numbers/ids only, each with a sister
assertion that can fail. A deterministic `FakeSession` (no threads, no sockets — the test advises
the transitions) implements `MpSession`:
- host side: mode → host page (port default 27420) → step → listen (the fake records the port) →
  the Join page with the host's own panel joined → a remote agent arrives in panel 2 with its
  codename → the whole Scenario → Map → Setup → Options → Confirm walk with the choices published
  to the fake → start_match settings handed over;
- join side: typed address (digits/dots/colon, Backspace, paste) → Connect → Connecting… →
  connected → the lobby mirrors the host's choices → match start hands over;
- refused / timed-out / Back-cancels-cleanly on both sides, page ids of the port-only pages.

## 6. The one-screen lobby (mp-lobby-one-screen, 2026-10-18)

The user: "we need a custom lobby screen where all the settings can be set on one screen." The
online path stops walking the couch pages; it gets **one page** that holds every MpSettings field
the couch pages set, the per-player rows and the Start. The LOCAL couch flow is untouched — every
couch page, handler and test stays exactly as it is [M the couch pages are the game's own; see §2
for the walk they keep].

Evidence note: the couch map wheel is NOT filtered per scenario — nothing in P_MPMAP 0x217468 or
C_SBMPMAP 0x217538 reads MPSettings+0x1A4 [M P_MPMAP 0x217468:19 selects the current
`MPSettings._424_4_` in the 0x2df400 wheel, C_SBMPMAP 0x217538 only locks/unlocks + confirms;
docs/systems/mp-menus.md states it]. The lobby's Map row therefore offers all 8 rows at every
scenario; the only map-dependent gate the game has is Ravine's AI-Bots lock [M P_MPOPTIONS
0x217678:20-25], which the lobby carries (below).

### 6.1 The page

`PAGE_MP_ONLINE_LOBBY 0x5000_0005` (`Page::MpOnlineLobby`) — a spare id next to the §2 block, so
the game's `0x4000_00xx` and the old port-only ids stay untouched [S]. Both roles share the page:

    Main row 1 → PAGE_MP_MODE
      row 1 Host Online → PAGE_MP_HOST (port + Start Listening, unchanged §2.1)
                        → PAGE_MP_ONLINE_LOBBY (everything on one screen; host edits all)
      row 2 Join Online → PAGE_MP_NETJOIN (address, unchanged §2.2)
                        → PAGE_MP_ONLINE_LOBBY (the same screen; the client edits only its own row)

The old walk (Join Game as the host lobby → Scenario → Map → Setup → Options → Confirm) and the
old client mirror page (PAGE_MP_LOBBY §2.2) are no longer on the online path [S — this task
replaces it]; their code and tests stay for the couch flow's sake.

### 6.2 The model (`MpLobby`, crates/nf-view/src/mp_lobby.rs)

One struct owns the screen [S]; every field cites the couch page that sets the same word [M]:

- Scenario = MPSettings+0x1A4, the 13 MP_SCENARIO_ROWS words [M C_SBMPSCEN's confirm store;
  locked rows (the reward column) refuse edits, the game's own confirm refuses them at
  C_SBMPSCEN 0x20c600:40-42 — the port maps the refusal onto the ◀ ▶ step [S]].
- Map = MPSettings+0x1A8, the 8 MP_MAP_ROWS codes, not filtered (see the evidence note) [M].
- Weapon Set = +0x1B4, 0..10 wrapping [M P_MPPLAYERMODS 0x2104d0:13, 26-49].
- Limit = +0x1BC... no: +0x19C, the MP_RULES_POINTS list [M P_MPRULES 0x20f790:45-76]; the row's
  unit is "Lives" for scenario 0x10 else "Points" [M 0x20f790:78-95].
- Duration = +0x1A0, the MP_RULES_DURATIONS list [M 0x20f790:21-42].
- Friendly Fire = +0x198, On/Off [M P_MPPLAYERMODS 0x2104d0:12]; Radar = +0x1C4 [M :16];
  the two further Player-Mods rows = +0x1BC / +0x1C8, On/Off [M :14-15].
- Enviro-Mods = the five rows in page order (view distance 0/1/2 [M P_MPENVIROMODS 0x210030:12-16],
  +0x1B8 On/Off, Explosive Scenery with the reward gate — locked shows the single 0x10 node and
  steps do nothing [M :35-61 + Menu_UnlockMPSettings 0x1ff698], +0x1D0 On/Off, vehicles 0..3).
- Bots = the four mpbots records [M mpbots 0x2deea8, layout in MpBot]: per bot an Enabled toggle
  (the Bot Setup page's own Yes/No row [M P_MPBOTSETUP 0x211270 row 0]), the character wheel
  (MP_CHARACTERS, locked refuse [M C_SBMPBTCHOOSE 0x210f08:18-20]; a pick applies the preset stats
  and side [M :26-43]) and a Skill row (Poor 8 / Average 5 / Good 3 / Very Good 1 [M 0x211270:49-59]).
- Per-player rows ×4: codename (display), character (the MP_CHARACTERS wheel), team (only when the
  scenario has the team bit 0x2000_0000 [M P_MPSETUP 0x20d7a8's state-2 arm], 0 Phoenix / 1 MI6
  [M C_RBMPSETUP 0x20de58]), ready flag, ping (display, the session's RTT [S]).
- Start row: host-only. Enabled when every joined human is ready and at least one human is in
  [S — the task's gate; the game's own ready gate is per-panel Menu_MPAreWeReady 0x1f78d8]. Press
  it and the lobby runs the game's own start checks [M C_SBMPOPTIONS 0x20e818:50-130 — the
  FFA good-side cap, the both-sides team gate, the two-player minimum] and then builds the
  MpSettings exactly as the couch confirm builds it [M P_MPCONFIRM 0x20ec00:53-94 — the port
  copies the model back into the front end's mp_* fields and calls mp_confirm_start, so the built
  struct is the same by construction].

Navigation is the pad pattern: Up/Down move the focus over the row list, ◀ ▶ step the focused
row's value, Confirm acts (toggle / start), Back cancels the lobby (session close, panels reset,
mode page [S §2]). The rows the getters hand out carry `editable` — the host's rows are editable
only for the host; a player row is editable only by that row's owner [S]. Mouse support follows
§4's contract (the draw code maps the hit rows; the model only consumes the mapped actions).

### 6.3 Getters (what the draw code calls)

- `MpLobby::rows(&self) -> Vec<MpLobbyRow>` — the row list in screen order: label (the game's Txt
  id where one exists — the confirm page's own pair ids 0x3d4/0x3d5/0x24d/0x3d8-0x3d7/0x3d6/0x3da
  [M P_MPCONFIRM's enter builds these; the rest are the couch pages' row labels], port-only rows
  carry the port's literal [S]), value (Txt id or text), `editable`, `focused`.
- `MpLobby::players(&self) -> &[MpLobbyPlayer; 4]` — the per-player rows (codename, character,
  team, ready, ping).
- `MpLobby::start_enabled(&self) -> bool` — the Start gate.
- `FrontEnd::mp_lobby_rows()` / `mp_lobby_players()` / `mp_lobby_start_enabled()` — the
  front-end delegates (None/empty off the lobby page).

### 6.4 Session messages

The lobby state crosses the session as a tagged DATA tunnel blob, exactly like the §3 choices
tunnel (`crates/nf-view/src/mpnet.rs`'s encode/decode over `nf_net::Packet::Data`). The codec —
`LobbyState` (host→clients, the full snapshot) and `LobbyEdit` (client→host, one own-row edit) —
lives in `crates/nf-net/src/lobby.rs` [S, numbers/ids only: u32 words, u8 flags, 32-byte
codenames; the layout is its own doc comment]. The `MpSession` trait gains four default methods
(`lobby_state` / `publish_lobby` / `push_row_edit` / `take_row_edit`) so existing impls compile
unchanged; `SessionProxy` implements them over the tunnel [S]. Host edits publish on the next
tick (the §3 publish-diff pattern); a client's edit rides to the host, merges into the host's
model, and reaches every other client with the next publish.

### 6.5 Tests

`crates/nf-view/tests/mp_lobby_one_screen.rs` — headless loopback (no ISO, no sockets), numbers
and ids only, every test with a sister assertion that can fail:
- host edit → the client's rows show it on the next tick (the publish-diff path);
- the client cannot edit host rows (the editable matrix, both the getter and the action);
- Start is gated on ready (a not-ready client holds Start; readying through the client's own row
  opens it; the start hands the match over the session to the client);
- the built MpSettings equals what the couch pages build for the same choices [M mp_confirm_start]
  — the same words through both paths produce equal structs;
- the `LobbyState` / `LobbyEdit` codec round-trips.

## 7. The failure paths (mp-online-failures, 2026-10-18) [S — port-only design]

The PS2 game has no online mode, so every rule here is the port's own [S]; the one game fact
leaned on is the dead-pad NEUTRAL a dropped slot rides (`psiInput_PollDevices` 0x10ca78 [M],
net-lockstep.md "Disconnects").

| Path | Before | Now | Test |
|---|---|---|---|
| A client leaves mid-match | The pause page's QUIT never closed the session: the leaver kept sending keepalives, so nobody timed it out and every other machine stalled on its missing inputs **forever**. A clean LEAVE desynced any client that was behind the host (it ran the leaver's last ticks NEUTRAL, the host had run them with its real input) | QUIT closes the session (`App::net_leave_match`, LEAVE at once); a dropped core sends LEAVE (`impl Drop for NetCore` — the window closing). The host keeps relaying the leaver's REAL inputs for every tick it ran (`left_at`, the drop tick) and a client takes every other slot from the host's relay — never its own guess | `drop_rejoin::a_client_leaving_mid_match_never_desyncs_the_others`, `…_crashing_…` (both FAIL on the old session: desync at tick 150) |
| The leaver comes back | Late join to the lowest free slot | A HELLO whose (non-empty) codename matches a slot's departed human takes THAT slot (its body still stands → the reconnect path of `Sim::mp_join_spawn`, no second body) | `drop_rejoin::a_reconnect_takes_its_own_slot_back_and_plays_on_in_sync`, `mp_online_failures::join_in_progress_from_the_menus_then_leave_and_reconnect` [ISO] |
| The host leaves mid-match | Clients logged "player 1 left" and froze; a host that quit to its menus kept its keepalives flowing, so the 5 s `HostTimeout` never fired; when it did, `net_fail` opened the couch Join Game page over the frozen arena | The host's LEAVE is `Event::HostLeft` → `CoreEvent::HostLeft` at once; `net_fail` lands on the page this machine came from (`FrontEnd::mp_net_return`: a client's Join Online with the address kept, a host's port page with Start focused) with the reason in the notice box, and the arena unloads (the debriefing's front-end rebuild) | `mp_online_failures::the_host_leaving_mid_match_reaches_every_client_at_once` |
| The host backs out of the lobby | The client sat on the lobby until the 5 s timeout | The client is back on Join Online within the link latency, "The host left the lobby" | `mp_online_failures::the_host_leaving_the_lobby_returns_the_client_at_once` |
| Join a running match from the menus | The dial reached the late-join handover but the page went to the one-screen lobby ("Waiting for the host…") forever: the handover event sat in the lobby core nobody drained | `MpNetState::Joining` ("Joining the match in progress…", only Back acts); the app adopts the lobby session as `mpnet::LobbyAdopt::LateJoin` (no arena load, no load hold), runs the handover's replay, confirms and closes the front end — the flag path's late join | `mp_online_failures::join_in_progress_from_the_menus_then_leave_and_reconnect` [ISO] |
| Rematch, a client presses Continue before the host | The re-dial hit the host's finished match (still open under its debriefing) and LATE-JOINED it — a dead lobby | `NetCore::finish_match` (`mp_results`): the finished session answers no new HELLO; the client's HELLO retries until the host's next lobby listens on the port | `mp_online_failures::three_machines_rematch_through_the_lobby` (host + 2 clients; the clients press Continue first, then the second match starts) |
| Back on the lobby, then host the same port | "Port already in use": the closed session kept its socket until the whole core dropped (the app keeps it in `net_lobby`) | `NetCore::close` swaps in an ephemeral loopback placeholder (the port is free when it returns); `rebind_host` releases the old socket before binding | `mp_online_failures::back_releases_the_socket_and_the_same_port_rehosts_at_once` |

The app glue (`crates/nf-view/src/lib.rs`): `adopt_lobby_session` (the `LobbyAdopt` decision; a
late join cancelled with Back drops the adopted core), `ensure_lobby_session` (the online pages'
proxy — not while a session is adopted, and wired at once by `net_fail`), `net_leave_match`,
`net_fail`, the `HostLeft` arm, the `LateJoinHandover` arm closing the join page.
