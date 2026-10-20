# net-lobby-list — the Join Online page's list of open games (task 700v3)

Evidence tags: [S] port-only design (the PS2 game has no online mode [D], net-online-lobby.md §1), [L] a live run on
the user's PC. Code: `crates/nf-net/src/discovery.rs`, the app glue in `crates/nf-view/src/mpnet.rs`, the page in
`crates/nf-view/src/menu.rs` (`MpNetJoin`).

## 1. Wire [S]

| packet | layout |
|---|---|
| QUERY | `NFLQ` + u32 nonce + u32 the joiner's ms clock |
| LOBBY | `NFLB` + the 8 echoed bytes + u16 protocol + u32 build + u16 game port + u32 level code + u32 scenario + u8 players + u8 max + u8 in-progress + name (u8 len, ≤ 32 B) + host codename (u8 len, ≤ 20 B) |

Neither starts with a session header the decoder accepts, so both are stray datagrams for the session
(`Session::take_stray`).

## 2. Flow [S]

- **Host:** no extra socket. Every frame the host's core answers each QUERY that reached its GAME socket with its
  LOBBY record (`discovery::answer`), so any number of hosts can run on one PC (each on its own game port). The
  front end refreshes the record each tick (`MpSession::set_lobby`: "<codename>'s game", the arena and scenario from
  the published choices, the joined panels); `in_progress` follows the match start.
- **Joiner:** while Join Online is up and nothing is dialing, a `LobbyList` bound on the LAN interface (not 0.0.0.0 —
  Windows sends a 0.0.0.0 broadcast out whichever adapter routes first, the Hyper-V/WSL lesson of net-p2p) queries
  every 1.5 s: the subnet broadcast, 255.255.255.255, this PC's own LAN address and 127.0.0.1, each on the scan ports
  27420-27423. Replies are keyed by sender-IP:game-port (one host answers once per target that reached it — the best
  ping is kept). A row not refreshed for 5 s expires.
- **Page:** cursor 0 is the typed address/code row, cursors 1.. the rows (name, arena, players/max, ping; "other
  version" rows are listed but refused; "in progress" rows dial and the host refuses until 700v5). ✕ on a row dials
  its address.

## 3. Internet listing

The same `LobbyInfo` record is what the optional home-server directory (700v4 item 5) will serve; its address is a
setting, default empty, never hard-coded.

## 4. Checks

`crates/nf-net/tests/lobby_list.rs` (loopback: fields + ping, another build not joinable, expiry, two hosts on one PC,
join from a row) and the unit round trip in `discovery.rs`. [L 2026-10-18] a `--net-host 27422` window was found by
the LAN scan at 192.168.1.212:27422 (the real broadcast path).
