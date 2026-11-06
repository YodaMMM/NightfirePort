# Sound rows the sweep left wrong

Branch from current `origin/master`, push to `s/sound-sweep-leftovers`. The sweep's report is in the history of `s/sound-gates-sweep` (docs/reports/sound-gates-sweep.md). Fix the rows it marked WRONG and left unfixed, including `Sound_StopAllWithId` 0x12/0x3D/0xF0 (0x190630:83-85, 0x1910f8:41-43), 0x5EF Oddjob's hat (0x12c5a0:286-290), and the row at the report's line 201. Each with a test that drives the event on a real level, proved red on the old code. Read the branch condition and trace flags to their writers.

---
claimed-by: sandbox7 at 2026-11-06T20:08Z
