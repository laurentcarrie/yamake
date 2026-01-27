# add edges

here we draw the edges between the nodes.

we explicit that an object file (.o) is the result of the compilation of a source file (.c),
and that when linking together .o files you produce an executable.

Here, we specify the graph, which will allow yamake to process the nodes in the right order.

What is actually performed to build the nodes (linking, compiling,...) is in the implementation of `Ofile` and `Xfile`, that implements `yamake::model::GNode` trait.

```rust
{{#include ../../../examples/project_C/main.rs:add_edges}}
```

the scan will add new edges.