# net-short-codes — 4-character join codes through the player-set lobby server

*All of this page is a port feature [S]: the PS2 game has no online mode (docs/systems/net-lobby-list.md,
docs/systems/net-p2p.md). The only game facts involved are the lobby pages themselves, already covered by
those docs. The goal (2026-10-19): 4-character codes instead of the long code, through a home server set by
players.*

The long self-contained join code (docs/systems/net-p2p.md §2, 16 Crockford base32 symbols, `H0K7-RRBB-3G40-0612`)
stays exactly as it is and remains the fallback. When the player has set a lobby server
(`lobby_server = host:port` in `%APPDATA%\NightfirePort\net.txt` or `NF_LOBBY_SERVER`; the address is never built
into the exe), hosts get a **4-character code** from that server and joiners can type either form.

## 1. The short code

- Alphabet `SHORT_CODE_ALPHABET` (`directory.rs`): `23456789ABCDEFGHJKMNPQRSTVWXYZ` — digits 2-9 plus the
  Crockford letters minus I/L/O/U, 30 symbols; 30^4 = 810 000 codes [S]. No 0/O and no 1/I/L can be confused
  (U is excluded for V, as in Crockford base32 — the long code's alphabet, `joincode.rs`).
- 4 symbols, shown bare (`ABCD`, no dash).
- The server assigns it to a REGISTER's entry, unique among the live entries, stable while the host keeps
  refreshing (the entry is keyed by the source address, directory.rs), expiring with the entry (`EXPIRE_MS`).
  Generation: a std-only xorshift seeded from the clock + `RandomState` [S]; collisions are re-drawn.

## 2. Wire (all little-endian like the rest of `directory.rs`; version byte `DIR_PROTOCOL = 1`) [S]

| message | magic | layout |
|---|---|---|
| REGISTER v2 (host→server) | `NFDR` | the old packet (`NFDR` + `LobbyInfo` record) **+ tail**: `u8 ver=1`, `u32 nonce`, `u32 lan ip` (0 = unknown), `u16 lan port` |
| REGISTER ack (server→host) | `NFDA` | `u32 nonce`, 4 ASCII code bytes |
| LOOKUP (joiner→server) | `NFDS` | `u8 ver=1`, `u32 nonce`, 4 ASCII code bytes |
| FOUND (server→joiner) | `NFDT` | `u32 nonce`, `u32 public ip`, `u16 public port`, `u32 lan ip` (0 = none), `u16 lan port`, the `LobbyInfo` record |
| negative (server→client) | `NFDN` | `u32 nonce`, `u8 reason` — 0 code not found, 1 rate limited, 2 unsupported version |
| LIST / ENTRIES | `NFDL` / `NFDE` | unchanged |

Versioning and compatibility:
- The v2 tail rides **after** the record, so an old `nf-lobbyd` still lists a new host (the record decodes
  unchanged; the tail is ignored, no reply is expected). An old host (no tail) registering on a new server is
  registered as today and gets no reply. A new server that receives a request whose version it does not know
  answers `NFDN` reason 2 ("unsupported") cleanly. A pre-versioning server answers nothing at all, so clients
  treat silence as "unsupported / no answer" after a few seconds — never a hang.
- The server tells the record end on the wire by walking it (`lobby_record_wire_len`), not by the decoded
  string lengths (a truncated multi-byte name lossy-decodes shorter).

## 3. Flows

- **Host** (`ShortCodeHost` in directory.rs, driven by `NetCore::poll`, mpnet.rs): the REGISTER that already
  goes out every `REGISTER_EVERY_MS` from the game socket becomes a v2 packet carrying this register period's
  nonce and the host's LAN endpoint (`p2p::local_ipv4()` + the game port — the joiner on the host's own network
  needs it; the server only sees the public mapping). An `NFDA` for the pending nonce stores the code; the code
  is shown while fresh (`CODE_FRESH_MS` = 15 s = three register periods [S]), then the long code shows again —
  an entry the server dropped must not advertise a dead short code.
- **Joiner**: the join box accepts either form, case-insensitive, dashes/spaces ignored
  (`p2p::parse_join_target` → `JoinTarget::Short/Code/Address`). 4 symbols → a `ShortLookup` sends `NFDS` to
  the player's lobby server every 400 ms, 10 tries (~4 s); `NFDT` hands the host's public endpoint as the server
  saw it, the LAN endpoint when the host reported one, and the `LobbyInfo` record; the joiner then dials the
  public endpoint exactly as a long code does (`NetCore::dial` → rebind + `JoinReach`), with the LAN endpoint as
  a probe hint so the same-network path (`JoinReach` §3a LAN redirect) keeps working. 16 symbols or
  `a.b.c.d:port` behave exactly as today.
- **Server** (`Directory::handle`): `NFDS` is rate limited per source address (`LOOKUP_WINDOW_MS` = 10 s,
  `LOOKUP_WINDOW_MAX` = 30 [S] — codes are brute-forceable at 810 k); over the limit → `NFDN` reason 1.
  Unknown code → reason 0. Bad version → reason 2. `LIST` is untouched.

## 4. Front-end surface (no drawing here)

- `MpSession::join_code_display() -> Option<String>` (menu.rs): the short code while fresh, else the host's
  long code — what the lobby's code line shows (it is also what `MpReach.code` carries now, so the existing
  draw shows it).
- `MpSession::join_code_note() -> Option<String>`: the joiner's short-code status line ("Looking up ABCD…",
  "Code ABCD not found on <server>", "The server did not answer — an older nf-lobbyd?", "No lobby server is
  set — 4-character codes need one", "Slow down — the server rate-limited lookups").
- `mpnet::join_accepted_forms() -> &'static str`: what the join box accepts ("a 4-character code, IP:PORT, or
  a 16-symbol join code — case does not matter, dashes and spaces are ignored"), for the hint line.

## 5. Tests (`crates/nf-net/tests/short_codes.rs`, loopback, no internet; [S] throughout — they pin the
port's own protocol, never game behaviour)

1. `host_registers_and_gets_a_four_character_code` — v2 REGISTER → `NFDA`, 4 symbols of the alphabet, stable
   across re-registers, unique between two hosts; sister: an unknown code answers not-found (the comparison
   can fail).
2. `joiner_looks_up_and_connects` — real `Session` handshake on 127.0.0.1 through the looked-up endpoint
   (the sister is the unchanged long-code path, `loopback_e2e.rs::net_e2e_loopback_live`).
3. `unknown_code_is_a_clear_not_found` — `ShortLookup` → `NotFound` (not a timeout).
4. `old_server_silence_times_out_cleanly` — a raw socket that never answers (an old server) → `NoAnswer` after
   the tries; and a v1 REGISTER (no tail) on the new server still lists (old-host compat).
5. `bad_version_is_answered_unsupported` — LOOKUP with `ver = 2` → `NFDN` reason 2.
6. `lookups_are_rate_limited_per_source` — hammering one source → `RateLimited`; a second source is unaffected.
7. `display_falls_back_when_the_server_stops_answering` — `ShortCodeHost` shows the code while fresh, drops it
   after `CODE_FRESH_MS` (the long code shows).
