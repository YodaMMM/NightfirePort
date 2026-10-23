# The Cloudflare lobby service — deploy steps for the player

The game's built-in meeting point (`docs/systems/net-cloud-lobby.md`) is a small Cloudflare Worker.
**You deploy it once, on your own free Cloudflare account** — nobody else ever touches your account
or tokens, and the service holds no secrets (it only lists lobbies and resolves join codes).

## What you need

- A free Cloudflare account (https://dash.cloudflare.com/sign-up — the free plan is enough; Workers
  and Durable Objects are included on it).
- Node.js 18+ with `npm`/`npx`.

## Deploy (from the repo's `tools/lobby-worker/` folder)

```sh
cd tools/lobby-worker
npm install          # wrangler + typescript (the only dependencies, dev-only)
npx wrangler login   # opens the browser; log in to YOUR Cloudflare account
npx wrangler deploy
```

`wrangler deploy` prints the URL of the service, e.g.

```
Published netfire-lobby (0.01 sec)
  https://netfire-lobby.<your-subdomain>.workers.dev
```

## The URL in the game

The deployed URL is built into every exe: `LOBBY_SERVICE_URL` in `crates/nf-net/src/cloud.rs`, currently
`https://netfire-lobby.netfire-lobby-worker.workers.dev` (deployed 2026-10-19). Players set
nothing. A player may override it with a `lobby_service = <url>` line in `%APPDATA%\NightfirePort\net.txt` or the
`NF_LOBBY_SERVICE` environment variable (the env var wins; an empty value turns the service off: long codes and LAN
lobbies only).

## What it costs

Nothing on the free tier: the free plan allows 100 000 requests/day; one hosted lobby costs ~2
requests/minute (a 30 s refresh) and an open join page ~12/minute. The worker stores nothing at rest
(live entries expire after 90 s, rate-limit windows live in memory) and logs no IP beyond the live
entry.

## Verifying it works

```sh
curl "https://netfire-lobby.<your-subdomain>.workers.dev/v1/lobbies?version=8.1"
# → {"lobbies":[]}
```

Then host a game in NightfirePort: the lobby page shows a 4-character code, and the lobby appears in
a second install's Public lobbies list.

## Changing the URL later

`npx wrangler deploy` again from the same folder replaces the worker in place (entries are in-memory;
a redeploy drops them, hosts re-register within 30 s). To move to a different URL, change the
`lobby_service =` line — no game update needed.
