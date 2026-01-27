# scan

The scan is performed automatically during `make()`. It adds edges to the graph based on `#include` directives found in source files.

After the scan, we will have new edges:
- main.o depends on add.h and wrapper.h (via wrapper.h)
- add.o depends on add.h


![graph plot after scan](./before-scan.png)

---

The edges that were added by the scan are labeled `Scanned`.

---

## Scan completion

The `scan()` method returns a tuple `(bool, Vec<PathBuf>)`:
- **First element**: `true` if scan is complete (all included files found), `false` otherwise
- **Second element**: List of discovered dependencies (header paths from `#include` directives)

When a scan is incomplete (missing files), the node is marked `ScanIncomplete` if there are other unbuilt nodes that could generate the missing files via `expand()`.

---

## Interaction with expand

When using `expand()` to generate code dynamically:

1. A node scans its source files and finds includes that don't exist yet
2. The node is marked `ScanIncomplete` and waits
3. Another node runs `expand()` and creates the missing files
4. On the next iteration, the scan succeeds and the node can build

This ensures nodes always build with up-to-date generated headers.

---

## Orphan file detection

If a scanned file exists in the sandbox but has no corresponding graph node, it was likely created by a previous `expand()` run. The build system waits for expand to run again in case the file needs updating.

This is critical for incremental builds where source changes should regenerate derived files.

---

*Scan is done in the sandbox: it only adds edges between existing nodes, it does not add nodes.*

The scan reads the source files and tries to find dependencies. So in our example, main.o depends on main.c, and therefore this file is scanned.
But if add.h was forgotten in the graph, this dependency will be ignored.

---

The scan is used to determine, using file digests, if a successor node needs to be rebuilt. So if the scanner is not correct, the build will be fine,
but the feature *rebuild only what is necessary* will not work correctly.
