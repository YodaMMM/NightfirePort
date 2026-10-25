
### level-0a-rain-indoors (started 05:43 UTC, 2026-10-25)

Task: 0700000a — rain streaks fall indoors, in the dark office lounge at Bond
(-37.50, 1.11, -16.95, yaw 2.827, pitch -0.092), frame 2719. Rain belongs outside the windows
only. (1) Find how the game confines its rain [M — the weather/rain emitter's creator and
update, its volume/box or room gate, any indoor/outdoor flag on the room or the camera's room,
the particle cull] and why the port draws it in this room. (2) Fix it [M]; a test that no rain
particle draws with the camera in this room, and a sister that rain still draws outside (0b's
rain stays as it is — the user already checked 0b's rain as right). Headless render of the spot
named in the report.

Plan: branch s/level-0a-rain-indoors from origin/master 38f1bc93; reuse the sandbox's existing
work/ + ISO + nf-queue; fast suite runner started early; census the rain emitter + particle cull
decompiles [M] and the 0a weather setup [D]; fix + tests; report + finish.

