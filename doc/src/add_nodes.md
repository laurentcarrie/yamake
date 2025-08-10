# add nodes

here we assume that we already have structs that implements M::Node, in our example we have 3 implementations :
- CFile for .c files
- HFile for .h files
- OFile for object files
- XFile for executable files

see section ...

in our example we will not create the source nodes manually, but do that automatically when walking through the source directory

```rust
{{#include ../../yamake/examples/c_program_example.rs:add_nodes}}
```

---

in a real project, you would :
- scan the srcdir to find the .c and .h files (what you do manually in a Makefile )
- for each .c, create a .o node ( the implicit rule of a Makefile )
- as in an real Makefile, you would need to explicit with objects and libraries you need to build the .o

---

*use the sandbox*

here for instance, use the sandbox as the search path for the header ( the -I ) option of compilation. Always refer to paths in the sandbox and not
in the srcdir, because when using built artefacts it will fail when using the srcdir

of course you can go around and use the srcdir for sources of some include path, but that would be a bad idea

---

*all paths are relative to the sandbox*

the sandbox value is passed as different argument, in case you need to forge an absolute path

---

*paths are unique* : two nodes cannot have the same path
