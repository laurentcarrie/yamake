# example

We use the regular :blush: make tool from the gnu suite. This is fine for top level make.

run `make help`

## make build
this builds a demo rust program, using yamake. This demo is a build tool for a project, written in C, that is in the C_demo_project.

## make run
runs the demo program. This will build our C project. Sources are taken from `C_demo_project`, files are generated and build in `sandbox`. It then runs the program written in C.

## try a few things...
print the build result, which is a json report file.

---

### first run, the sandbox is empty
```sh
git checkout HEAD -- a_demo_project_in_C ; rm -rf sandbox
make run show-report
```
You will see that all files have status `ReBuilt`. The source files were mounted (copied from the srcdir to the sandbox)

---

### delete one of the built targets

```sh
# clean up
git checkout HEAD -- a_demo_project_in_C ; rm -rf sandbox ; make run
# delete one built target and rerun
rm sandbox/project_1/add.o
make run show-report
```

you will see that :
    - `project_1/add.o` has status `Rebuilt`. Because it was deleted, we had to re-compile it.
    - `project_1/demo` has status `RebuiltButUnchanged`, because all its deps have the same digest. This saves computation time. If a node has all its predecessors with status `RebuiltButUnchanged`, then this node is not rebuilt : changes that have no effect are not propagated

---

### make a change in the sources that has no effect

```sh
# clean up
git checkout HEAD -- a_demo_project_in_C ; rm -rf sandbox make run
# make a change and see how the build behaves
echo "// useless comment" >>  a_demo_project_in_C/project_1/add.c
make run show-report
```

you will see that :
    - `project_1/add.c` has status `Rebuilt`. The source file changed
    - `project_1/add.o` has status `RebuiltButUnchanged`, because we only added a comment
    - `project_1/demo` has status `NotRebuilt`, because all its deps have the same digest as before the build, it does not need to be rebuilt.

---

### now let's make an error in our C sources
```sh
# clean up
git checkout HEAD -- a_demo_project_in_C ; rm -rf sandbox make run
# make a coding error
echo "blah blah" >> a_demo_project_in_C/project_1/add.c ;
make run show-report -k
```
you will see that :
    - `project_1/add.c` has status `Rebuilt`, it means it was mounted again.
    - `project_1/add.o` has status `Failed`, because of course the compilation failed
    - `project_1/demo` has status `AncestorFailed`, it could not be built because one of its ancestors could not be built
- run `make show-errors`,
you will see the compiler error.
