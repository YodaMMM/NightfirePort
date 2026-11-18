
## retune-nb-05 (2026-11-18, s/retune-nb-05)

Task: commit 3412a89a3 made every guard run the game spacing scan and the close-range alt attack from the fire states; the 05 walk `the_door_band_exit_chains_to_07000006_walked_real_damage_00agent` now dies. Read the ignore and the new 05 helpers in that commit, fix the fight that kills Bond (real input only, no top-ups, never lower damage or health, never change guard code), helpers stay gated to level+difficulty. Remove the ignore when green; if still red, push the measured ledger and the next lever into the ignore message. Run the whole walkreal_07000005_live binary.

### retune-nb-05 progress

### retune-nb-05 progress (2026-11-18, ~16:35 UTC)

- **The 00 Agent walk `the_door_band_exit_chains_to_07000006_walked_real_damage_00agent` is
  UN-IGNORED and green** — the whole walkreal_07000005_live binary runs 17 passed / 0 failed /
  0 ignored in 72.2 s (--test-threads=1, --release, slim ISO).
- The nb-scan-all ledger death ("every gun dry at the walkway pair", t10830 at
  (43.01, 45.19, -73.11) [L the 2026-11-18 repro]) had two measured causes:
  1. **The master weapon cycle livelocks on the dry 0x16.** switch_to_armed presses
     weapon_cycle and waits 60 ticks; the picked-up 0x24's draw outlasts the wait, so every
     press re-queued the cycle from the old gun — 12 cycle prints all `weapon 0x16 clip 0
     spare 0`, the 0x24 (clip 5, picked up on the ditch flank [L the state census]) never
     armed, and the answer burned 732 ticks dry in #13's volley. Fix: level-local
     `cycle_armed_slow` (one press, waits up to 400 ticks for the draw, prefers the 0x24) —
     the shared walker keeps the master behaviour for every other walk.
  2. **44.0 of the pre-strip cost was #3** (the level's only 20-hp guard [L the state census],
     4 hits of 11.0 = the 0x16 headshot tier, 2 before the return-fire fight opened): the
     lane-approach pre-emptive in `walk_to_the_slab_east_end` (health-signature pick > 10.0,
     gated `sim.drones.difficulty >= 3`) kills him from the strip west leg before his field
     crosses Bond. First lever run measured: a radius-5 spot pick found nothing (he had left
     his post), the fight ran unchanged — the health pick is the fix.
- The strip answers now lead with the 0x24 (30.0 damage [L ee_start_05.bin weapon_data
  0x2bf150 + 0x24*0x10C] — one round drops a 10-hp guard; the 0x16's grazes measured 1.6-1.7
  and #13 survived six of them at 0.1). The d3 walkway answer (after the walkway south 1 leg)
  answers the walkway pair / door guards' advance.
- **The passing run's ledger**: 6 hits for 24.8 the whole walk, end health 75.2, the pair
  one-shot each (the west corner #12 at 6.9 u, the walkway south 1 #13 at 1.3 u), ch 0x1E +
  exit 0x07000006 at t11844 [L the 2026-11-18 pass].
- All levers are `difficulty >= 3`-gated in the 05 file only; the d1/d2 sisters keep the master
  cycle and their proven rolls (whole-binary run above: the sisters passed unchanged).
