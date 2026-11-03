# text-send-message — the game's on-screen text message path

Text_SendMessage (456 bytes) and Text_SetupChar are never mentioned in crates/*/src. Read their callers in the decompile: which in-game messages go through them, and whether the player sees each one in the port with the same text, position and duration. Port what is missing with addresses, or cite the port function that replaces it. Done = a real-input test for one message on a level that sends it; the HUD/message tests green; the full suite total line.
