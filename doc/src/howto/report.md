<!-- toc -->

# report
[top](#top)

after running make, the file `make-report.json` is written in the sandbox, here are the first lines printed

```json
{{#include ../../../sandbox/make-report.json:1:20}}
```

you can add this in your makefile ( :blush: yes I use make for the top level tasks )

## add to your Makefile
[top](#top)

```makefile
{{#include ../../../Makefile:json-errors}}
```

> [!TIP]
note that sandbox paths are replaced with srcdir paths. This way, in your editor, for instance vscode, you clicking on the error will open the source file, and not the copy in the sandbox.

## the status types
[top](#top)

```rust
{{#include ../../../yamake/src/model.rs:buildtype}}
```
