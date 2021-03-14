# agikit

agikit is a developer toolchain for Sierra's AGI (Adventure Game Interpreter) engine.  AGI was used
in the 1980s to develop adventure games including King's Quest I, II, and III, Space Quest I and II,
and more.  Later, it was reverse engineered by fans and used to develop many fan-made games.
A lot more information about AGI is available at
[the AGI Programmers Wiki](http://agiwiki.sierrahelp.com).

## Goals

agikit aims to provide:

* A composable, extensible, cross-platform set of tools for working with AGI
* Compatibility with other tools, such as WinAGI and AGI Studio
* A platform to build other tooling, such as text editor extensions, language servers, etc.
* A modern compiler architecture that allows for language extensions and optimization

## Visual Studio Code usage

Install the extension from: https://marketplace.visualstudio.com/items?itemName=nbudin.agikit-vscode

For some template code to start with, clone: https://github.com/nbudin/agikit-project-template

## Command line usage

To install it:

`npm install -g agikit-cli`

To extract an AGI game to source files:

`agikit extract path/to/game output/path`

To build AGI game volume files from extracted source files:

`agikit build path/to/project`

To auto-format a LOGIC script:

`agikit formatLogic path/to/scriptfile.agilogic`

## Current status

agikit is very, very early stage right now.  As of this writing (version 0.5.0), it can decompile and compile
King's Quest I, Space Quest II, and the agikit template project successfully.  It possibly works with other
games too, but I haven't tested it on everything yet.

Known limitations:

* Only supports AGI version 2 for now
