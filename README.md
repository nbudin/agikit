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

`agikit build path/to/source/files output/path`

To auto-format a LOGIC script:

`agikit formatLogic path/to/scriptfile.agilogic`

## Current status

agikit is very, very early stage right now.  As of this writing (version 0.1.0), it's only been
tested against the MS-DOS version of King's Quest I (2.0F).  The extracted and rebuilt game seems to run fine under ScummVM.

Known limitations:

* Only supports AGI version 2 for now
* Compiled LOGIC code can't necessarily be decoded by AGI Studio, because the compiler doesn't (yet) guarantee that all blocks fully contain their child blocks
  * This might be more of a limitation of AGI Studio's decompiler
