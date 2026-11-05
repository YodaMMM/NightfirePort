# WORKLOG

## guard-damage-since-1017
Started 2026-11-05 07:20 UTC. Branch s/guard-damage-since-1017 (cut from origin/master a89818235).
Task: standing 10 s on 07000006 costs 188 health and on 07000011 costs 180, against 6.6 and 10
on a 10-17 build; other stands barely moved. Bisect master from 10-17 for the commit(s) that
changed it, same stand at default difficulty. Read each against the decompile [M]: guard hit
roll, range falloff, weapon damage, difficulty scale. Fix anything that differs and pin the
numbers; if the current behaviour is right, say so with the addresses.
