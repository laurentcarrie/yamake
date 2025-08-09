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

tests: test1 test2

png : out.png


out.png : run
	dot -Tpng -o out.png out.dot


build:
	cargo fmt
	cargo build --example c_program_example

doc:
	cargo doc

clean:
	rm -rf sandbox
	mkdir sandbox
