# add nodes

here we assume that we already have structs that implements M::Node, in our example we have 3 implementations :
- CFile for .c files
- HFile for .h files
- OFile for object files
- XFile for executable files

see section ...

in our example, for the demo add the nodes manually.
```rust
{{#include ../../../examples/project_C/main.rs:add_nodes}}
```

---

## in a real project

in a real project, you might want to :
- scan the srcdir to find the .c and .h files (what you do manually in a Makefile )
- for each .c, create a .o node ( the implicit rule of a Makefile )
- as in an real Makefile, you would need to explicit which objects and libraries you need to build, with their sources

---

## use the sandbox

here for instance, use the sandbox as the search path for the header ( the -I ) option of compilation. Always refer to paths in the sandbox and not
in the srcdir, because when using built artefacts it will fail when using the srcdir

> [!CAUTION]

everything, except the mount, happens in the sandbox. All paths are relative, and understood from the sandbox.

---

## paths are unique

> [!CAUTION]

paths are unique : two nodes cannot have the same path, that would yield an error
