// The handler logic's unit tests (plain node:test, no Miniflare needed): the same
// routes the game's client speaks, driven directly against the store (docs/systems/net-cloud-lobby.md).
import test from "node:test";
import assert from "node:assert/strict";
import { handle, newStore, filterName, ALPHABET, ENTRY_TTL_MS, RATE_LIMITS } from "../src/handler.ts";

const T0 = 1_700_000_000_000;

function svc() {
  const store = newStore();
  return {
    store,
    call(method, path, { ip = "203.0.113.5", body = null, query = "", now = T0 } = {}) {
      return handle({ method, path, query: new URLSearchParams(query), ip, body, now }, store);
    },
  };
}

function hostBody({ version = "8.1", name = "Bond's game", pub = true, port = 27420, lan = "192.168.1.10:27420", players = 1, code } = {}) {
  return JSON.stringify({ version, name, map: 0x07000023, scenario: 1, players, max: 4, public: pub, lan, port, ...(code !== undefined ? { code } : {}) });
}

test("a host registers and gets a 4-character unambiguous code", () => {
  const s = svc();
  const r = s.call("POST", "/v1/host", { body: hostBody() });
  assert.equal(r.status, 200);
  assert.equal(r.json.code.length, 4);
  for (const c of r.json.code) assert.ok(ALPHABET.includes(c), `${c} of the alphabet`);
  assert.equal(r.json.public_addr, "203.0.113.5:27420");
  assert.equal(r.json.expires_in, 90);
});

test("a refresh keeps the code and updates the row", () => {
  const s = svc();
  const a = s.call("POST", "/v1/host", { body: hostBody() }).json.code;
  const b = s.call("POST", "/v1/host", { body: hostBody(), now: T0 + 30_000 }).json.code;
  assert.equal(a, b, "the same while refreshed");
  const changed = hostBody({ players: 2, name: "Renamed" });
  const c = s.call("POST", "/v1/host", { body: changed, now: T0 + 60_000 }).json.code;
  assert.equal(c, a, "still the same entry");
  const list = s.call("GET", "/v1/lobbies", { query: "version=8.1", now: T0 + 60_000 }).json.lobbies;
  assert.equal(list.length, 1);
  assert.equal(list[0].players, 2);
  assert.equal(list[0].name, "Renamed");
});

test("two hosts get different codes; one IP holds at most 8 entries", () => {
  const s = svc();
  const a = s.call("POST", "/v1/host", { body: hostBody() }).json.code;
  const b = s.call("POST", "/v1/host", { body: hostBody({ port: 27421 }), ip: "203.0.113.6" }).json.code;
  assert.notEqual(a, b);
  // The per-IP rate limit (6/min) and the 90 s TTL both shape the cadence: 12 s apart keeps every
  // 60 s window under 6 posts, and all 8 entries stay live at T0+84 s, so the 9th hits the cap.
  for (let i = 0; i < 7; i++)
    assert.equal(s.call("POST", "/v1/host", { body: hostBody({ port: 27422 + i }), now: T0 + 12_000 * (i + 1) }).status, 200, `entry ${i + 2}`);
  assert.equal(s.call("POST", "/v1/host", { body: hostBody({ port: 30000 }), now: T0 + 84_001 }).status, 429, "the 9th entry from one IP");
});

test("the list filters by version and the public flag, and never shows an IP", () => {
  const s = svc();
  s.call("POST", "/v1/host", { body: hostBody() }); // public 8.1
  s.call("POST", "/v1/host", { body: hostBody({ pub: false, port: 27421 }), ip: "203.0.113.6", now: T0 + 30_000 }); // private
  s.call("POST", "/v1/host", { body: hostBody({ version: "9.0", port: 27422 }), ip: "203.0.113.7", now: T0 + 60_000 }); // other version
  const r = s.call("GET", "/v1/lobbies", { query: "version=8.1" });
  assert.equal(r.status, 200);
  assert.equal(r.json.lobbies.length, 1, "public + same version only");
  assert.equal(JSON.stringify(r.json).includes("203.0.113"), false, "never an IP");
  assert.equal(r.json.lobbies[0].code.length, 4);
  const other = s.call("GET", "/v1/lobbies", { query: "version=9.0" }).json.lobbies;
  assert.equal(other.length, 1, "the 9.0 host registered public, so it lists under its own version");
});

