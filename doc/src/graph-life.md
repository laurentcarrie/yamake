<!-- toc -->

# Graph life 
[top](#top)

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
title: graph life cycle
---

flowchart

start@{ shape: f-circ}

start --> reset[Reset statuses
to Initial]:::action

reset --> load_digests[Load previous
make-output.yml]:::action

load_digests --> mount[Mount root nodes]:::action

mount --> decision_mount{mount
success ?}:::choice

decision_mount -- no --> mount_failure[MountedFailed]:::ko

decision_mount -- yes --> scan[Scan for
dependencies]:::action

scan --> decision_new_roots{new roots
discovered ?}:::choice

decision_new_roots -- yes --> mount

decision_new_roots -- no --> build_loop[Build nodes
in parallel]:::action

build_loop --> decision_build{all builds
success ?}:::choice

decision_build -- yes --> save_output[Save
make-output.yml]:::action
decision_build -- no --> save_output

save_output --> decision_final{any
failures ?}:::choice

decision_final -- no --> success[Success]:::ok
decision_final -- yes --> failure[Failure]:::ko


classDef ko fill:#f00,color:white,font-weight:bold,stroke-width:2px,stroke:yellow
classDef changed fill:#0ff,color:black,font-weight:bold,stroke-width:2px,stroke:yellow
classDef unchanged fill:#0f0,color:black,font-weight:bold,stroke-width:2px,stroke:yellow

classDef ok fill:#0f0,color:black,font-weight:bold,stroke-width:2px,stroke:yellow

classDef action fill:#FF8C00,color:black,font-weight:bold,stroke-width:2px,stroke:black,shape:bolt
classDef choice fill:lavender,color:black,font-weight:bold,stroke-width:2px,stroke:red,shape: circle


```

---

