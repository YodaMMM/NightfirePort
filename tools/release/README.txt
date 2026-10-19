NightfirePort {VERSION}
=======================

A native PC port of 007: Nightfire (PS2), rebuilt from the game's own data and logic.

You need your own copy of the game: a disc image (.iso) of 007: Nightfire for the
PlayStation 2, USA release (SLUS-20579). Nothing from the game is included here.

Getting started
---------------
1. Unzip this folder anywhere you can write to (not Program Files).
2. Run NightfirePort.exe.
3. The first time, choose your disc image (or drop the .iso onto the window).
   The launcher checks it and remembers it.
4. Press PLAY.

nf-view.exe is the game itself; the launcher starts it for you.

Updates
-------
Each time you start NightfirePort.exe it checks for a newer release and installs
it before PLAY. If it can't reach the internet it just starts the installed
version.

Online multiplayer
------------------
Host or join from the game's multiplayer menu. A host listens on UDP port 27420
by default; others can join it by its address and port. If friends can't reach
your game, forward UDP 27420 on your router to your PC.

To list games through a lobby server, create the file
%APPDATA%\NightfirePort\net.txt containing one line:
    lobby_server = <server address>:<port>
(ask whoever runs the server for its address).

Files
-----
Settings, saves and logs live in %APPDATA%\NightfirePort.
If the game won't start, nf-view.log there has the details.

Licences
--------
THIRD_PARTY_NOTICES.txt and THIRD_PARTY_RERAC_LICENSE.txt list the open-source
software included in these programs.
