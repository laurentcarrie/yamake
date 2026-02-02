# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

### Added
- **BuildNotChanged status**: New status for nodes that are rebuilt but produce identical output
  - Distinguishes between `BuildSuccess` (output changed) and `BuildNotChanged` (output unchanged)
  - Downstream nodes see `BuildNotChanged` as "unchanged" for incremental build decisions
- **expand_single_node() method**: Helper to call expand on individual nodes during build
- **Concurrent builds**: Node builds now run in parallel using Rayon
  - Nodes at the same dependency level are built concurrently
  - Respects dependency graph - nodes only build after all predecessors are ready
- **Status summary per iteration**: `print_status()` called at end of each build loop iteration
- **Status count assertion**: Verifies all nodes have a status at end of each iteration

### Changed
- **Replaced walk module with make module**: Complete rewrite of build orchestration
- **Build loop order**: Changed from mount→scan→build→expand to mount→expand→scan→build
  - Expand runs before scan so dynamically generated nodes exist when scanning for dependencies
  - Scan runs before build so discovered dependencies are available for build decisions
- **Moved mount_root_nodes to mount module**: Better code organization
- **Graph digest includes node statuses**: Digest now tracks both file contents and node statuses
  - Ensures loop continues when statuses change even if file contents don't
- **build_nodes() refactored for concurrency**: Split into three phases:
  1. Sequential categorization (AncestorFailed, BuildNotRequired, ready to build)
  2. Parallel builds using `rayon::par_iter()`
  3. Sequential status updates from build results

### Fixed
- **Expand timing with dynamic edges**: Nodes that receive edges from expand are now properly rebuilt
  - BuildFailed nodes are reset to Initial when expand adds new predecessor edges
- **Incremental builds with generated headers**: Scan-discovered dependencies from changed sources trigger rebuilds
  - Nodes reset to Initial when scan adds edges from new/changed dependencies
- **AncestorFailed propagation**: Nodes with Initial predecessors are skipped instead of marked AncestorFailed
  - Prevents premature failure marking before all predecessors are processed

### Tests
- Updated `test_incremental_build_comment` and `test_incremental_build_deleted_output` to expect `BuildNotChanged`
- All project_expand tests now pass with correct incremental rebuild behavior

## [0.1.8] - 2026-01-29

### Added
- **Colored status logs**: Build status now displayed with colors for better visibility
  - `MF` (MountedFailed): red
  - `BF` (BuildFailed): red
  - `BS` (BuildSuccess): green
  - `AF` (AncestorFailed): orange
- **Build reason logging**: Logs the reason why a node is being built before building starts

### Changed
- **expand() return type**: The `expand` method now returns `Result<(), String>` instead of `()`
  - Allows expand implementations to report errors
  - Build fails gracefully when expand returns an error

## [0.1.7] - 2026-01-28

### Added
- **GRootNode expand support**: The `expand` method is now called on root nodes after mounting
  - Previously, `expand` was only called on build nodes after they were built
  - Root nodes can now dynamically generate additional nodes and edges
- **expand_root_nodes() method**: New internal method in walk.rs to handle root node expansion
- **GRootNode documentation**: Added comprehensive doc examples for the `GRootNode` trait
  - Basic usage example showing a simple source file node
  - Advanced example demonstrating `expand` to generate nodes dynamically
- **Example documentation**: Added module-level docs to examples/project_C and examples/project_expand
  - Project structure diagrams
  - Build graph visualizations
  - Usage instructions
- **Examples in cargo doc**: Configured Cargo.toml to include examples in documentation output

### Changed
- **mount_root_nodes() return type**: Now returns `(bool, Vec<NodeIndex>)` to track which roots were mounted

### Tests
- `test_grootnode_expand_called_during_make`: Verifies that `expand` is called on `GRootNode` during `make()`

## [0.1.6] - 2026-01-27

### Added
- **ScanIncomplete node status**: New status for nodes waiting on dependencies to be generated
  - Nodes are marked `ScanIncomplete` when scanned files don't exist or haven't been generated yet
  - Build waits for expand operations to complete before proceeding
- **Orphan sandbox file detection**: Detects files from previous expand runs that may need updating
  - Prevents building with stale generated headers during incremental builds
- **scan() return type change**: Now returns `(bool, Vec<PathBuf>)` tuple
  - First element indicates if scan is complete (all files found)
  - Second element is the list of discovered dependencies
- **Expanded nodes added to built set**: Files created by expand are immediately available as dependencies

### Changed
- **Build ordering with expand**: Nodes depending on generated files wait for expand to run first
- **Include path scanning**: Scan now checks include_paths for headers before marking incomplete

### Fixed
- **Incremental builds with expand**: Fixed issue where nodes would build with stale generated headers
- **test_project_expand_incremental**: Adding new languages now properly triggers rebuild
- **Clippy warnings**: Fixed uninlined format args in command.rs and model.rs

### Documentation
- Added new documentation pages: mount, scan, expand, build-logs, timestamp, capture-error, generate-makefile
- Added recursive Makefile example and comparison
- Reorganized SUMMARY.md with improved structure
- Added examples for project_C and project_expand
- Fixed expand.md to reference correct demo project path

## [0.1.5]

### Added
- **Incremental build support**: Build system now tracks file digests and node statuses to avoid unnecessary rebuilds
- **OutputInfo structure**: New data structure containing pathbuf, status, and digest for each node
- **New node statuses**:
  - `MountedChanged`: Source file was mounted and its digest changed since last build
  - `MountedNotChanged`: Source file was mounted but digest is unchanged
  - `BuildNotRequired`: Node was skipped because all predecessors are unchanged and output digest matches
  - `BuildSuccess`: Node was built successfully with changed output
  - `AncestorFailed`: Node was skipped because a predecessor failed
- **make-report.yml format**: Now stores full build information including:
  - `pathbuf`: File path relative to sandbox
  - `status`: Final node status after build
  - `digest`: SHA256 hash of file contents (or null if file doesn't exist)
  - `absolute_path`: Absolute path to the output file
  - `stdout_path`: Absolute path to stdout log file (null for source files)
  - `stderr_path`: Absolute path to stderr log file (null for source files)
  - `predecessors`: List of predecessor nodes with their pathbuf and status
- **Build logging**: stdout and stderr captured to `<sandbox>/logs/<node>.stdout` and `<sandbox>/logs/<node>.stderr`
  - Command echoed as first line in both log files
- **OFile compile_flags**: `OFile::new()` now accepts `include_paths` and `compile_flags` parameters
- **root_predecessors() function**: Returns all root nodes in a node's predecessor tree

### Changed
- Renamed `Build` status to `BuildSuccess` for clarity
- Renamed `BuildSkipped` to `BuildNotRequired` to better reflect meaning
- Build loop now checks predecessor statuses before building:
  - If any predecessor is `BuildFailed` or `AncestorFailed`, node is marked `AncestorFailed`
  - If all predecessors are unchanged, checks output digest before building
- `make()` function resets all node statuses to `Initial` at start, allowing multiple calls on same graph

### Tests
- `test_incremental_build_unchanged`: Verifies second build marks root nodes as `MountedNotChanged` and built nodes as `BuildNotRequired`
- `test_incremental_build_with_failure`: Verifies build failure propagation with `BuildFailed` and `AncestorFailed` statuses
- `test_incremental_build_after_delete`: Verifies rebuild after deleting output file results in `BuildNotRequired` (digest unchanged)
