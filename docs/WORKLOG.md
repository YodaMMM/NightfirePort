
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
