<!-- toc -->


# expand

## usual cases, all files are known

Standard makefiles define rules, eg :
```make

%.o : %.c
    gcc -c $< -o $@

libproject.a : a.o b.o c.o
    ar rcs $@ $^
```

which build the graph :

``` mermaid
---

---

flowchart TD

a.c --> a.o --> libproject.a
b.c --> b.o --> libproject.a
c.c --> c.o --> libproject.a

```

---

## now we have code generators, lists of files are unknown

Now, image that the list of files is unknown, because we have a code generator `cg`, that takes an input file `config.yml`.
Our Makefile would be the impossible : 

```make

%.o : %.c
    gcc -c $< -o $@

unknown list of files : config.yml
    cg $^

libproject.a : unknown list of files
    ar rcs $@ $^

```

Usual solution with make would be to have a directory or filename pattern for the generator, and use a wildcard.

---

## yamake expand

Yamake has another approach, names expansion. Our initial known graph has nodes `main.c`, `main.o`, `app`, `libapp.a` and `config.yml`.
We have a code generator that will generate cfiles from `config.yml`, but we don't have that list.

Yamake allows nodes to have an `expand` method. In our example, the expansion will create 6 nodes : `a.c`, `a.o`, ... and the edges, this is shown in orange.

```mermaid

---

animate-yml-file: expand2.yml

---

flowchart TD 

config.yml econfa@--> ac 
ac eacao@--> ao 
ao ealib@--> libapp.a

config.yml econfb@--> bc 
bc ebcbo@--> bo 
bo eblib@--> libapp.a

config.yml econfc@--> cc 
cc eccco@--> co 
co eclib@--> libapp.a

ac(a.c) ;
bc(b.c) ;
cc(c.c) ;
ao(a.o) ;
bo(b.o) ;
co(c.o) ;

main.c --> main.o --> app
libapp.a --> app

classDef class_active_node   stroke-width:1px,color:black,stroke:black ;
classDef class_expanded_node   stroke-width:1px,color:black,stroke:orange ;
classDef class_hidden_node   stroke-width:1px,color:white,stroke:white,stroke-dasharray: 9,5,stroke-dashoffset: 900 ;

classDef class_active_edge   stroke-width:3px,color:orange,stroke:orange;
classDef class_hidden_edge   stroke-width:1px,stroke:white ;
classDef class_expanded_edge   stroke-width:3px,stroke:orange ;
classDef class_animate_edge  stroke-dasharray: 9,5,stroke-dashoffset: 900,animation: dash 25s linear infinite,color black;



%% mermaid-animate-tag ac
%% mermaid-animate-tag bc
%% mermaid-animate-tag cc
%% mermaid-animate-tag ao
%% mermaid-animate-tag bo
%% mermaid-animate-tag co

%% mermaid-animate-tag econfa
%% mermaid-animate-tag econfb
%% mermaid-animate-tag econfc

%% mermaid-animate-tag eacao
%% mermaid-animate-tag ebcbo
%% mermaid-animate-tag eccco

%% mermaid-animate-tag ealib
%% mermaid-animate-tag eblib
%% mermaid-animate-tag eclib


```
