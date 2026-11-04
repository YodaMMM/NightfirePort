# audit-sprite-set-text — on-screen strings outside the message queue

Sweep every Sprite_SetText and Sprite_SetTextNFmt caller in ACTION.ELF for on-foot play and multiplayer (status sprites, timers, counters, pane labels; driving is out of scope). For each: the string or label, when it is written and cleared, the sprite template (read it from ee_start_05.bin: position, font, colour), and the port's draw site. A recent find: the round-end status text was missing for one state of MP_Update, so check every state of each switch that writes one. Fix what is missing or wrong where the draw path already exists, pin each with a test, and list anything needing new drawing as a table with the template values. Merge origin/master first.

---
claimed-by: sandbox4 at 2026-11-04T20:19Z

---
## Report (done, 2026-11-04T20:54Z)

Closed by the driver: the report is on `s/audit-sprite-set-text` (docs/reports/audit-sprite-set-text.md); the worker didn't run finish.sh.
