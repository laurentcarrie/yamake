.PHONY: run build

run : build
	mkdir -p sandbox
	./target/debug/examples/c_program_example ../c_example sandbox
	./sandbox/project_1/demo 


build:
	cargo fmt
	cargo build --example c_program_example

