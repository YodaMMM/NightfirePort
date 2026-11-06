# Equinox: the missile countdown onto master alone

The fix is proved and it should not wait for the aim branch. Live, PCSX2 0700001b with channels 193/194 held from frame 2256: MissileDeploy goes m5 at +1, m4 at +1200, m3 at +2401, then m5 to 3 at +1799 and m4 to 3 at +2999. That is seed x 60 with the seeds 105/105/40/20/0/60/105/60 by index.

Branch from current `origin/master`, push to `s/missile-countdown-master`. Take only: `COUNTDOWN_STEP` = 1/60 and the corrected seed order (085b88ca7 on `s/aim-route-4`), and the test re-pacing it needs on master's raw aim: CONSOLE_ORDER, the phase cap, `drake_fight_live`, `interactables_0700001b_live`, the klaxon test (bec12dff9, 8e4094c45). No aim snap code and no other aim-stack commit.

Master has no aim snap, so its console shots may behave differently from that branch. Goal: `walkreal_0700001b_live`, `drake_fight_live`, `interactables_0700001b_live` whole and green on master. Report any Drake-fight red with its cause. Test-side changes only besides the constant and seed order.

---
claimed-by: sandbox3 at 2026-11-06T18:45Z

---
## Report (done, 2026-11-06T19:36Z)

Closed by the driver: the report is on `s/missile-countdown-master` (docs/reports/missile-countdown-master.md); the worker didn't run finish.sh.
