<!-- toc -->

# Node life cycle
[top](#top)

# node target
[top](#top)

The target of a node is a file in the sandbox. `node <=> target` is a 1-1 relation.

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

decision_digest{
    compute 
    digest
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

decision_expand{
}:::choice


missing_source[missing
source]

decision_has_preds -- no --> decision_file_exists_in_sources
decision_file_exists_in_sources -- yes --> mount:::action
decision_file_exists_in_sources -- no --> missing_source:::ko
decision_has_preds -- yes --> decision_all_preds_ok

mount -->  decision_digest

decision_digest -- changed --> expand:::action
decision_digest -- not changed --> notchanged:::unchanged

notchanged[un
changed]


%%% build branch
build_skipped[build
Ancestor Failed]

decision_all_preds_ok -- no -----> build_skipped:::ko


decision_all_preds_ok -- yes --> decision_some_preds_changed

build:::action

decision_some_preds_changed -- yes --> build
decision_some_preds_changed -- no --> notchanged:::unchanged

build --> decision_build_success
decision_build_success -- yes --> decision_digest
decision_build_success -- no --> build_failed:::ko

build_failed[Build
Failed]


expand --> decision_expand

decision_expand -- failure --> expand_failure
expand_failure[
    expand
    failure
]:::ko

decision_expand -- graph changed --> graph_changed:::ko
graph_changed[
    graph
    changed
]

%% decision_expand -- graph unchanged --> notchanged
decision_expand -- graph unchanged --> changed:::changed


classDef ko fill:#f00,color:white,font-weight:bold,stroke-width:2px,stroke:yellow
classDef changed fill:#0ff,color:black,font-weight:bold,stroke-width:2px,stroke:yellow
classDef unchanged fill:#0f0,color:black,font-weight:bold,stroke-width:2px,stroke:yellow

classDef action fill:#FF8C00,color:black,font-weight:bold,stroke-width:2px,stroke:black,shape:bolt
classDef choice fill:lavender,color:black,font-weight:bold,stroke-width:2px,stroke:red,shape: circle


```

---

# digest

digest of nodes are stored in a cache file. The first time the tool is ran, the digest is missing. If not, the action of computing the digest allows
to node to be marked as changed or unchanged since the last run.

---

# source node
[top](#top)

a source node has no predecessor. It is a file in the source directory, and there is no rule to build it, it is mount, ie copied from source directory to sandbox. When mounted, its digest is compared to the old one.

---

# expand

Expanding a node means adding nodes and edges to the build tree.


