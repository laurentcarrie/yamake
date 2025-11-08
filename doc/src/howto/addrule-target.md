# target ( mandatory )

```rust
{{#include ../../yamake/src/model.rs:target}}
```

the target is the path of a node, relative to the sandbox directory.

If the node is a source, this path must exist in the srcdir and will be copied to the sandbox during mount.

if two nodes of the graph have the same path, an error will be returned

---

## example

we need to store the target path in the struct

```rust
{{#include ../../yamake/src/c_project/o_file.rs:structofile}}
```

```rust
{{#include ../../yamake/src/c_project/o_file.rs:target}}
```
