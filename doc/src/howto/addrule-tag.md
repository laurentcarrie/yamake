# tag ( mandatory )

```rust
{{#include ../../yamake/src/model.rs:tag}}
```

at build time, the build function will receive a collection of nodes, that are the predecessors. Not all nodes have the same role,
some can be C file, config files, ... so the tag will help sort that. See later the build example.

---

## example

the tag of an object file

```rust
{{#include ../../yamake/src/c_project/o_file.rs:tag}}
```
