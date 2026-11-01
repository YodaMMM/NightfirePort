
## level-06-start-subtitles — 2026-11-01

Task: at the 07000006 start the PS2 shows the driver dialogue subtitles ("Can't I rest a
moment?...", "Then it's six hours back, old man!...") [L]; the port shows none. First check the
port's subtitle option default against the game's [M the settings init / save default]; if the
default is on, find where the dialogue's subtitle lines come from [D the script streams / text
bank entries for that conversation; M the subtitle post path] and port it. Pin: the lines post at
the script's ticks with the option on, none with it off. Run the subtitle/script binaries you
touch and nf-game; quote the totals.

Session 1 (2026-11-01): branch from origin/master (3f760e7d); sandbox kept from the previous
task (repo, work/, ISO, nf-queue worktree). WORKLOG started.
