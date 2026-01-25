<!-- toc -->

# Node life cycle
[top](#top)

# node target
[top](#top)

The target of a node is a file in the sandbox. `node <=> target` is a 1-1 relation.

---

# node statuses
[top](#top)

Each node has a status that tracks its state during the build process:

| Status | Description |
|--------|-------------|
| `Initial` | Node has not been processed yet |
| `MountedChanged` | Source file mounted, digest changed since last build |
| `MountedNotChanged` | Source file mounted, digest unchanged |
| `MountedFailed` | Failed to mount source file |
| `Running` | Node is currently being built |
| `BuildSuccess` | Build completed successfully with changed output |
| `BuildNotRequired` | Build skipped (predecessors unchanged and output digest matches) |
| `BuildFailed` | Build failed |
| `AncestorFailed` | Skipped because a predecessor failed |

---

# node type
[top](#top)

<!-- mermaid version

```mermaid
info
```
-->


```mermaid

---
title: node life cycle
---

flowchart

start@{ shape: f-circ}

start --> decision_has_preds

decision_has_preds{node
has preds ?}:::choice

decision_file_exists_in_sources{file
in sources ?}:::choice

decision_digest_changed{
    digest
    changed ?
}:::choice

decision_all_preds_ok{
    all preds
    ok ?
}:::choice

decision_some_preds_changed{
    some preds
    changed ?
}:::choice

decision_build_success{
    build
    success ?
}:::choice

decision_output_digest{
    output
    digest
    changed ?
}:::choice


missing_source[MountedFailed]

decision_has_preds -- no --> decision_file_exists_in_sources
decision_file_exists_in_sources -- yes --> mount:::action
decision_file_exists_in_sources -- no --> missing_source:::ko
decision_has_preds -- yes --> decision_all_preds_ok

mount -->  decision_digest_changed

decision_digest_changed -- yes --> mounted_changed[MountedChanged]:::changed
decision_digest_changed -- no --> mounted_not_changed[MountedNotChanged]:::unchanged


%%% build branch
ancestor_failed[AncestorFailed]

decision_all_preds_ok -- no -----> ancestor_failed:::ko


decision_all_preds_ok -- yes --> decision_some_preds_changed

build:::action

decision_some_preds_changed -- yes --> build
decision_some_preds_changed -- no --> decision_output_digest

decision_output_digest -- yes --> build
decision_output_digest -- no --> build_not_required[BuildNotRequired]:::unchanged

build --> decision_build_success
decision_build_success -- yes --> decision_final_digest{output
digest
changed ?}:::choice
decision_build_success -- no --> build_failed[BuildFailed]:::ko

decision_final_digest -- yes --> build_success[BuildSuccess]:::changed
decision_final_digest -- no --> build_not_required2[BuildNotRequired]:::unchanged


classDef ko fill:#f00,color:white,font-weight:bold,stroke-width:2px,stroke:yellow
classDef changed fill:#0ff,color:black,font-weight:bold,stroke-width:2px,stroke:yellow
classDef unchanged fill:#0f0,color:black,font-weight:bold,stroke-width:2px,stroke:yellow

classDef action fill:#FF8C00,color:black,font-weight:bold,stroke-width:2px,stroke:black,shape:bolt
classDef choice fill:lavender,color:black,font-weight:bold,stroke-width:2px,stroke:red,shape: circle


```

---

# digest

Digests (SHA256 hashes) of nodes are stored in `make-output.yml` in the sandbox. The file contains an array of `OutputInfo` entries:

```yaml
- pathbuf: project_1/main.c
  status: MountedNotChanged
  digest: 5ebac2a26d27840f79382655e1956b0fc639cbdca5643abaf746f6e557ad39b8
- pathbuf: project_1/main.o
  status: BuildNotRequired
  digest: ec1a9daf9c963db29ba4557660e3967a6eeb38dab5372e459d3a1be446c38417
```

On subsequent builds, digests are compared to determine if files have changed:
- **Source files**: Compared before mounting to set `MountedChanged` or `MountedNotChanged`
- **Built files**: Compared after build to set `BuildSuccess` or `BuildNotRequired`

---

# source node
[top](#top)

A source node has no predecessor. It is a file in the source directory, and there is no rule to build it. It is mounted (copied from source directory to sandbox). When mounted, its digest is compared to the previous digest stored in `make-output.yml`.

---

# incremental builds

The build system supports incremental builds by tracking:
1. **Source file digests**: Detect when source files change
2. **Output file digests**: Avoid rebuilding when output would be identical
3. **Predecessor statuses**: Skip builds when all predecessors are unchanged

When all predecessors have status `MountedNotChanged` or `BuildNotRequired`, the output file is checked:
- If it exists and its digest matches the previous build, status is set to `BuildNotRequired`
- Otherwise, the build runs and the result is compared to set `BuildSuccess` or `BuildNotRequired`

---
