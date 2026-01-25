<!-- toc -->
# How to add rules

to add rules, you need to define the nodes of your build graph. 

--- 
# root nodes

a root node implements the trait GRootNode

```rust
{{#include ../../src/model.rs:GRootNode}}
```

## tag 

the tag will be used later in your code, in successor nodes, when you want to know the nature of a predecessor

For example, an exe file is linked from object files and static libraries. When running the link, you need to know
if a file is a object or a library. Granted, you could do that with the file extension...


## example

```rust
{{#include ../../src/c_nodes/c_file.rs}}
```

---

# built nodes

the nodes which are not root have predecessors. They must implement trait `yamake::GNode`

```rust
{{#include ../../src/model.rs:GNode}}
```

## scan

scan function

