# Guards turn off Bond on contact: the last two walks

Branch `s/latch-walks-2` (on `s/latch-batch`: master + the drone contact-turn rule + earlier repairs) passes everything except two walks. Read docs/reports/latch-walks-2.md on it, sections "Classified, not fixed", "Diagnosed to the door" and "What's left":
- walkreal_07000008_live::walk_the_whole_mission_on_real_input (Operative): guards #7/#8 park in the west-ledge cabin doorway. Run the NF_DOORPROBE-style per-tick log on that door on master and on the branch; decide whether the port lets guards block a doorway the game's guards would clear (door use, contact-turn, the bounds push), and fix the port or re-route as a player would.
- walkreal_07000011_live 00 Agent real damage: guard #33 parks point-blank on the 23.0 strip. Measure the flight-A mouth stand's exposure and fight the #33/#34 pair from there as a player would.

Never lower damage or health, no loosened pins, no new ignores. Branch from origin/s/latch-walks-2, push to `s/latch-walks-3`, report in docs/reports/latch-walks-3.md. Run each touched binary alone, then walkreal_07000001, 07, 08, 09, 11.

---
claimed-by: sandbox3 at 2026-11-09T00:57Z
