# Multiplayer bots: arrival distance and walk speed

The markers-mp-bots work described two bot rules as ported that are not in the code: the goal arrival distance 2.0 (the port still uses under 1.0) and the walk speed 1.3 unclamped (the port still caps it with `.min(1.0)`). Read both in the decompile (the BOT_* move and goal functions the bots.rs comments cite), port them, and pin each with a test that fails on the old code. Run all mp_* and net_* binaries, then a two-instance host+join check on 127.0.0.1 with bots (no desync).

Branch from origin/master, push to `s/mp-bot-arrival-speed`, report in docs/reports/mp-bot-arrival-speed.md. No draw changes.

---
claimed-by: sandbox5 at 2026-11-10T01:23Z

---
## Report (done, 2026-11-10T02:56Z)

