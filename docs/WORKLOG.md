Plan: branch from origin/master 8de76e31; fast suite daemon started in the first minutes; read
SP_Hit/SP_Update decompile, census mask-bit-2 SPs on disc, port the bullet arm, real-input tests;
touched crates + suite; report.

### level-07-urn-break (started 08:26 UTC, 2026-10-25)

Task: placement #312 OBJECT_UrnBreak (script 0x0600007A, mode 64, mask 2) on 07000007 is a script
player that should start when shot. SP_Hit 0x1dc470 bit 2 fires on class-5 HITDATA against the
current stream objects' +0xD0 chains; the port's held_hit_targets skips hit_points-0 players so a
gunshot can never start the urn. Port the bullet arm for every mask-bit-2 SP, census the disc,
pin with real-input tests (urn shot -> state 1->2, unshot stays primed, 07000012 sky bridge, 07000006 radio).

- (08:26) Branch from origin/master 8de76e31, pushed. Suite daemon started in the background.
