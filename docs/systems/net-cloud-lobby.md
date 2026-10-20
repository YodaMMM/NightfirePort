# net-cloud-lobby — the Cloudflare lobby service (public list + 4-character codes)

*All of this page is port design [S]: the PS2 game has no online mode (docs/systems/net-lobby-list.md,
docs/systems/net-p2p.md) and the only game facts involved are the lobby pages those docs already cover.
The user chose the shape (2026-10-19): "go with the cloudflare lobby" — a small free Cloudflare Worker as
the game's built-in meeting point; play stays peer-to-peer.*

Task 700x1 (`s/net-short-codes`, docs/systems/net-short-codes.md) built 4-character codes on the
player-set home server `nf-lobbyd` (UDP). This task swaps the **transport and meeting point** to the
cloud service; `nf-lobbyd` keeps working as the optional override.

## 1. What the service is (`tools/lobby-worker/`)

A Cloudflare Worker + one Durable Object (`Lobby`, singleton `idFromName("global")`), free tier:

| route | what it does |
|---|---|
| `POST /v1/host` | JSON body: `version, name, map, scenario, players, max, public, lan, port`. Stores the entry keyed by the requester's public IP (`CF-Connecting-IP` — set by Cloudflare, not spoofable) + `port`; answers `{"code":"ABCD","public_addr":"a.b.c.d:port","expires_in":90}`. The code is 4 symbols of the unambiguous alphabet (no 0/O/1/I/L/U — the same 30-symbol set as the short code, docs/systems/net-short-codes.md §1), unique among the live entries, **the same while refreshed** (the entry key is stable). Entry expires 90 s after the last refresh; the host refreshes every 30 s. |
| `DELETE /v1/host` | Removes the requester's entries (the body may name one `port`). Answers `{"ok":true}`. |
| `GET /v1/lobbies?version=…` | The **public**, live lobbies of that exact game version: `{"lobbies":[{code,name,map,scenario,players,max}]}`. **Never an IP** — a joiner gets an address only through a code lookup. |
| `GET /v1/code/{CODE}` | That lobby's public endpoint + LAN endpoint (the same-network path) + the display fields, or **404**. |

Abuse limits (all in the Durable Object, in memory): per-IP rate limits on every route
(§3 of `tools/lobby-worker/wrangler.toml` comments; 429 when over), max 8 live entries per IP,
name length (32 after filtering) and charset filtering (control + formatting characters stripped),
a 2 KB request-body cap. No IP is logged or stored beyond the live entry itself and the ephemeral
rate-limit windows.

## 2. The game side (nf-net / nf-view, no new crates)

- `nf_net::cloud::LOBBY_SERVICE_URL` — the deployed `*.workers.dev` URL, a constant **empty until the
  user deploys** (= the service is off). Overridable: `NF_LOBBY_SERVICE` env, else the
  `lobby_service = <url>` line of `%APPDATA%\NightfirePort\net.txt` (the same file `lobby_server`
  already rides; never built into the exe beyond the empty default).
- **Transport**: the system `curl` (`curl.exe` on Windows) in a **background thread**, exactly the
  updater's shape (crates/nf-view/src/launcher/update.rs `fetch`) — one short-lived `curl` per request,
  output captured, the game never blocks. The answer comes back to the poller through a channel.
- **Host** (`CloudHost`, driven by `NetCore::poll` while hosting + advertising): `POST /v1/host` every
  `CLOUD_REFRESH_MS` = 30 s (the body's `port` is the STUN-observed public UDP port when reachability
  knows one, else the game port; `lan` is the LAN endpoint, the same-network path). The answer's code
  shows while fresh (`CLOUD_FRESH_MS` = 90 s + one refresh period — the server drops the entry at 90 s,
  so a service that stopped answering drops the code too) and `join_code_display()` shows it ahead of
  the `nf-lobbyd` short code and the long code. On leave/close a `DELETE /v1/host` fires (fire and
  forget). Service unreachable or unset → the long code, today's fallback.
- **Public/private**: lobbies are **public by default**; the host toggle (`MpSession::
  set_public_lobby` / `public_lobby()`) flips the body's `public` flag. Private = code only: the entry
  resolves by code but never lists.
- **Join**: the Join Online page's **Public lobbies** list (`MpSession::cloud_lobbies()`, refreshed
  every `CLOUD_LIST_EVERY_MS` = 5 s while the page is open) + the code box. Choosing a lobby or typing
  a 4-character code → `GET /v1/code/{CODE}` → the public endpoint dials exactly as a long code does
  (rebind + `JoinReach`, the LAN endpoint as the same-network probe hint); unknown code → the note
  "Code ABCD was not found". 16-symbol codes and `a.b.c.d:port` behave exactly as before.
- **Override**: when `lobby_server = host:port` (nf-lobbyd) is set it wins over the cloud service —
  the UDP REGISTER/LOOKUP path of net-short-codes runs as before; the cloud service is the default
  meeting point when nothing else is set.
- **Getters for the draw code** (the main agent draws every page): `MpSession::cloud_lobbies()`,
  `MpSession::join_code_display()` (already there), `MpSession::public_lobby()`.

## 3. Version

`game_version()` = `"<PROTOCOL_VERSION>.<DEFAULT_BUILD_ID>"` (`"8.1"`); the list request and the host
register both carry it and the service string-matches. A version bump drops a host off older joiners'
lists instead of handing them a lobby they cannot join (the same gate the LAN browser's `joinable`
row applies).

## 4. Tests

- **The worker** (`tools/lobby-worker/test/handler.test.mjs`, plain `node:test` — no Miniflare in the
  sandbox): the handler logic against a fake DO environment — register/refresh stability, code
  alphabet, list filters (version + public), code lookup, expiry, DELETE, rate limits, name
  filtering, size caps.
- **The game** (`crates/nf-net/tests/cloud_lobby.rs`): a local mock HTTP server (`std::TcpListener`
  + threads, no crate) speaking the same routes: register → code → lookup → a **real loopback
  session** connects; the list filters by version and the public flag; expiry drops the row; the
  service unreachable → the long code shows and the joiner's lookup fails cleanly;
  `net_e2e_loopback_live` stays untouched (the cloud path is strictly additive).