test("a code lookup answers both endpoints or 404", () => {
  const s = svc();
  const code = s.call("POST", "/v1/host", { body: hostBody() }).json.code;
  const r = s.call("GET", `/v1/code/${code.toLowerCase()}`, { ip: "198.51.100.9" });
  assert.equal(r.status, 200);
  assert.equal(r.json.public_addr, "203.0.113.5:27420");
  assert.equal(r.json.lan_addr, "192.168.1.10:27420", "the same-network path");
  assert.equal(r.json.name, "Bond's game");
  assert.equal(s.call("GET", "/v1/code/ZZZZ", { ip: "198.51.100.9" }).status, 404);
});

test("an entry expires 90 s after its last refresh", () => {
  const s = svc();
  const code = s.call("POST", "/v1/host", { body: hostBody() }).json.code;
  assert.equal(s.call("GET", "/v1/lobbies", { query: "version=8.1", now: T0 + ENTRY_TTL_MS }).json.lobbies.length, 1, "still live at 90 s");
  assert.equal(s.call("GET", `/v1/code/${code}`, { now: T0 + ENTRY_TTL_MS + 1 }).status, 404, "gone one ms later");
  assert.equal(s.call("GET", "/v1/lobbies", { query: "version=8.1", now: T0 + ENTRY_TTL_MS + 1 }).json.lobbies.length, 0);
});

test("DELETE removes the leaver's entries at once", () => {
  const s = svc();
  const code = s.call("POST", "/v1/host", { body: hostBody() }).json.code;
  assert.equal(s.call("DELETE", "/v1/host", { body: JSON.stringify({ port: 27420 }) }).json.ok, true);
  assert.equal(s.call("GET", `/v1/code/${code}`).status, 404);
  assert.equal(s.call("GET", "/v1/lobbies", { query: "version=8.1" }).json.lobbies.length, 0);
  assert.equal(s.call("DELETE", "/v1/host", { ip: "198.51.100.9" }).json.ok, true, "someone else's delete removes nothing of ours");
});

test("per-IP rate limits answer 429 on every route", () => {
  const s = svc();
  const [, hostMax] = RATE_LIMITS.host;
  for (let i = 0; i < hostMax; i++) s.call("POST", "/v1/host", { body: hostBody({ port: 27420 + i }) });
  assert.equal(s.call("POST", "/v1/host", { body: hostBody({ port: 30000 }) }).status, 429);
  for (let i = 0; i < RATE_LIMITS.list[1]; i++) s.call("GET", "/v1/lobbies", { query: "version=8.1" });
  assert.equal(s.call("GET", "/v1/lobbies", { query: "version=8.1" }).status, 429);
  for (let i = 0; i < RATE_LIMITS.code[1]; i++) s.call("GET", "/v1/code/ZZZZ");
  assert.equal(s.call("GET", "/v1/code/ZZZZ").status, 429);
  assert.equal(s.call("GET", "/v1/lobbies", { query: "version=8.1", ip: "198.51.100.9" }).status, 200, "another IP is unaffected");
});

test("oversized and malformed requests are refused", () => {
  const s = svc();
  assert.equal(s.call("POST", "/v1/host", { body: "x".repeat(2049) }).status, 413);
  assert.equal(s.call("POST", "/v1/host", { body: "{not json" }).status, 400);
  assert.equal(s.call("POST", "/v1/host", { body: JSON.stringify({ version: "8.1", map: 1, scenario: 1, players: 1, max: 4, port: 0 }) }).status, 400, "port 0");
  assert.equal(s.call("POST", "/v1/host", { body: JSON.stringify({ version: "", map: 1, scenario: 1, players: 1, max: 4, port: 27420 }) }).status, 400);
  assert.equal(s.call("GET", "/v1/nothing").status, 404);
});

