# WORKLOG

## mp-bot-voices
Started 2026-11-05 04:46 UTC. Branch s/mp-bot-voices (cut from origin/master 17a784980).
Task: MP bots grunt when hit and call out when they die. Drones::bot_handle_pain (drone.rs) and
BotBrain::bot_sound_effect (mp/bots.rs) hold the line pools but have no production caller and
body.sfx_out on a bot is only cleared. Read BOT_handlePain 0x125ed8, BOT_soundEffect 0x126618,
their callers (DealWithObjHit 0x13fff0:336, BotDeathByExplosion 0x1755a8:15, BotDeathAnim
0x175508:13) and row 2 of "Left as a table" in docs/reports/audit-voice-triggers.md on
origin/s/audit-voice-triggers. Wire both into the MP bot damage/death flow in scene.rs
step_mp_bots and mp/bots.rs with each path's own pool and gates. The two readings disagree on
pain vs death ids (0x14d.. vs 0x154..): settle from the disassembly and report the address.
Bot sound stays on the shared match RNG stream; online state hash must stay equal across
machines. Pin through a real match: a hit plays the pain line, a death plays the death line.
Run mp_world_sync_live, every mp_*_live and nf-game; suite via tools/test/suite.py in the
background + once more on the final tree. Sandbox expires ~06:44 UTC.
