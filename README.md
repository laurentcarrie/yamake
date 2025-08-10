# yamake
yet another makefile tool, written in rust.

main features :
- a node is a trait, that can implement the build and the scan function
- you describe the graph yourself, adding nodes and edges
- run in a sandbox : your source directories are not polluted by the build.
- when running make, digest of targets are computed and written to disk. on following make invocations, only the targets that need to be rebuilt will be computed