test("names are filtered: control and formatting characters stripped, length capped", () => {
  assert.equal(filterName("Bond\u0000\u001b's game"), "Bond 's game");
  assert.equal(filterName("Bond\u200b\u202e's game"), "Bond 's game");
  assert.equal(filterName("  spaced\tname  "), "spaced name");
  assert.equal(filterName("L".repeat(40)).length, 32);
  assert.equal(filterName(""), "Lobby");
  assert.equal(filterName("\u0000"), "Lobby");
});

test("a private lobby resolves by code but never lists", () => {
  const s = svc();
  const code = s.call("POST", "/v1/host", { body: hostBody({ pub: false }) }).json.code;
  assert.equal(s.call("GET", "/v1/lobbies", { query: "version=8.1" }).json.lobbies.length, 0);
  assert.equal(s.call("GET", `/v1/code/${code}`).status, 200, "code join still works");
});

// net-cloud-code-stable: the advert's port can change mid-session (the STUN-observed mapping lands
// around the lobby → match hand-over), and the entry is keyed by ip:port — so the client re-sends
// its code and the service moves the same host's entry instead of assigning a new code.
test("a port-change re-register carries the code: one entry, one code, for the session", () => {
  const s = svc();
  const a = s.call("POST", "/v1/host", { body: hostBody() }).json.code;
  const b = s.call("POST", "/v1/host", { body: hostBody({ port: 51234, code: a }), now: T0 + 30_000 }).json.code;
  assert.equal(b, a, "the code survives the port change");
  const r = s.call("GET", `/v1/code/${a}`, { ip: "198.51.100.9", now: T0 + 30_000 });
  assert.equal(r.status, 200);
  assert.equal(r.json.public_addr, "203.0.113.5:51234", "the lookup answers the new endpoint");
  const list = s.call("GET", "/v1/lobbies", { query: "version=8.1", now: T0 + 30_000 }).json.lobbies;
  assert.equal(list.length, 1, "one entry per session — the stale pre-change entry moved, not doubled");
  // A lowercase ask normalizes to the same code.
  const c = s.call("POST", "/v1/host", { body: hostBody({ port: 27421, code: a.toLowerCase() }), now: T0 + 60_000 }).json.code;
  assert.equal(c, a, "the ask is case-insensitive");
});

test("a code owned by another host is never stolen; two different hosts still differ", () => {
  const s = svc();
  const a = s.call("POST", "/v1/host", { body: hostBody() }).json.code; // host 203.0.113.5
  const b = s.call("POST", "/v1/host", { body: hostBody({ port: 27421, code: a }), ip: "203.0.113.6" }).json.code;
  assert.notEqual(b, a, "a different address asking with someone else's code gets a fresh one");
  const r = s.call("GET", `/v1/code/${a}`, { ip: "198.51.100.9" });
  assert.equal(r.json.public_addr, "203.0.113.5:27420", "the owner's entry is untouched");
  // And the plain sister: two hosts, no asks, still get different codes.
  const c = s.call("POST", "/v1/host", { body: hostBody({ port: 27422 }), ip: "203.0.113.7" }).json.code;
  assert.notEqual(c, a);
  assert.notEqual(c, b);
});

test("an unknown or malformed requested code falls back to a fresh one, not an error", () => {
  const s = svc();
  const a = s.call("POST", "/v1/host", { body: hostBody() }).json.code;
  const b = s.call("POST", "/v1/host", { body: hostBody({ port: 27421, code: "AA0I" }), now: T0 + 30_000 }).json.code; // 0/I are not in the alphabet
  assert.equal(b.length, 4);
  assert.notEqual(b, a);
  const c = s.call("POST", "/v1/host", { body: hostBody({ port: 27422, code: "ZZZZ9" }), now: T0 + 60_000 }).json.code; // 5 chars
  assert.equal(c.length, 4, "a malformed ask is ignored, never a 4xx");
  const d = s.call("POST", "/v1/host", { body: hostBody({ port: 27423, code: 42 }), now: T0 + 90_000 }).json.code; // not a string
  assert.equal(d.length, 4);
});
