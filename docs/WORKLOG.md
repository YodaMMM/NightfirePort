# WORKLOG

## audit-nis-ops (2026-11-05)
Started 2026-11-04 22:35 UTC. Branch s/audit-nis-ops (cut from origin/master b2cd660b3).
Task: census every script (NIS) op used on the disc by on-foot levels, per level, against what
the port's script player does with each op; name each op the port decodes but ignores or only
partly runs, how often it appears and on which levels, and what the game's handler does [M];
fix those with a clear player-visible effect (objects, sounds, fades, text, channels) and pin
each with a test on the real level; report the rest as a table.
