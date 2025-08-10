# make

running the make command :
- call the mount
- call the scan
- traverse the graph and build the output artefacts.

make will

```rust
{{#include ../../yamake/examples/c_program_example.rs:make}}
```

as a return of the make command, and also stored in the sandbox, you get the result of this action :

```json
{{#include make_result.json}}
```
