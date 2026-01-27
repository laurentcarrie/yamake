# recursive makefile condidered harmful
[top](#top)

this is a well known problem, you can follow this link :
[recursive makefile considered harmful](https://accu.org/journals/overload/14/71/miller_2004/)

this a quote of the article :

> [!CAUTION]
    - It is very hard to get the order of the recursion into the sub­directories correct. This order is very unstable and frequently needs to be manually ‘‘tweaked.’’ Increasing the number of directories, or increasing the depth in the directory tree, cause this order to be increasingly unstable.
    - It is often necessary to do more than one pass over the sub­directories to build the whole system. This, naturally, leads to extended build times.
    - Because the builds take so long, some dependency information is omitted, otherwise development builds take unreasonable lengths of time, and the developers are unproductive. This usually leads to things not being updated when they need to be, requiring frequent “clean” builds from scratch, to ensure everything has actually been built.
    - Because inter-directory dependencies are either omitted or too hard to express, the Makefiles are often written to build too much to ensure that nothing is left out.
    - The inaccuracy of the dependencies, or the simple lack of dependencies, can result in a product which is incapable of building cleanly, requiring the build process to be carefully watched by a human.
    - Related to the above, some projects are incapable of taking advantage of various “parallel make” impementations, because the build does patently silly things.


If you worked on big projects, you know what this is, especially the inter-directory dependencies.

<!-- Consider this example :

```sh
root-directory
    +-- dir1
        +--- a1.c
    +-- dir2
        +--- a2.c
    +-- dir3 
``` -->

Yamake solves this issues by not scanning directories, artefacts form a DAG regardless of where files stay.
