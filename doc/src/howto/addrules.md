# How to add rules

Adding a rule is implementing `yamake::model::GNode` trait.



# optional implementations are :

## scan

```rust
{{#include ../../src/model.rs:scan}}
```

the default implementation of a scanner returns an empty list. Having a wrong scanner will not prevent the build from run, but it will prevent it to run
the correct rules in case a source is modified.

## build

```rust
{{#include ../../src/model.rs:build}}
```

@todo : review this comment
the default implementation will return true, which means that the build was successful. This is fine for source nodes, as the target will exist
(it is mounted). For for built nodes, the target will not be built and the successor nodes built will fail.


## id

a string that uniquely represents a node, used as key in the maps. Default implementation is to use the target, you can change it at your own risk.

```rust
{{#include ../../src/model.rs:target}}
```
