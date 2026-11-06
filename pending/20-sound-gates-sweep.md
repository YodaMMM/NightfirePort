# Sound rows: right event, not just right id

`Player_HandleJump` 0x1916b0:191-198 was listed and ported as a head bump. It is the jump landing: the arm is gated on flag 8 (feet on ground), while a head hit is flag 0x20 and silent [M :68-71]. A live trace confirmed it. The id was right and the event was wrong, so a test on the id passed.

Sweep every PORTED row of docs/systems/sound-coverage.md. For each one read the branch conditions around the call in the decompile, name the flag or state that reaches it, and confirm the port fires on that same event. Trace flags to their writers instead of trusting the row's label.

Report one line per row: MATCH or WRONG with the citation. Fix each WRONG row with a test that drives the event, and correct the row.
