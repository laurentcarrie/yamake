.PHONY: run build doc png

run : build
	clear
	mkdir -p sandbox
	./target/debug/examples/c_program_example ../c_example sandbox
	./sandbox/project_1/demo


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