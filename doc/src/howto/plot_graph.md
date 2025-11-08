# plot graph

now that the graph is built, we use petgraph utility to plot the graph

```rust
{{#include ../../../yamake/examples/C_demo_project.rs:dot}}
```

run `dot -Tpng -o out.png out.dot` to get the image :

![graph plot](./before-scan.png)

we notice that the graph is not connected, the scanner will add edges. Notice that all the edges are labeled `Direct`, which means a direct
dependency that was explicited. Direct also means that you need it, it is part of the rule that will build the output artefact.
