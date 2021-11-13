# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.1]
 - Fix bug where broken pipes would color the shell. For instance, if the last line in some output patched, running
   `hline <pat> myfile.txt |head` would color your terminal red. Oops!
 - Made error message output a bit more human-friendly.

## [0.1.0] - 2021-11-07
 - Initial public release 🎉

[0.1.0]: https://github.com/ollien/hline/releases/tag/v0.1.0
[0.1.1]: https://github.com/ollien/hline/releases/tag/v0.1.1