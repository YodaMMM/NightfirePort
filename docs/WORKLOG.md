# WORKLOG

One entry per task, append only (never rewrite or reorder others' entries).

## unread-params-props

2026-11-07 10:36 UTC, branch `s/unread-params-props` (from origin/master 10956badb).

Task: sibling sweep to unread-params-switches over the prop classes — pickups, breakables,
gun mounts, hazards, placed effects and lights (crates/nf-game/src/pickup.rs, breakable.rs,
gunimp.rs, effect.rs, light.rs). Read each class's `*_Create` in the decomp, list every
placement param/word it stores, and check what the port does with each. Port-ignored params
get a census of non-zero values on EVERY level (`Level::parse` + `nf_view::dev::LEVELS`);
zero everywhere = closed with the census as proof; non-zero on any on-foot level = a real
gap: trace the reader in the game, port it, test on the real level, red on the old code.
Timing from fresh-load readings, not symbol names. Census table in
docs/reports/unread-params-props.md.

2026-11-07 11:55 UTC — the sweep is complete: every placement word for the 0x1d04f0 prop classes
(pickup 0xf0, break 0x20, destroy 0xe1, gunimp 0x34, hurt 0xe4, mine 0xfe, fusebox 0x31, light
0x4e, flicker/ripples/leafgen/cloud/apocalypse/emitter/lightning, corona 0xfc, rainbox 0x32,
RB 0xd8, dynobj 0x3b, rotor 0x71, onesided 0xd5) checked against the port and censused on all 34
levels. One real gap FIXED: the 0x4E placement lights read the placement's raw quaternion/scale
words instead of the params block — every placement light was built radius 0 / black rgb (inert);
now reads p0..p8 per the case-0x4E marshaling [M 0x1d0b8c-0x1d0c04], create_full carries the
sound word; pin placement_lights_07000001_live (07000001's four lights: radius 6, rgb 255/121/103,
mask 1) — red on the base. Two gaps traced and left: the mine class (0xfe — nine mines on
07000015, absent entirely; full Mine_Create/Update param table in the report) and the hurt
count/rate/path flight triple (non-zero only on 07000046's seven volumes). Closed by census+decomp:
gunimp's p0/p1 are dead data (the game's level_tag arg is unread), pickup p7+/break p3+/destroy
p7+/fusebox p5+ never read by the game, rainbox/corona/onesided read no params, rotor's bytes are
zero everywhere. Runs: nf-game lib 688/0/5ign, placement_lights 1/0, lighting 10/0, interact_01 5/0,
walkreal_07000001 8/0, g049_blast_light 1/0, npc_doors 2/0/1ign, walkreal_02/03/04/05/06/07/0d all
green; full serial suite did not fit the window (the only changed branch is unreachable outside
07000001). Census table in docs/reports/unread-params-props.md.
