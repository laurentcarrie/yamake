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

decision_source_exists{
    source
    exists ?
}:::choice

decision_has_preds -- no --> decision_source_exists

decision_source_exists -- yes --> mount
decision_source_exists -- no --> error_no_source
mount[
    mount
    file
]:::action


error_no_source[
    No
    Source
]:::ko

mount --> expand:::action

decision_all_preds_exist{all preds
exist ?}:::choice

decision_has_preds -- yes --> decision_all_preds_exist


decision_all_preds_exist -- yes --> build:::action

decision_all_preds_exist -- no --> error_no_source[
    missing
    file
]:::ko

build --> expand

expand --> final1@{shape: framed-circle}
error_no_source --> final2@{shape: framed-circle}


classDef ko fill:#f00,color:white,font-weight:bold,stroke-width:2px,stroke:yellow
classDef ok fill:#0f0,color:black,font-weight:bold,stroke-width:2px,stroke:yellow

classDef action fill:#dbd7d2,color:black,font-weight:bold,stroke-width:2px,stroke:black,shape: lin-rect
classDef choice fill:lavender,color:black,font-weight:bold,stroke-width:2px,stroke:red,shape: circle


```


---

## other


```mermaid

---
title: node lifecycle
---

stateDiagram-v2

    state has_preds <<choice>>
    state source_exists <<choice>>
    state all_deps_present <<choice>>
    state build_success <<choice>>

    EvalDigest: digest
    Error:::ko : Error
    %%Error2:::ko : Error
    %%Error3:::ko : Error

    [*] --> has_preds
    has_preds --> all_deps_present: has predecessors
    has_preds --> source_exists : no predecessors
    source_exists --> mount : source file exists
    source_exists --> Error : no source file
    %%Error --> [*]
    mount --> EvalDigest
    %%EvalDigest --> [*]

    all_deps_present --> Build : all deps present
    all_deps_present --> Error : some deps are missing 


    Build --> build_success
    build_success --> EvalDigest : build success
    build_success --> Error : build failure
    build_success --> BuildSkipped : skipped

    EvalDigest --> done:::ok
    BuildSkipped --> done:::ok

    %%Error --> [*]
    %%Error2 --> [*]
    %%Error3 --> [*]

    done --> [*]
    Error --> [*]


classDef ko fill:#f00,color:white,font-weight:bold,stroke-width:2px,stroke:yellow
classDef ok fill:#0f0,color:black,font-weight:bold,stroke-width:2px,stroke:yellow



```

---

```mermaid

---
title: Make
---
stateDiagram-v2
        MountedOk: mounted
        MountedKo: mounted
        BuiltOk: built
        BuiltKo: built
        %% Missing: not present in sources

        [*] --> Mount 
        [*] --> Built
        [*] --> generated : expansion
        BuiltOk:::ok --> BuiltKo
        BuiltKo --> BuiltOk
        %% MountedKo --> MountedOk:::ok

        [*] --> Digest
        [*] --> Scan

        NotModified:::ok not modified
        Modified:::ko modified

        NeedsRebuild:::ko : needs rebuild
        UpToDate:::ok : up to date

        state Digest {
            [*] --> NotModified
            [*] --> Modified
            NotModified --> Modified
            Modified --> NotModified
        }

        state Scan {
            [*] --> NeedsRebuild
            [*] --> UpToDate
        }
        




classDef ko fill:#f00,color:white,font-weight:bold,stroke-width:2px,stroke:yellow
classDef ok fill:#0f0,color:black,font-weight:bold,stroke-width:2px,stroke:yellow

%%class Mounted badBadEvent
%%class Expanded ok

```

