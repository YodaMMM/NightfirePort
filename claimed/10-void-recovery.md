# Falling out of the world: the game recovers Bond, the port does not

Branch from current `origin/master`, push to `s/void-recovery`. Read docs/reports/door-walk-03b.md on `s/door-walk-03b`: on 07000003 the walk fell to y -62000 through strips x 11.1-13.4 and 15.6-17.5, z -30.4..-35.6, and a player can reach them.

Measured with Bond teleported, no input, same spots in both (god mode in the game):
| spot | PCSX2 | port |
| (14.5, 6.0, -32) | stands, y 6.28, flags 0x18 | stands, y 6.28 |
| (16.5, 6.3, -33) | pushed to x 15.99, stands, y 6.23, flags 0x18 | falls forever, y -160 after 6 s |
| (12.0, 6.3, -32) | back at the level start (-8.85, 6.41, -23.19) inside 1 s | falls forever; `find_cel` None |

Find what the game does for an object outside every cel, and for the push at x 16.5. Start from `control_handle_cel_change` 0x133288 and `build_LinkToRoom` 0x1c9680 (the unlinked retry), `Player_StandAtNewPosition` 0x190d88, and the collision the port misses at x 16: is it geometry the port's cel scoping drops once `bond.cel` is None? Cite it.

A teleport is not a walk: also reproduce it with real input off the moat edge in the port, and say whether the game's recovery would run there. Fix the port with a test on the real level, proved red on the old code. Keep the change to the out-of-cel case: broad collision changes failed the suite on `s/level-14-float-on-master`. Gate on walkreal 03, the walkreal set, and `mp_perf_16_live`.

---
claimed-by: sandbox7 at 2026-11-06T17:24Z
