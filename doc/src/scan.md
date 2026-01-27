# scan

Scanning is the action of adding dependencies by scanning a source file. A typical example is, when you have C source code, scan the .c file
to find the `#include` directives and add these files as dependencies. Same with C++, and, for my project, latex files, tikz files, lilypond files,...

Make has `makedepend`, omake has a scanner, here you write the scanner as rust code, as a trait implementation.

