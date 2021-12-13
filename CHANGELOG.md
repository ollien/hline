# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.1] - 2021-12-12
### Changed
  - Made `Error` enum non-exhaustive to promote future expansion.

## [0.2.0] - 2021-11-17
### Added
  - Added binary file detection. When a binary file is detected, `hline` will refuse to highlight it, unless passed the `-b` flag.

### Fixed
  - Fixed inconsistent error output
  - Fixed a panic when non-utf-8 data was encountered

## [0.1.1] - 2021-11-13
### Fixed
 - Fix bug where broken pipes would color the shell. For instance, if the last line in some output patched, running
   `hline <pat> myfile.txt |head` would color your terminal red. Oops!
 - Made error message output a bit more human-friendly.

### Changed
 - Change highlight color to light red

## [0.1.0] - 2021-11-07
 - Initial public release 🎉

[0.1.0]: https://github.com/ollien/hline/releases/tag/v0.1.0
[0.1.1]: https://github.com/ollien/hline/releases/tag/v0.1.1
[0.2.0]: https://github.com/ollien/hline/releases/tag/v0.2.0
[0.2.1]: https://github.com/ollien/hline/releases/tag/v0.2.1
