# What is it ?

this is yet another makefile tool. This is motivated by the different situation : imagine you have a rust project, that takes inputs,
for instance json files, chunks of LaTeX files, lilypond snippets, and this project generates other latex files, then runs lilypond, latex,
fluidsynth, to get get outputs ( pdf songbooks with parts, and wav files).

One solution is to have the project generate a Makefile, run make, and scan the output directory to check generated artefacts.
Because you know your build graph, make isn't such a big help as you can walk the graph yourself.

This tools allows you to run make as a rust function.

We add some features :

- the build takes place in a sandbox, where the sources are copied first. This leaves the source tree clean
- when make runs, digest of each node are written to disk. Subsequent runs will avoid build nodes that don't need to
- the scanner is provided by the user, using the library
- there is no cyclic dependency between directories as we have with make
- the the result of the build is written to a json file
- for each node, sdout and stderr of the build is dumped to files in the sandbox
