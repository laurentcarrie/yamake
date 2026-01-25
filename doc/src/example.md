<!-- toc -->

---

# example

Our example is a C project that we want to compile. You will find the sources of the project in [sources of the C project](https://github.com/laurentcarrie/yamake/tree/main/demo_projects).

To build this project, instead of writing a Makefile, we write our own tool, using the yamake crate. You will find the sources of this tool in [sources of the demo tool](https://github.com/laurentcarrie/yamake/blob/main/examples/c_project.rs)

---

# running the example

```bash
cargo run --example c_project -- -s demo_projects -b sandbox
```

This will:
1. Copy source files from `demo_projects` to `sandbox`
2. Scan C files for `#include` directives and add dependency edges
3. Build all targets in parallel where possible

# node statuses

After a build, each node has one of these statuses:

- **Initial**: Not processed yet
- **Mounted**: Source file copied to sandbox
- **MountedFailed**: Failed to copy source
- **Running**: Build in progress
- **Build**: Successfully built
- **BuildFailed**: Build failed
- **AncestorFailed**: Skipped because a dependency failed

# error handling

When a build fails:
- The failing node gets `BuildFailed` status
- All dependent nodes get `AncestorFailed` status
- `make()` returns `false`
