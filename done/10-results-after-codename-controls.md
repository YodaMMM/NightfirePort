# results-after-codename-controls — codename creation stops on the wheel

After creating a codename the flow stops on the wheel; the game forwards to Controls [M 0x213288:38-40]. Port that step through the real menu path and test it with key presses from Main: create a codename, reach Controls, carry on to the mission list. Check Esc on Main at the same time: read what the game does there, against the port returning to the title and replaying the start movie, and fix it if it differs.

---
claimed-by: sandbox1 at 2026-11-05T11:06Z

---
## Report (done, 2026-11-05T12:14Z)

Closed by the driver: the report is on `s/results-after-codename-controls` (docs/reports/results-after-codename-controls.md); the worker didn't run finish.sh.
