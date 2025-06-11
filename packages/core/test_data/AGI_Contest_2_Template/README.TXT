AGI Template Game
-----------------
Version 1.11
By Peter Kelly
15 February 1998

The template game is an AGI version 2 game which you can use to build other
games from. It contains one room, and all the basic functionality you would
expect in an AGI game such as menus, key assignments, loading/saving and a
debug mode. Using this template as a starting point for other games will be
easier than modifying an existing game because it does not contain lots of
game-specific code.

Before attempting to program an AGI game, I suggest you have a look at the
AGI Specs documentation (http://www.ozemail.com.au/~ptrkelly/agi/specs).
This will tell you a lot about how LOGIC programming is done, and contains
an excellent command reference which you will find very handy. Also, a good
way to learn the language is by looking at the logics of existing games
(including this one!).

Compiling
=========
You will need a compiler to compile the logics in this game. There are three
compilers available at the time of writing: MATS, AGIC and AGI Studio. The
syntax used by this version of the template game is now the "official" syntax
that all logic decoders and compilers support, so this game should compile with
any compiler.

(Note: At the time of writing, MATS and AGIC do not fully support the new
 syntax. They should eventually support it though.)

Running the game
================
This package is released without an interpreter. The reason for this is because
the interpreter is written by Sierra-online and it could possibly be a violation
of copyright laws distributing their interpreter with the game. Because of this,
you will need to use an interpreter from another game. The only DOS versions of
the interpreter that should be used is 2.915, 2.917 or 2.936. Others will most
likely not work.

If you do not have one of the interpreters mentioned above, you can obtain one
from one of the demo packages that are available (there are some on my website).
Make sure you get the right interpreter version.

To use the game with an existing interpreter, just copy the files distributed
with the game into the same directory as the interpreter and run the loader
program (usually called SIERRA.COM). Alternatively, you can copy all the *.OVL
files, the AGI file and the loader into the directory containing the data files.

Eventually, other interpreters will probably be written capable of running this
and other AGI games.

Distributing the game
=====================
This template game is freely distributable. You can modify it as much as you
want (that's what it's for!) and distribute any games you make from it as you
like. However, if you are distributing it as a template (it's current form)
rather than a game, I request you leave all the files intact, including this
readme.

Also, if you are distributing games made from this I suggest that you do not
distribute Sierra's interpreter with it. As mentioned above, it may be a
violation of copyright laws. I am not sure about the status of the interpreter,
but it's probably safer not to distribute it as part of your game. It is
available in the demo package anyway, which is freely distributable. As far as
I can see, there is nothing wrong with creating your own data files which work
with the interpreter, as with the many other game editors that are available.

A note about game IDs
=====================
The DOS versions of the interpreter include a "Game ID", which is a short string
identifying the game they are supposed to run. Somewhere in the logics the game
is supposed to set the game ID. When it does this, the interpreter compares this
with it's own game ID. If they are different, the interpreter quits instantly.
This was probably to stop people from running games with the wrong version
interpreter, which could mean the game wouldn't run properly.

However, I have discovered that there is a very easy way around this, and that
is to simply not set the game ID at all. The game will run fine, and the saved
games will be called "sg.1", "sg.2" instead of having the game ID in front of
the name, e.g. "sqsg.1", "sqsg.2". This template game does not set the game ID,
and you should also not do this, so that your game will run with any interpreter
(provided it is the right version).

Operation of this game
======================
Looking at all the code that is required for just this small template game can
be a bit daunting at first, so it helps to understand what it is all doing. I
will attempt to explain this a bit. Read the following section with the logic 0
alongside so you can understand what each piece of code is doing. You should
already know a bit about variables, flags, logics etc.

When the interpreter first starts, it loads logic 0 into memory and executes
it. The interpreter operates in cycles, and on each cycle it runs logic 0
and updates any screen objects that are visible. Logic 0 calls other logics
during it's execution, most importantly the logic of the current room, whos
number is stored in var 0.

First off, we check the value of error_code. If this is greater than 0, an error
has occured and the error handling logic (98) is called, which displays an error
message and quits the game.

Next we check the value of v0 (the current room number). If this is 0, the
game has just started so logic 91 is called to do some initialization. If
game_restarted is set, then we are ready to go straight into the game. If so, we
go to the first room which is 2. Otherwise, we set up the menus and go to the
intro/opening screen (room 1).

Then we check to see if the flag new_room is set. If so, this is the first cycle
in a new room and there are some things that need to be set up. Since all
resources other than logic 0 are unloaded when the current room changes, we need
to load the resources that we need again. First we load logic 90, which contains
various bits of game-specific code. Then we turn off debug, and set up ego.

Then we check the value of death_type. If this is greater than 0, then the
player is dead so we disable some menu items and then call logic 94 (the
death handler). Logic 94 looks after anything that happens after the player
is dead. Also, if the player is dead, we skip to the end of the logic as we
don't want them doing any other stuff like picking up things or talking to other
characters.

Next we check for menu events. If the player has pressed Esc, then we activate
the menu with the menu.input() command. We check for and respond to any menu
commands like saving or restoring games or changing speed. We also check for
commands like save, restore etc. These are only checked for if the flag
disable_game_functions is not set (so if you want to stop the player from
doing these things, for example during intros/cutscenes, you can set this flag).

After that we have to check the animation of ego. If ego has stopped, then we
must unanimate him/her. To determine this we compare ego's old position with
their current position. There are also two flags, always_animate_ego and
never_animate_ego which can force the animation on or off.

If the player wants to turn the clock on or off, we check for this and act
accordingly. If the clock is on, we update it.

If they have restored a game, we must now clear the bottom lines of the screen,
turn debug mode off and disable the file separator menu item again.

The next thing that is done is one of the most important things that logic 0
does. The logic for the current room number is called. Because logic 0 is
executed on every cycle, and the logic for the current room number is called
from logic 0 every time, the current room's logic also loops around like
logic 0. Then, if debug mode is active, we then call the debug logic (99).
Also called is logic 90 which performs any game-specific functions.

After that we check to see if the player has typed in an unknown word. If so,
we tell them we don't understand the word they've typed. You might like to put
some funny responses in here.

Then, if the player's input has not yet been parsed, we tell them that we don't
understand what they've typed and perhaps suggest that they try rephrasing it.

This is the end of logic 0. The other logics are fairly self explanatory, and
are fairly easy to understand once you know how logic 0 works.

Resources used by this game
===========================
Picture 1 - intro text background (black)
Picture 2 - first room background (white)
View 0 - ego
View 1 - dead ego
View 220 - test object
Logic 0 - main logic
Logic 1 - intro screen
Logic 2 - first room
Logic 90 - game specific functions
Logic 91 - initialization
Logic 92 - help
Logic 93 - debug mode help
Logic 94 - player death handler
Logic 95 - trace info (this script contains a list of commands used in tracing)
Logic 98 - error handler
Logic 99 - debug script

Variables, flags and controllers used by this game
==================================================
Variables:
 30 - new_ego_x
 31 - new_ego_y
 32 - old_ego_x
 33 - old_ego_y
 34 - old_ego_dir
 35 - death_type
 36 - thankyou_timer (used by death handler)
 37 - old_clock_seconds

Flags:
 30 - never_animate_ego
 31 - always_animate_ego
 32 - debug_active
 33 - disable_game_functions
 34 - clock_active
 35 - coords_shown

Controllers:
 1  menu item - quit game
 2  menu item - help
 3  menu item - save game
 4  see object (key)
 5  menu item - restore game
 7  menu item - restart game
 9  echo line
 10 menu item - inventory
 12 menu item - clock on/off
 14 enter debug mode
 15 menu item - configure joystick
 16 menu item - sound on/off
 17 clear input line
 18 menu item - pause game
 19 activate menu
 20 menu item - separator on file menu
 21 menu item - about
 22 menu item - see object
 23 menu item - fast speed
 24 menu item - normal speed
 25 menu item - slow speed
 26 decrease volume
 27 increase volume
 28 menu item - fastest speed

Changes since v1.1
==================
This is just a minor bug fix. I found out that when the player restarts the
game, the separator on the file menu is enabled so that the player can select
it. This was fixed by adding the line "disable.item(menu_fileseparator);" near
the top of logic 0 (you'll see it there after "if (game_restarted) {").

Changes since v1.0
==================
There have been number of changes since the last version, mainly due to the
new syntax and the ability to use the #include command (all variable names are
now defined in defines.txt). There are a few other things, however:

I have removed all computer-specific code from the game - things such as
different key assignments for different machines, etc. I figure all new games
will either be run under Sierra's DOS interpreter or a new interpreter written
for unix, mac, windows etc. which will act like the DOS interpreter and map the
appropriate keys across. However, I have left support in for sound volume
control as this may be supported by new interpreters.

Also removed from the game is any reference to "Template game" or "by Peter
Kelly". This is because it is supposed to be just that - a template game, and
you want to be able to start off afresh without having to remove anything
that's already there. However, I do suggest you remove the part when the player
dies when they type "die" (this can be found in logic 90). This code is in there
to demonstrate how to kill the player, and is not meant to be part of an actual
game. Also, at some point early in the development of your game you will want
to change the game_version_message and the game_about_message to something
suited to your game. These can be found at the bottom of defines.txt (you will
need to recompile logic 0 after changing these).

The flags debug_active, clock_active, never_animate_ego, always_animate_ego and
disable_game_functions have been changed from f22, f24, f20, f21, f33 to f32,
f34, f30, f31, f33 respectively. This was to avoid possible conflicts with
"reserved" flags (the highest one of these known so far is 15, but they may go
higher).

Logic 0 (main logic):
-Added clock on/off to special menu and removed "change graphics mode"
-Fixed errors where || was missing
-Changed if (v25 = 1) to if (v25 == 1) in place where test object is shown
-added clear.lines(24,24,0); after reset(debug_active); to clear the line which
 shows ego's coordinates when in debug mode.

Logic 90 (game specific code):
-Added a test at the start to see if unknown_word_no (v9) was equal to 0. When
 this test was missing and the player typed in "look <something>", where
 <something> was not in the words.tok file, they would get two error messages
 ("What? Where?" and "I don't understand <something>"). This was also the case
 with "get <something>" and "use <something>" and any other generic responses
 that had been added.

Logic 91 (initialization):
-Added key assignment for clock on/off (F6)
-Fixed assignment for + and - keys (for increasing/decreasing volume) which were
 the wrong way round (not that it matters for Sierra's DOS interpreter, but it
 might for new interpreters with sound card support).
-sound_volume (v23) is now set to 15 (the highest allowable value) on startup.
 Previously this was set to 0.

Logic 92 (help screen):
-Modified help screen a bit

Logic 93 (debug help screen):
-Condensed a couple of the descriptions in the debug help screen and added
 command to toggle display of ego's coordinates

Logic 94 (death handler):
-Added test for player saying "inventory" as well as testing for the menu
 controller
-Label1 removed and goto replaced with else

Logic 99 (debug mode):
-Added code to show ego's coordinates on screen when in debug mode

I apologize about all these changes to those who have already started writing
games based on version 1.0 of the template game. I suggest these people take
what code they have written so far and move it over to v1.1 of the template
game, as this contains proper names for all main variables used and therefore
is a bit easier to program with (and also because it is in the new syntax). I
do not expect the template game to change again (except perhaps a small bug
fix or two which does not require much modification).

More information
================
For more information on AGI, visit my website:
http://www.ozemail.com.au/~ptrkelly/agi

AGI Specs, a collection of documentation on all aspects of the interpreter, is
available at http://www.ozemail.com.au/~ptrkelly/agi/specs

You can contact me at ptrkelly@ozemail.com.au