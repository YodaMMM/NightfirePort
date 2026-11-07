# Unread placement params: drones, spawners, AI volumes, nodes

Classes: drone placements, spawners, AI volumes and path nodes (crates/nf-game/src/drone.rs create paths). Work that shifts the shared random stream stays on its branch and says so in the report.

Method. For each class below read its `*_Create` in the disassembly and list every placement param and word it stores. For each one, find what the port does with it. Where the port ignores it, count non-zero values on every level: a small throwaway test using `Level::parse` and the `nf_view::dev::LEVELS` list runs in under a second. A value that is zero on every level is closed with that census as the proof. A non-zero value on any on-foot level is a real gap: trace the reader in the game, port it, and test it on the real level, red on the old code. If a port comment calls something inert, check that claim on all levels, not one.

Why: `Destroy_Update`'s p6 countdown was noted inert because p6 is 0 on 0700000d. 07000014 has 17 leaves with non-zero p6, and its generator chain ran 100 times too fast.

Timing values must come from a fresh-load reading where one exists; do not derive a tick count from a symbol name (that countdown's scale is 30 per p6, not FRAME_RATE). Branch from origin/master, push to `s/unread-params-drones`, report in docs/reports/unread-params-drones.md with the census table. No draw changes, no driving levels. Do not pin anything you have not traced.

---
claimed-by: sandbox4 at 2026-11-07T09:01Z
