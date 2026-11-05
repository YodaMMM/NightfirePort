
## fx-muzzle-repin (2026-11-05)
- Task: land s/fx-muzzle-and-trail on current master. Live PCSX2 (07000005) confirmed the muzzle flash object + 3 jitter draws fire on every guard shot (0x178f68 x40, 0x179248/0x179268 x39), so the draws stay. Merge master, keep gas.rs hidden/born/child/rot fields on every literal, fix the ~30 tests the rebase breaks (RNG stream shift) without removing/reordering draws, fix stale drone.rs comment, run the full list + full suite.
