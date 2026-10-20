// The lobby service's handler logic (docs/systems/net-cloud-lobby.md, docs/tools/cloud-lobby.md) [S port design].
//
// Pure and runtime-free on purpose: `handle(ctx, store)` takes a plain request context and a plain store,
// returns a plain response. The Durable Object (lobby.ts) adapts the Cloudflare Request/Response objects
// around it; the unit tests (test/handler.test.mjs) drive it directly under plain node — no Miniflare needed.

export const ALPHABET = "23456789ABCDEFGHJKMNPQRSTVWXYZ"; // 30 symbols, no 0/O/1/I/L/U (the short code's set)
export const CODE_LEN = 4;
export const ENTRY_TTL_MS = 90_000; // an entry expires 90 s after its last refresh
export const MAX_BODY_BYTES = 2048;
export const MAX_ENTRIES_PER_IP = 8;
export const MAX_NAME = 32;
export const MAX_VERSION = 32;

// Per-IP sliding-window rate limits per route class: [window ms, max requests]. 429 when over.
export const RATE_LIMITS = {
  host: [60_000, 6], // one POST every 30 s (a refresh), plus a leave/re-register
  list: [60_000, 20], // the join page refreshes every 5 s = 12/min
  code: [60_000, 30], // a joiner types one code; brute force stays impractical (810k live-code space is capped by entries)
  del: [60_000, 6],
};

// ── the store (the Durable Object keeps exactly this; nothing at rest, no logs) ──
export function newStore() {
  return { entries: new Map(), // key `${ip}:${port}` → entry
    codes: new Map(), // code → key
    rate: new Map(), // `${class}:${ip}` → [timestamps ms]
  };
}

export function newEntry(key, code, body, ip, now) {
  return { key, code, ip, version: body.version, name: body.name, map: body.map, scenario: body.scenario,
    players: body.players, max: body.max, pub: !!body.public, lan: body.lan || "", port: body.port, seen: now };
}

// ── helpers ──

/** A random code unique among the live entries (crypto.getRandomValues — Workers and node ≥ 19 both have it). */
export function assignCode(store) {
  for (;;) {
    let code = "";
    const bytes = new Uint8Array(CODE_LEN);
    crypto.getRandomValues(bytes);
    for (const b of bytes) code += ALPHABET[b % ALPHABET.length];
    if (!store.codes.has(code)) return code;
  }
}

/** The name as it is stored: control (C0/C1) and formatting characters stripped, whitespace collapsed,
 *  trimmed, capped at MAX_NAME code points. Empty → "Lobby". */
export function filterName(raw) {
  if (typeof raw !== "string") return "Lobby";
  let s = "";
  for (const ch of raw) {
    const c = ch.codePointAt(0);
    const control = c < 0x20 || (c >= 0x7f && c <= 0x9f);
    const formatting = (c >= 0x200b && c <= 0x200f) || c === 0x2028 || c === 0x2029 || (c >= 0x202a && c <= 0x202e) ||
      (c >= 0x2060 && c <= 0x2064) || c === 0xfeff || c === 0xad;
    s += control || formatting ? " " : ch;
  }
  s = s.replace(/\s+/g, " ").trim();
  const pts = Array.from(s);
  if (pts.length > MAX_NAME) s = pts.slice(0, MAX_NAME).join("").trimEnd();
  return s === "" ? "Lobby" : s;
}

function dropExpired(store, now) {
  for (const [key, e] of store.entries) if (now - e.seen > ENTRY_TTL_MS) { store.entries.delete(key); store.codes.delete(e.code); }
}

function rateLimited(store, cls, ip, now) {
  const [win, max] = RATE_LIMITS[cls];
  const k = `${cls}:${ip}`;
  const ts = (store.rate.get(k) || []).filter((t) => now - t < win);
  if (ts.length >= max) { store.rate.set(k, ts); return true; }
  ts.push(now);
  store.rate.set(k, ts);
  return false;
}

function resp(status, json) { return { status, json }; }

function badReq(msg) { return resp(400, { error: msg }); }

const LAN_RE = /^\d{1,3}(\.\d{1,3}){3}:\d{1,5}$/;

/** The body's requested code, normalized: 4 alphabet symbols (case-insensitive) or "" when absent /
 *  malformed (an ask is a preference — it is never an error). */
export function askedCode(raw) {
  if (typeof raw !== "string" || raw.length !== CODE_LEN) return "";
  const up = raw.toUpperCase();
  for (const c of up) if (!ALPHABET.includes(c)) return "";
  return up;
}

