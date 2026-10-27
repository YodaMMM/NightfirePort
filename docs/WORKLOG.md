
cutscene-kind31-frame-2 — started (2026-10-27 ~15:15 UTC)

- Goal: replace level-14-capture-placement's per-script scope (script 0x0600062F's unparented
  kind-0x31 streams excluded from the SP base compose) with the general rule the game applies
  [M Script_Interp 0x1d9a08 case 1 :91-138, Script_CameraStart 0x1d9140's base reset,
  Script_EventHandler 0x1d94c0 case 6, Script_Free 0x1da880:44-48]. Census every SP script's
  kind-0x31 streams [D]; the 01 intro (0x06000087 s5/s11) and 14 capture tests stay green.

cutscene-kind31-frame-2 — the general rule replaces the per-script scope (2026-10-27)

- The rule [M]: an unparented case-1 stream ALWAYS concats si+0x00 (the GetInterp 10th arg is
  (parent == -1) [0x1d9c0c; tail Concat 0x1da838-0x1da844]) and si+0x00 is the SP placement base
  from Script_Load [0x1d8248:148] until Script_CameraStart overwrites it with the identity —
  binary-proven a0=si at 0x1d916c — never restored [0x1db348]. No per-kind branch exists
  [Script_CreateEntity 0x1d8968:35-173]. Port: keys_alone = camera_started && parent < 0.
- Census [D nis_kind31_census.rs]: 149 scripts with kind-0x31 streams; 0x06000087's camera op
  ticks at 0 (s5/s11 world-frame; s11 has NO keys — first_key 3893 > last_key 3892 — so the
  event-6 hand-back snapshot is its create-time identity, unchanged by the rule); 0x0600062F as
  level-14-capture-placement established. 27 WORLD rows, the rest in no-camera scripts.
- Green: the 01/14 real-path battery, nf-game 933/0/8, the cutscene nf-view chunk 373/0/18.
  Open: walkreal_07000008_live timeout at the gondola leg (no baseline; 07000008's 0x31 scripts
  have no camera op).

cutscene-kind31-frame-3 — started (2026-10-27 ~18:57 UTC)

- Goal: frame-2's general rule `keys_alone = camera_started && parent < 0` turned five real-path
  tests red: walkreal_07000008_live (gondola floor plate ch 4 [D #236 mask 8] x2), zoe_gondola_realpath_live
  (same), mayhew_hold_live (escort never parked in AllyLeadMissionWait 0x21), escort_hide_findrun_live
  (route index 2, want 1). Find which unparented stream in which camera script reads keys-alone
  where the game composes (or vice versa) — scene camera op tick vs stream op tick, SubScriptStart
  base reset [M 0x1db050:28-38], SP_SetPosRot [M 0x1db920] — fix the rule to the game's behaviour,
  keep frame-2's 01/14 battery green.

## cutscene-handback-census (branch s/cutscene-handback-census)

Census of every SP script's event-6 hand-back on every on-foot level [D 0700* archives, kind-0xd9
placements]: each script runs through the real port runtime (Script::load at the placement base →
update ticks → free) and the Request::PlayerEnable snapshot is checked against the level's own
static collision (floor within 2.5 below). 26 hand-back rows over 14 levels, 12 fallbacks, 0 voids;
the two 4a rows match level-4a-lift-4's [L] landing-pad pin (the "missing floor" is the lift
platform, a dynamic object) and are pinned as a named position-equality exception
[nis_handback_census.rs]. Findings: (1) kind-2 (anim) prev streams — the game snapshots
anim_obj+0x90 (live via AnimObjectUpdate 0x1befd8:290 → Control_GetObjMatrix 0x134210) but the
port falls back; four rows hit it (0600007f/06000084/06000080/060006d3), documented not pinned;
(2) 0x06000074 (07000005 SP #71) is on no archive [D probe] — the missing-script arm is correct;
(3) the frame-2 rule holds disc-wide (every snapshot row's script has cam@0 → keys-alone).
Code: one read-only accessor (Script::stream_kind). Ran: nf_formats 91/0, nf_game lib 578/0/5ign,
nf_view lib 96/0, the 14-bin cutscene-sensitive integration set 13 green (walk_07000014_live
rc=137 = sandbox OOM kill, not a failure); the remaining ~845 bins did not fit the window
(background jobs are killed between tool calls here — foreground chunks only).
