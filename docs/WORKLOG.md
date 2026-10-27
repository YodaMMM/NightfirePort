# WORKLOG

## level-4a-ride-bond-float — started

Branch `s/level-4a-ride-bond-float` from origin/master (6ff39ea9). Task: Bond floats above the
deck in the 0700004a lift-ride landing cut-scene (pack 0x060006cf, SP #289 base (0, -53.53,
-3.38)); find his stream + sequence roots [D] and the game's placement rule [M], port it scoped
so 07000014 stays unchanged. Predecessor: level-4a-ride-cast-b (de77afd4) decoded the
anim-object placement and pinned the car-local roots; its live checks were the compose-site
discriminator. Note: origin/master no longer carries docs/WORKLOG.md (stripped by
batch-1025g/4c1f5ffb process-file cleanup); this file is recreated fresh with this entry.

Note for the decode: origin/s/cutscene-kind31-frame-2 (5f55744d, not yet on master) landed a
closely-related rule — an unparented case-1 stream concats si+0x00, which holds the SP
placement base from Script_Load [M 0x1d8248:148] until Script_CameraStart overwrites it with
the identity, never restored [M 0x1d916c, 0x1db348]. Reading that diff before touching nis.rs.
