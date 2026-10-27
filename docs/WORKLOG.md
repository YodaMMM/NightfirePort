
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

cutscene-kind31-frame-3 — the hash-0 carve-out on the general rule (2026-10-27 ~20:40 UTC)

- Frame-2's rule turned five real-path tests red. Bisect (a temporary NF_KA_CLASS knob, removed):
  kind-0x31/0x19 WITH hash going keys-alone keeps mayhew_hold green; HASH-0 entities going
  keys-alone turns it red. The disc census (new nf-formats tests): the only hash-0 entity streams
  in camera scripts are the hand-back ANCHORS — 0x06000087 s11 (hash 0, 0x31, op@2118, ONE key;
  keys[a..=b] is INCLUSIVE so frame-2's "s11 never keys" was the camera's exclusive-convention
  misread) and 0x06000077 s15 (hash 0, 0xff, op@3870, ONE key) — each one tick before its
  event-6 hand-back (EventHandler 0x1d94c0 case 6 snapshots the PREVIOUS stream's obj+0x90).
- Fix: keys_alone = camera_started && parent < 0 && !(hash == 0 && kind != 0x19) — the gfx-bound
  rule keeps frame-2's world-frame reading (07000014's doors), hash-0 non-0x19 anchors compose
  (five tests), the hash-0 0x19 marker skip keeps its 4a [L] pin.
- Green: the five (mayhew 1/1, escort_hide_findrun 3/3, zoe 1/1, walkreal_07000008 4/4 — the
  frame-2 gondola-timeout open item resolved) + the 01/14 battery (walkreal_07000001 3/3,
  interact_07000001 10/10, exit_aim_01 1/1, capture_placement 1/1, walkreal_07000014 2/2,
  walk_07000014 2/2 — the merged tree has 2 tests, exit_aim_hint 2/2, probe14 ignored).
- Open: the [M] restorer of si+0x00 between the camera op and the anchor tick is NOT derived;
  the full fast-runner suite did not fit the sandbox window (background processes are reaped
  between tool calls here); tools/test/suite_resume.py is the resumable driver for the next run.
