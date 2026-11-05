# Civilian alarm re-pin, part 2

Continue from `origin/s/civ-alarm-repin`, push to `s/civ-alarm-repin-2`. Read docs/reports/civ-alarm-repin.md first. Test-side changes only unless a walk exposes a real mismatch with the decompile.

Still red: walkreal_07000009_live (5), walkreal_0700000a_live (4), walkreal_07000011_live real_damage and _agent, walkreal_07000013_live real_damage, walkreal_07000006_live real_damage and _agent.

The 09/0A walks must play stealth as a player does: drop security guards before they ack, or stay out of sight. Keep "must not raise the alarm" asserted.

Do not loosen a pin tagged [L]. Live figures from level 11 with Bond idle at default difficulty: 12 damage a hit, 6 hits by frame 1500.

Run each binary whole and quote results.

---
claimed-by: sandbox1 at 2026-11-05T20:19Z

---
## Report (done, 2026-11-05T20:49Z)

