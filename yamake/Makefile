.PHONY: run build doc png test1 test2 tests

run : build
	clear
	mkdir -p sandbox
	./target/debug/examples/c_program_example ../c_example sandbox
	./sandbox/project_1/demo

test1 : build
	clear
	./target/debug/examples/c_program_example ../c_example sandbox
	rm sandbox/project_1/demo
	./target/debug/examples/c_program_example ../c_example sandbox
	ls sandbox/project_1/demo

test2 : build
	clear
	./target/debug/examples/c_program_example ../c_example sandbox
	rm sandbox/project_1/main.o
	./target/debug/examples/c_program_example ../c_example sandbox
	ls sandbox/project_1/demo

test: build
	cargo test

png : out.png


before-scan.png after-scan.png : run
	dot -Tpng -obefore-scan.png before-scan.dot
	dot -Tpng -o after-scan.png  after-scan.dot
	cp *scan.png ../doc/src/.

png : before-scan.png after-scan.png


build:
	cargo fmt
	cargo build --example c_program_example

doc:
	cargo doc

clean:
	rm -rf sandbox
	mkdir sandbox
