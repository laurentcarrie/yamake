<!-- toc -->

---

# example

Our example is a C project that we want to compile. You will find the sources of the project in [sources of th C project](https://github.com/laurentcarrie/yamake/tree/main/demo_projects).

To build this project, instead of writing a Makefile, we write our own tool, using the yamake crate. You will find the sources of this tool in [sources of the demo tool](https://github.com/laurentcarrie/yamake/blob/main/yamake/examples/c_project_demo/main.rs)

---

you also have an example on a LaTeX project

---

to build and run the demo, we use a regular Makefile, :blush: nothing wrong with that

run `make help`

# real life

for the demo, the source files are explicitely listed in the source code. To make it exensible, you might
- scan the source directory to automatically find the .c and .h files, and have some heuristic for building libs and exes.
- or have some `description.yml` file of your project, where you list the sources, the libraries, exes, ... à la cmake

# make build
this builds a our demo rust program, using yamake. This demo is a build tool for our C project. When you run it, you build the program written in C.

# make run
runs the demo program. This will build our C project. Sources are taken from `C_demo_project`, files are generated and build in `sandbox`. It then runs the program written in C.

# try a few things...

---

### first run, the sandbox is empty
<details close>
<summary><i>make run_1</i></summary>

```sh
{{#include ../../Makefile:run_1}}
```
</details>

You will see that all files have status `ReBuilt`. The source files were mounted (copied from the srcdir to the sandbox)
<details close>
<summary><i>make-report.json</i></summary>

```sh
{{#include make-reports/run_1.json}}
```
</details>

you will see that :
- source files have status `Mounted` : they are copied from srcdir to the sandbox
- other files have status `Built`


---

### delete one of the built targets : `add.o`, and rebuild
<details close>
<summary><i>make run_2</i></summary>

```sh
{{#include ../../Makefile:run_2}}
```
</details>

<details close>
<summary><i>make-report.json</i></summary>

```sh
{{#include make-reports/run_2.json}}
```
</details>

you will see that :
    - `project_1/add.o` has status `Rebuilt`. Because it was deleted, we had to re-compile it.
    - `project_1/demo` has status `RebuiltButUnchanged`, because all its deps have the same digest. This saves computation time. If a node has all its predecessors with status `RebuiltButUnchanged`, then this node is not rebuilt : changes that have no effect are not propagated

---

### make a change in the sources that has no effect

<details close>
<summary><i>make run_3</i></summary>

```sh
{{#include ../../Makefile:run_3}}
```
</details>

<details close>
<summary><i>make-report.json</i></summary>

```sh
{{#include make-reports/run_3.json}}
```
</details>



you will see that :
    - `project_1/add.c` has status `Mounted`. The source file changed
    - `project_1/main.c` has status `NotReMounted`. The source file has not changed
    - `project_1/add.o` has status `RebuiltButUnchanged`, because we only added a comment
    - `project_1/demo` has status `NotRebuilt`, because all its deps have the same digest as before the build, it does not need to be rebuilt.

---

### now let's make an error in our C sources
<details close>
<summary><i>make run_4</i></summary>

```sh
{{#include ../../Makefile:run_4}}
```
</details>

<details close>
<summary><i>make-report.json</i></summary>

```sh
{{#include make-reports/run_4.json}}
```
</details>

<details close>
<summary><i>the error log</i></summary>

```sh
{{#include make-reports/error.txt}}
```
</details>

you will see that :
    - `project_1/add.c` has status `Rebuilt`, it means it was mounted again.
    - `project_1/add.o` has status `Failed`, because of course the compilation failed
    - `project_1/demo` has status `AncestorFailed`, it could not be built because one of its ancestors could not be built
- run `make show-errors`,
you will see the compiler error.
