# WORKLOG

## movie-skip-keys (started 2026-11-05 08:12 UTC, sandbox5)

Task: on PC only Confirm skips the start movie and the attract trailer; the keys mapped to
action 0x19 work only on state-6 movies. Find the game's skip rule for each movie kind in the
decompile and make every kind accept the same inputs through the real App key path. Test each
at 240 Hz with the 180-tick grace. Cite [M] addresses.

- Branch cut from origin/master (27c191518) and pushed.
- Plan: 1) pin the game's per-kind skip rule from the decompile (frontend/App state machine,
  movie players); 2) read the port's App key path (keys → actions → per-kind movie skip);
  3) failing test through the real App key path; 4) fix; 5) 240 Hz tests with the 180-tick
  grace for every kind; 6) full suite; report.
