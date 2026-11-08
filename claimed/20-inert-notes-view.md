# Port notes that say not ported: nf-view sim code

The same grep over crates/nf-view/src/scene.rs and lib.rs, gameplay arms only. For each note: is the arm reachable on any on-foot level? Close with a count, or port it.

Read the game code first, count on every level with `Level::parse` over `nf_view::dev::LEVELS`, and test on the real level, red on the old code. A count of zero on every level closes an item with that count as proof. Do not trust a port comment that says inert. Values measured at load can differ from the same global in play (FRAME_RATE reads 30 while a level builds, 60 after). Branch from origin/master, push to `s/inert-notes-view`, report in docs/reports/inert-notes-view.md. No driving levels. Draw work: leave a stub in the hand-off list and say so.

---
claimed-by: sandbox6 at 2026-11-08T00:07Z
