# Aim: the routes left

Branch from `origin/s/1b-zero-g-floor`, merge current `origin/master`, push to `s/aim-route-finish`. Read docs/reports/1b-zero-g-floor.md: all six other reds are route, none are snap faults. All eight consoles latch on 1b.

1. 1b: the missile deadline. Consoles 5, 8, 1, 2 land late (30-438 ticks). Use the report's levers: pre-charge the clip, lower the burst gate to 36, give the 7 -> 1 flight the two-phase treatment.
2. 14 x3: the waypoint through the open doorway (about x 31.8, z 36.3) before the leg-2 spot.
3. 08, 0c, 16: the report's classification.

Do not change collision, zero-g or snap code; `10-zero-g-slide` handles zero-g separately, and if it lands, drop any lever that only exists to avoid the pocket. Keep the [L] 08 pins and the 0x30000000 fix. This branch moves where rounds land, so report what it does to the stream batch: if it needs joining `s/stream-batch`, say so rather than re-pinning twice.

---
claimed-by: sandbox7 at 2026-11-06T13:22Z
