

libproject.a : ../dir1/a1.o ../dir2/a2.o
	ar rcs $@ $^

main.o: ../dir1/a1.h ../dir2/a2.h
	gcc -c -I .. -o main.o main.c

app : main.o libproject.a
	gcc -o app main.o libproject.a