# make
[top](#top)

Running the make command:
- Resets all node statuses to `Initial`
- Mounts root nodes (copies source files to sandbox)
- Compares source file digests to detect changes
- Scans for dependencies (adds edges for `#include` directives)
- Traverses the graph and builds output artifacts in parallel where possible
- Saves build results to `make-output.yml`

```rust
{{#include ../../../examples/c_project.rs:make}}
```

## Build output

After the build, `<sandbox>/make-output.yml` contains detailed information for each node:

```yaml
- pathbuf: project_1/main.c
  status: MountedNotChanged
  digest: 5ebac2a26d27840f79382655e1956b0fc639cbdca5643abaf746f6e557ad39b8
  absolute_path: /path/to/sandbox/project_1/main.c
  stdout_path: null
  stderr_path: null
  predecessors: []
- pathbuf: project_1/main.o
  status: BuildNotRequired
  digest: ec1a9daf9c963db29ba4557660e3967a6eeb38dab5372e459d3a1be446c38417
  absolute_path: /path/to/sandbox/project_1/main.o
  stdout_path: /path/to/sandbox/logs/project_1/main.o.stdout
  stderr_path: /path/to/sandbox/logs/project_1/main.o.stderr
  predecessors:
  - pathbuf: project_1/main.c
    status: MountedNotChanged
  - pathbuf: project_1/wrapper.h
    status: MountedNotChanged
```

## Build logs

Build commands capture stdout and stderr to log files in `<sandbox>/logs/`:

- `<sandbox>/logs/<node>.stdout` - standard output
- `<sandbox>/logs/<node>.stderr` - standard error

The first line of each log file contains the command that was executed:

```
"gcc" "-c" "-I" "sandbox" "-o" "sandbox/project_1/main.o" "sandbox/project_1/main.c"
```

## Incremental builds

The `make()` function can be called multiple times on the same graph. On subsequent runs:
- Source files with unchanged digests get status `MountedNotChanged`
- Built files that don't need rebuilding get status `BuildNotRequired`
- Only files with changed inputs or missing outputs are rebuilt
