<!-- toc -->

# report
[top](#top)

The `make()` method returns a boolean indicating success or failure. You can also inspect the `nodes_status` map to see the status of each node after the build.

## the status types
[top](#top)

```rust
{{#include ../../../src/model.rs:buildtype}}
```

Status meanings:
- **Initial**: Node has not been processed yet
- **Mounted**: Root node source file has been copied to sandbox
- **MountedFailed**: Failed to copy source file to sandbox
- **Running**: Node build is currently in progress
- **Build**: Node was built successfully
- **BuildFailed**: Node build failed
- **AncestorFailed**: A predecessor node failed, so this node was skipped