/** Validates and normalizes the POST /v1/host body. Returns {ok, body|error}. */
export function parseHostBody(raw) {
  let b;
  try { b = JSON.parse(raw); } catch { return { ok: false, error: "bad JSON" }; }
  if (typeof b !== "object" || b === null) return { ok: false, error: "bad JSON" };
  if (typeof b.version !== "string" || b.version === "" || b.version.length > MAX_VERSION) return { ok: false, error: "bad version" };
  if (!Number.isInteger(b.map) || b.map < 0 || b.map > 0xffffffff) return { ok: false, error: "bad map" };
  if (!Number.isInteger(b.scenario) || b.scenario < 0 || b.scenario > 0x7fffffff) return { ok: false, error: "bad scenario" };
  if (!Number.isInteger(b.players) || b.players < 0 || b.players > 255) return { ok: false, error: "bad players" };
  if (!Number.isInteger(b.max) || b.max < 1 || b.max > 255) return { ok: false, error: "bad max" };
  if (typeof b.port !== "number" || !Number.isInteger(b.port) || b.port < 1 || b.port > 65535) return { ok: false, error: "bad port" };
  if (b.lan !== undefined && b.lan !== "" && (typeof b.lan !== "string" || !LAN_RE.test(b.lan))) return { ok: false, error: "bad lan" };
  return { ok: true, body: { version: b.version, name: filterName(b.name), map: b.map, scenario: b.scenario,
    players: b.players, max: b.max, public: b.public !== false, lan: b.lan || "", port: b.port,
    code: askedCode(b.code) } };
}

function publicRow(e) {
  return { code: e.code, name: e.name, map: e.map, scenario: e.scenario, players: e.players, max: e.max };
}

/**
 * One request through the routes. `ctx` = { method, path, query (URLSearchParams), ip, body (string|null), now }.
 * Every answer is JSON; errors are `{"error": "..."}` with a 4xx/5xx status.
 */
export function handle(ctx, store) {
  const { method, path, ip, now } = ctx;
  dropExpired(store, now);
  if (path === "/v1/host" && method === "POST") {
    if (ctx.body === null || ctx.body.length > MAX_BODY_BYTES) return resp(413, { error: "too large" });
    if (rateLimited(store, "host", ip, now)) return resp(429, { error: "rate limited" });
    const p = parseHostBody(ctx.body);
    if (!p.ok) return badReq(p.error);
    const key = `${ip}:${p.body.port}`;
    let code = store.entries.get(key)?.code;
    if (!code) {
      // The code is keyed to ip:port, and a host's dial port can change mid-session (the STUN
      // mapping lands around the lobby → match hand-over). A new key must not mean a new code:
      // the client re-sends the code it holds (net-cloud-code-stable), and the same host's entry
      // MOVES — one entry, one code, for the whole session — while any other address's code stays
      // untouchable and a malformed ask just falls through to a fresh one.
      const asked = p.body.code;
      const ownerKey = asked ? store.codes.get(asked) : undefined;
      if (ownerKey) {
        const owner = store.entries.get(ownerKey);
        if (owner && owner.ip === ip) { store.entries.delete(ownerKey); store.codes.delete(asked); }
      }
      if (asked && !store.codes.has(asked)) {
        code = asked; // free (or just moved here): the session keeps its code
      } else {
        if ([...store.entries.values()].filter((e) => e.ip === ip).length >= MAX_ENTRIES_PER_IP)
          return resp(429, { error: "too many lobbies from this address" });
        code = assignCode(store);
      }
    }
    store.entries.set(key, newEntry(key, code, p.body, ip, now));
    store.codes.set(code, key);
    return resp(200, { code, public_addr: `${ip}:${p.body.port}`, expires_in: ENTRY_TTL_MS / 1000 });
  }
  if (path === "/v1/host" && method === "DELETE") {
    if (rateLimited(store, "del", ip, now)) return resp(429, { error: "rate limited" });
    let port = null;
    if (ctx.body) { try { const b = JSON.parse(ctx.body); if (Number.isInteger(b?.port)) port = b.port; } catch { /* delete all */ } }
    for (const [key, e] of store.entries)
      if (e.ip === ip && (port === null || e.port === port)) { store.entries.delete(key); store.codes.delete(e.code); }
    return resp(200, { ok: true });
  }
  if (path === "/v1/lobbies" && method === "GET") {
    if (rateLimited(store, "list", ip, now)) return resp(429, { error: "rate limited" });
    const version = ctx.query.get("version") || "";
    const lobbies = [...store.entries.values()].filter((e) => e.pub && e.version === version).map(publicRow);
    return resp(200, { lobbies });
  }
  if (path.startsWith("/v1/code/") && method === "GET") {
    if (rateLimited(store, "code", ip, now)) return resp(429, { error: "rate limited" });
    const code = path.slice("/v1/code/".length).toUpperCase();
    const key = store.codes.get(code);
    const e = key ? store.entries.get(key) : undefined;
    if (!e) return resp(404, { error: "not found" });
    return resp(200, { code: e.code, public_addr: `${e.ip}:${e.port}`, lan_addr: e.lan, name: e.name,
      map: e.map, scenario: e.scenario, players: e.players, max: e.max });
  }
  return resp(404, { error: "no such route" });
}
