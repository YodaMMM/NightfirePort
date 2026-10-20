// The worker entry: every /v1 request goes to the single `Lobby` Durable Object
// (docs/systems/net-cloud-lobby.md) [S port design]. A GET / answers with a one-line health note.
import { Lobby } from "./lobby";

export { Lobby };

export default {
  async fetch(request: Request, env: { LOBBY: { idFromName(n: string): unknown; get(id: unknown): { fetch(r: Request): Promise<Response> } } }): Promise<Response> {
    const url = new URL(request.url);
    if (url.pathname === "/") return new Response("NightfirePort lobby service (docs/tools/cloud-lobby.md)\n");
    if (!url.pathname.startsWith("/v1/")) return new Response(JSON.stringify({ error: "no such route" }), { status: 404, headers: { "Content-Type": "application/json" } });
    if (request.method === "OPTIONS") return new Response(null, { status: 204, headers: { "Access-Control-Allow-Origin": "*", "Access-Control-Allow-Methods": "GET, POST, DELETE", "Access-Control-Allow-Headers": "Content-Type" } });
    const id = env.LOBBY.idFromName("global");
    return env.LOBBY.get(id).fetch(request);
  },
};
