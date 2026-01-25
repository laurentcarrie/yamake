<!-- toc -->

# first

first we have a description of messages in different languages, in a yaml file. 

we have a code generator, written in rust. The code source is part of the project

This will generate C files, and we have our `main.c` that will print some messages
file `greetings.yml`

```yml
{{#include ../../demo_projects/demo_expand/greetings.yml}}
```

from the `greetings.yaml` file C files and H files will be generated, but for now we don't have this list so we cannot build the build graph,
so our graph looks like this but is incomplete


we have 
- `greetings.yml` and `main.c` are source files, and will be mounted
- `main.o` will be compiled from `main.c` ( we miss header files )
- executable file `demo` will be the product of linking `liblanguages.a` and `main.o`
- and .... `liblanguages.a` is a library that will be built from C files, generated from greetings.yml, we miss these files yet

the Green arrow shows the dependency, it will trigger the expansion.



```mermaid

---

---


flowchart

    greetings_yml([greetings.yml]):::source
    main_c([main.c]):::source
    main_o{{main.o}}:::ofile
    liblanguages_a{{liblanguages.a}}:::ofile
    demo{{demo}}:::ofile



    main_c ec1@--> main_o
    liblanguages_a ec3@--> demo
    main_o ec4@--> demo
    greetings_yml ex1@--> liblanguages_a


    %% classDef source fill:#f96
    classDef generated fill:#bbf,stroke:#f66,stroke-width:2px,color:#fff,stroke-dasharray: 5 5
    classDef ofile fill:#03f,color:#f66

    classDef e_htosource stroke:#aaa,stroke-width:0.7x ,stroke-dasharray: 10,5;
    class es1,es2,es3,es4,es5,es6,es7,es8,es9,es10 e_htosource;

    classDef e_generate stroke:#f00,stroke-width:1px;
    class eg1,eg2,eg3,eg4,eg5,eg6,eg7 e_generate;

    classDef e_compile stroke:#00f,stroke-width:1px;
    class ec1,ec2,ec3,ec4,ec5,ec6,ec7 e_compile;

    classDef e_expand stroke:#3f3,stroke-width:3px,color:red;
    class ex1,ex2,ex3,ex4,ex5,ex6,ex7 e_expand;

```

---

# one solution

one solution would be to call the code generator, get the list of generated files, and construct our build graph.
this would work here, but not in a more general situation, where the generated files will also be used to build other tools, that will also generate other files


---

# chosen solution

for each node, we have an `expand` trait. It usually does nothing, but, in our case, `expand` will :
- generate the files in the sandbox
- add these files to the graph
- add edges


---

# the final graph

we have the nodes

- in orange the sources
- in lavender with green dotted border, expanded nodes, and file is generated in sandbox
- in blue with green dotted border, expanded nodes, these are `.o` files, they will be built later
- in blue the compiled files 

and the edges :

- in red, the generation of code
- in green the expansion
- in blue, the compilation and link of C source code
- in dotted, scanning of `#include` transitive directives




```mermaid

---

animate-yml-file: expand.yml

---

flowchart

    greetings_yml([greetings.yml]):::source
    greetings_json([greetings.json]):::json
    main_c([main.c]):::source

    subgraph expanded nodes
        %%subgraph generated files
            languages_h{{languages.h}}:::generated
            english_h{{english.h}}:::generated
            german_h{{german.h}}:::generated
            french_h{{french.h}}:::generated

            english_c{{english.c}}:::generated
            german_c{{german.c}}:::generated
            french_c{{french.c}}:::generated

        %%end
        %%subgraph expanded files
            english_o{{english.o}}:::oexpanded
            german_o{{german.o}}:::oexpanded
            french_o{{french.o}}:::oexpanded
        %%end
    end

    main_o{{main.o}}:::ofile
    liblanguages_a{{liblanguages.a}}:::ofile
    demo{{demo}}:::ofile

    greetings_json eg1@--> english_h
    greetings_json eg2@--> german_h 
    greetings_json eg3@--> french_h 
    greetings_json eg4@--> english_c 
    greetings_json eg5@--> german_c 
    greetings_json eg6@--> french_c 
    greetings_json eg7@--> languages_h

    languages_h es8@--> main_o
    english_h es1@--> english_o
    english_h es2@--> main_o
    german_h es3@--> german_o
    german_h es4@--> main_o
    french_h es5@--> french_o
    french_h es6@--> main_o
    %% languages_h es7@--> liblanguages_a

    main_c ec1@--> main_o
    %% languages_c ec2@--> languages_o
    liblanguages_a ec3@--> demo
    main_o ec4@--> demo

    english_c ex20@--> english_o
    german_c ex21@--> german_o
    french_c ex22@--> french_o

    english_o ex1@--> liblanguages_a
    german_o ex2@--> liblanguages_a
    french_o ex3@--> liblanguages_a

    greetings_yml eg8@--> greetings_json 


    %% classDef source fill:#f96
    %% classDef generated fill:#bbf,stroke:#3f3,stroke-width:5px,color:#fff,stroke-dasharray: 5 5
    %% classDef oexpanded fill:#03f,stroke:#3f3,stroke-width:5px,color:#fff,stroke-dasharray: 5 5
    %% classDef ofile fill:#03f,color:#f66
    %% classDef compile color:red,stroke-dasharray: 9,5,stroke-dashoffset: 900,animation: dash 25s linear infinite;

    classDef e_htosource stroke:#aaa,stroke-width:0.7x ,stroke-dasharray: 10,5;
    classDef e_generate stroke:#f00,stroke-with:1px;
    classDef e_compile stroke:#00f,stroke-width:1px;
    classDef e_expand stroke:#3f3,stroke-width:3px;
    classDef e_hidden stroke:#fff,stroke-width:1px;

   %% class eg1,eg2,eg3,eg4,eg5,eg6,eg7,eg8,eg9,eg10,eg11,eg12,eg13,eg14,eg15,eg16,eg17,eg18,eg19,eg20 e_generated;
   class json json;

    %% mermaid-animate-tag languages_h
    %% mermaid-animate-tag generated
    %% mermaid-animate-tag source
    %% mermaid-animate-tag oexpanded
    %% mermaid-animate-tag ofile
    %% mermaid-animate-tag e_generated
    %% mermaid-animate-tag json

```



