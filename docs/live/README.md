# docs/live — ground truth from the real game

Records of what the retail game (007: Nightfire, PS2 NTSC-U) actually does, measured in
PCSX2 with the trace tools (`tools/trace/pcsx2_shot.py`, `gdbwatch.py`, `gdbcall.py`, `fieldprobe.py`,
`pad_log.py`, `goto_level.py`, PINE). Files are dated measurement sessions; the port's fixes cite them as
[L] evidence.

- `2026-10-13-portal-ground-truth.md` — portal culling at three placed spots (06 courtyard, 07 window,
  07 clock hall), the game's visible-cel list, the chain of cel 0x0179fb30, and the forced-objects refutation.
- `2026-10-13-weapon-modes-pcsx2.md` — the fire modes of every weapon in a multiplayer match (the Sentinel's
  guided camera, MGL impact vs delay, the 32-weapon mode table, the mode button).
- `2026-10-14-weapon-measurements-pcsx2.md` — live measurements: rocket speed, grenade release timing, the
  stunner, the laser watch, the remote mine, the AIMS-20, SG5 casings, guard sighting→first shot, per-level
  loadouts and gadget cycles.

## The backlog rule

Every open question that needs the real game lives in **`backlog.md`** — one ledger row per question, with its
procedure (level, spot, tool, what to read), the expected answers and what each would change in the port, the
evidence so far, and a status (open / answered / obsolete). It collects the open live checks of the docs and the `UNKNOWNS.md`/`GAPS.md` rows that need live play.

New live checks get a row id (L-NNN) in `docs/live/backlog.md`. When a check is answered, its row is marked
`answered` with `SETTLED:` naming the `docs/live/` file that records the answer.
