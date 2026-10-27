
cutscene-kind31-frame-2 — started (2026-10-27 ~15:15 UTC)

- Goal: replace level-14-capture-placement's per-script scope (script 0x0600062F's unparented
  kind-0x31 streams excluded from the SP base compose) with the general rule the game applies
  [M Script_Interp 0x1d9a08 case 1 :91-138, Script_CameraStart 0x1d9140's base reset,
  Script_EventHandler 0x1d94c0 case 6, Script_Free 0x1da880:44-48]. Census every SP script's
  kind-0x31 streams [D]; the 01 intro (0x06000087 s5/s11) and 14 capture tests stay green.
