# make
[top](#top)

running the make command :
- call the mount
- call the scan
- traverse the graph and build the output artefacts.

make will

```rust
{{#include ../../../yamake/examples/C_demo_project.rs:make}}
```

as a return of the make command, and also stored in the sandbox, you get the result of this action in the file `<sandbox>/make-report.json`
