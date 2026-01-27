toc

<!-- toc -->

---

# What is it ?
[top](#top)

this is yet another makefile tool.


Starting a new project, that involves yml descriptions, LaTeX, lilypond and tikz code generation, unknown list of files, usage of build tools 
such as lualatex or other custom tools (strudel_of_lilypond), ... make was the logical tool to use to build the outputs,
but it turned to be more a problem than a solution.

Having a long experience with make ( and all replacements, such as cmake, omake, and all kinds of proprietary solutions ),
some issues are recurrent, and this tool intends to solve them.

Yamake tries to solve this issues :
- [mount](./mount.md) allows you to build in a separate sandbox, and leave your source directory clean
- [impossible recursion, and the famous article recursive Makefile considered harmful](./recursive-makefile-considered-harmful.md)
- [how to deal with code generators ?](./expand.md). How do you write a Makefile if you don't know which intermediate or final artefacts to build ?
- [how to break unneeded build chain ?](./digest.md). In the chain `a -> b -> c`, if after modifying `a` and rebuilding `b`, b is the same, why rebuild `c` ?
- [how to manage non explicit dependencies, aka scan ?](./scan.md).
- [how to check make capture error ?](./capture-error.md)
- [how to generate a Makefile ( instead of writing it ) ?](./generate-makefile.md)
- [how to nicely get logs of artefact builds](./build-logs.md) ( and not having a cluttered stdout with hundred of lines )
- [how to get a build report](./build-report.md)





---




## typo error
[top](#top)

Consider this Makefile :

    x.o : x.c
        gcc -o blah.o $<

if file x.c is correct, running make will not yield an error, though the target x.o will not be built. For manually captured big makefiles, this is a real issue that produces bugs


## logs
[top](#top)

When you build a huge projects, you will have only two logs, the stdout and stderr, and can be thousands of line log. On top of that, if the build is parallelized,
log lines are untangled and just impossible to read.

