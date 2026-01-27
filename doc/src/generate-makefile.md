# Makefile

yamake is a library, that lets you, after you have implemented your own rules ( by implementing the GNode trait ), create an executable that will build your project.

make uses its own syntax, for you to write a Makefile. Here you need to construct the graph yourself, either :
- in the main function, like in the documentation examples
- you may write your own yml file, that describes your project, and you build your graph from it
- you could also scan your source tree, and construct nodes as your discover files
