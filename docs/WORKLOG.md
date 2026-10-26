# WORKLOG

## level-13-audit — the 07000013 behaviour audit (2026-10-26 ~07:10 UTC, sandbox expiring ~09:05 UTC)
Branch start: s/level-13-audit = origin/master (b604242f). Task: walk test for 07000013 passes but
real play finds missing behaviour (characters walk through Bond, guests never talk, doors never
lock, consoles whose action does nothing, cut-scenes with broken cameras). Audit EVERY placement on
07000013's main map [D] (characters 0x0f, doors, triggers, scripts/SPs, switches, monitors/sensors,
pickups, destroyables, breakables, ladders, grapple points, lifts, special objects) against the
game: creator in parsemap_create_dynamic_objects 0x1d04f0 / static path, update + use/damage/event
handlers [M], and the port's handling; same for the level's scripts (every stream's ops, camera
streams, hand-backs) and mission records (objectives, channels, fail conditions). Table goes to
docs/levels/07000013-audit.md with a GAP column; small gaps fixed [M] with tests, large gaps
documented as their own future tasks. Tests pin only proven behaviour ([M]/[D]/[L] evidence).

### level-13-audit — results (2026-10-26 ~08:20 UTC)
The 07000013 per-placement audit is written: docs/levels/07000013-audit.md — 173 main-map rows
(one per placement, every kind vs parsemap_create_dynamic_objects 0x1d04f0 + control_funcs 0x26f890
+ the port at this tree), the channel web (ch 0x56 = two writers, NO reader — the data's own dead
end; ch 0x23's only writer is the elevator SP's completion, E-only via the class-0 entity arm —
docs/systems/elevator-button.md prior art; the exit gate 0x4e and the stairwell door's lock 0xc8
have no writers), the 24 kind-7 scripts (dump_07000013_scripts added to census_07000013.rs; the
intro 060006d7's camera/anim/event ops and the shutter script 06000959's Entity+Event ops quoted),
and the mission record (1 objective, ch 0x50/enable 0x4a). Per-kind re-verification found the
level doc's 0x20 "open ch 0x30" mis-read (p2 is the INVERTED bullet filter [M Break_Create
0x12c230:29-35 + Collide_FilterBullets 0x1ece30]) — §5 fixed. NEW pin: data_web_07000013.rs
(census + channel web off the disc) GREEN. Gaps: GAP-13-1 the drone→Bond bump (msg 0x22) unported
(drone.rs:16544-16551; the port's only push is Bond-side airborne-only [S] player.rs:1821);
GAP-13-2 rotor spin not applied to the draw; GAP-13-3 script events 3/4 fall to the unused generic
arm (0x1d94c0:98-121 vs nis.rs:1002); GAP-13-4 the 0x31 shutter art draw ([S] — live check);
GAP-13-5 guest talk voice-type mapping (live check). ENVIRONMENT FINDING: walk_07000013_live OOMs
on this 4 GB sandbox (3.5 GB anon RSS, kernel global-oom, both invocation shapes, clean relink) —
the full suite could not run here (background jobs die between tool calls; the build phase alone
exceeds the foreground window). Report: docs/reports/level-13-audit.md.
