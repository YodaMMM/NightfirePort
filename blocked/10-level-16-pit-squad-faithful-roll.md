# level-16-pit-squad-faithful-roll — finish the finale under the game's grenade roll

Branch from origin/master (it carries consider-explosive-gate-2: the game's flee gate and DroneWeap_CanThrowGrenade 0x17a798 to the disassembly).

Two 07000016 walks were switched off when the faithful throw cadence landed: walkreal_07000016_live::walk_the_chasers_defeat_completes_the_mission (Bond runs dry: weapon 0x11 clip 0 spare 0, the chain stalls) and ::the_chasers_punch_lands_the_record_damage (the no-return-fire real-damage walk dies). The finale must stay completable.

Do:
1. Re-run both and read where the pit-squad fight goes now (who throws, who flees, where Bond's ammo goes).
2. Give the walks a player's tactic for the faithful fight: the level's ammo pickups on the route, weapon choice, cover from the throws. No health/ammo writes, real input only. If the port still differs from the game anywhere in that fight, fix and cite it.
3. Un-ignore both; run walkreal_07000016_live, fix_finale_live, every grenade test, then the full suite.

Rules for this task:
- If you change anything under crates/nf-view/tests/common/ (walker, damage_log, walker_b2, ...), the change must be opt-in for your level (a flag your test sets, and the code must check it) and you must run EVERY walkreal_*_live binary before finishing.
- A player-visible fix ships with a real-input test that reaches the spot from the level start (no teleport after load), so the passing test is the proof.

---
claimed-by: sandbox7 at 2026-11-02T20:35Z

---
## Report (blocked, 2026-11-02T21:55Z)

