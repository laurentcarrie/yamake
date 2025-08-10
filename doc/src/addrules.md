# How to add rules

Adding a rule is implementing `yamake::model::GNode` trait.



# optional implementations are :

## scan

{{#include ../../yamake/src/model.rs:scan}}



## build

{{#include ../../yamake/src/model.rs:build}}

the default implementation will return true, which means that the build was successful. This is fine for source nodes, as the target will exist
(it is mounted). For for built nodes, the target will not be built and the successor nodes built will fail.


## id

a string that uniquely represents a node, used as key in the maps. Default implementation is to use the target, you can change it at your own risk.

{{#include ../../yamake/src/model.rs:target}}
