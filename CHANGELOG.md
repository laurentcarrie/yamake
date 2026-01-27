# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

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
