# level-08-hind-flight-pin — prove the 08 helicopter flies its attack

On 07000008 the helicopter that attacks the gondola must fly its attack, not hold one spot. Copter_Move 0x17d628 and Copter_Update 0x17dbb8 are ported in crates/nf-game/src/copter.rs, but no test measures the flight on the real path. On real input from walkreal_07000008_live's route, log the copter's position, state and path index every 30 ticks from its arrival to its death: how far it travels, which Copter_Update cases run, and whether any stretch holds still. Compare with what the disc paths and the case machine say it should do [M + D]. Fix any stall with the address, and pin the travel distance and the case order in a real-input test. Every walkreal_*_live; the full suite total line.

---
claimed-by: sandbox1 at 2026-11-04T18:54Z

---
## Report (done, 2026-11-04T20:16Z)

Closed by the driver: the report is on `s/level-08-hind-flight-pin` (docs/reports/level-08-hind-flight-pin.md); the worker didn't run finish.sh.

---
## Report from the worker (done, 2026-11-04T20:22Z)

