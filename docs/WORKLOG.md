# WORKLOG

## level-07-guest-voice

- 2026-10-25T02:20Z sandbox: branch created from origin/master (11e19082), then merged origin/s/level-07-guest-talk (2ae22f68) — the task's premise (talk gates live, `walking_up_to_a_guest_acks_the_talk_gates_and_does_not_scare` present) only exists on that branch. Task: port the facial arm of NDrone2_SpeechSFX 0x141b00 (facial anim attach / +0x4c script id with +0x48=now+0x14 delay, SFXPreCache of AnimScriptPeekSfx(facial), sound from the facial script's own sound command, cooldown advance once +0x53c stands [M 0x1446b0 case 0xd tail]), re-derive the two speech_facial_fallback_live pins, port the head turn (HeadTrackObj(player,200) 0x1446b0/0x144488), extend the walk-up test with line-id + sound-emitted + head-track pins. Setup in progress (work-slim + ISO downloads).
