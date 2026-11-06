# WORKLOG

## sound-gates-sweep

Branch cut from origin/master d6245281a. Task: sweep every PORTED row of
docs/systems/sound-coverage.md — for each, read the branch conditions around the call in the
decompile (work/decomp/ACTION.ELF), name the flag or state that reaches it, and confirm the
port fires on that same event; trace flags to their writers instead of trusting the row's
label. The `Player_HandleJump` 0x1916b0:191-198 head-bump/landing mislabel (fixed by
head-bump-surface-sound) is the pattern to hunt: right id, wrong event, so an id-level test
passes while the game plays nothing there.
Plan: six parallel audits over the table's PORTED rows (player, weapons/bullets, world objects,
drones/music/script), each row read against the decompile branch and the port's gating code;
claimed WRONGs re-verified in disassembly (capstone MIPS64, scripts/mipsdis.py) before any
change; each fix lands with a test that drives the event through the real input path; the doc
rows corrected with the citation. Deliverable: one MATCH/WRONG line per row appended to this
entry, fixes + tests, report in docs/reports/sound-gates-sweep.md.
Progress: setup verified (work/, ISO, nf-queue present from an earlier session; branch created
fresh from master d6245281a). Sweep started 12:59 UTC; deadline ~14:54 UTC, no new work 14:34.
