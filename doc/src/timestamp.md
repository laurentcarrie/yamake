# timestamp

Make uses the timestamp of a target to determine if it needs to be rebuilt or not. Let's consider this scenario :

```make

a.o: a.c a.h
    gcc -c $@ -o $<

libproject.a : a.o b.o c.o
    ar rcs $@ $^

```

if you add a comment to `a.h`, `a.o` gets rebuild, but because you only added a comment, `a.o` is unchanged after the rebuild. Its timestamp changes though,
and this will trigger the useless link of `libproject.a`, and all other artefacts down the build path.

If you have a code generator, this is even worse, as many files get involved.

Yamake solves this issue by considering the digest of the files, and not their timestamp. In our example, as `a.o` has same digest after the build, the rebuild is not propagated.

