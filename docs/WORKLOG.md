# WORKLOG

## audit-voice-triggers
Started 2026-11-04 23:29 UTC. Branch s/audit-voice-triggers (cut from origin/master f503c15c7).
Task: every caller of the drone voice path in ACTION.ELF (Drone_SfxInfo lines, speech and bark
functions): the trigger, the condition, the repeat delay, the line; whether the port plays it
on that trigger. Include allies, civilians, bosses and the player. Fix every missing or wrong
trigger with a clear port site, pin each with a test on real level data; report the rest as a
table. Merge origin/master first.
