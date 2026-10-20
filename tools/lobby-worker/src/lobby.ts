// The Durable Object holding the live lobby entries (docs/systems/net-cloud-lobby.md) [S port design].
//
// One instance (`idFromName("global")`) serves every route: the entries are tiny and the whole table must
// agree on codes, rate limits and expiry, so a single serialized object is the simplest correct shape on
// the free tier. State is memory only — live entries expire after 90 s anyway and hosts re-register within
// 30 s after any restart; nothing is written to storage and no IP is logged beyond the live entry itself.
import { handle, newStore } from "./handler";

export class Lobby {
  store: Map<string, unknown>;

  constructor(_state: unknown, _env: unknown) {
    this.store = newStore() as unknown as Map<string, unknown>;
  }

  async fetch(request: Request): Promise<Response> {
    const url = new URL(request.url);
    const ip = request.headers.get("CF-Connecting-IP") || "0.0.0.0";
    const body = request.method === "POST" || request.method === "DELETE" ? await request.text().catch(() => null) : null;
    const out = handle({ method: request.method, path: url.pathname, query: url.searchParams, ip, body, now: Date.now() },
      this.store as never);
    return new Response(JSON.stringify(out.json), {
      status: out.status,
      headers: { "Content-Type": "application/json", "Access-Control-Allow-Origin": "*" },
    });
  }
}
