# capture error

this is a classic error that happens when you don't use automatic rules :

```makefile

a.o : a.c
    gcc -c -o b.o a.c

main.o : main.c
	gcc -c -o $@ $^

app : a.o main.o
    gcc -o $@ $^

```

when running make, you won't get an error because you will compile correctly, but you will get `b.o` instead of expected `a.o`. In the next
steps of the build, linking `app` will fail because `a.o` does not exist, or, worse, it was not updated and you have a wrong version. This error is hard to spot.

after build, yamake checks that the target file exists.