
.SUBDIRS: dir1 dir2 dir3
.PHONY: clean app
.DEFAULT: app

app : dir3/app

clean:
	rm -f dir3/app 
	rm -f $$(find . -name "*.a") 
	rm -f $$(find . -name "*.o")