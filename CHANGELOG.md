# Version 0.8.1 - August 23, 2022

- Bug fix: emit assignn instruction when assigning a #define that points at a literal value (#3)

# Version 0.8.0 - February 19, 2022

- AGIv3 extraction and build support
- New format for .agipic files (a JSON list of commands). Reading the binary format is still supported; editing a
  binary .agipic in Visual Studio Code and saving it will automatically convert it to JSON format. This new format
  makes it easier to transition between AGIv2 and AGIv3 because they use slightly different binary encodings for PIC
  resources.
- Added a new file called `agikit-project.json` at the root of a project. This currently allows setting the target
  AGI version and game ID, but in the future can be extended to support many other options.
- Improved console output for build and extract, with automatic color support detection
- `extract` CLI command now accepts `-l`, `-v`, `-p`, and `-s` options (like XV3 does) for extracting specific
  LOGIC, VIEW, PIC, and SOUND resources specifically
- `extract` CLI command no longer outputs `.dot` or `.agiasm` files by default when decompiling LOGIC scripts. Passing
  the new `-d` option will cause it to output them for debug purposes
- VSCode extension now supports using ScummVM as a "debugger". This doesn't actually add step-through debug support,
  because ScummVM doesn't expose an interface for external apps to do that, but it does allow the VSCode extension's
  build and run workflow to work more like VSCode users expect, with tasks.json and launch.json support.
- Added the beginnings of an automated test suite using Jest. Right now, the only tested code is the LZW and PIC
  compression and decompression, but I hope to expand the tests in the future.
