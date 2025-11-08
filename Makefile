.PHONY: run build doc png test1 test2 tests help json json-errors runPdf
.DEFAULT: help


include scripts/colorprint

sandbox:=sandbox
srcdir:=demo_projects
my_C_tool:=yamake/target/debug/examples/C_demo_project
my_Pdf_tool:=yamake/target/debug/examples/Latex_demo_project


help: ## Show this help message
	@echo "Available commands:"
# 	echo $(MAKEFILE_LIST)
#	was $(MAKEFILE_LIST) but fails if there is an include. So we use Makefile
	@grep -E '^[ a-zA-Z_-]+:.*?## .*$$' Makefile | awk 'BEGIN {FS = ":.*?## "}; {printf "$(Red)%-20s$(Color_Off) : $(Blue)%s$(Color_Off)\n", $$1, $$2}'



run : build ## use our demo C tool, to compile our demo C project
	@mkdir -p $(sandbox)
	@printf "\n$(White)$(On_Blue)run yamake tool to build the C program$(Color_Off)\n"
	$(my_C_tool) $(srcdir) $(sandbox)
	@printf "\n$(White)$(On_Blue)run the compiled program$(Color_Off)\n"
	./$(sandbox)/project_1/demo
	@printf "\n$(White)$(On_Blue)end of run$(Color_Off)\n"

runPdf : build ## use our demo Latex tool, to compile our demo Latex to Pdf
	@mkdir -p $(sandbox)
	@printf "\n$(White)$(On_Blue)run yamake tool to build the Pdf doc generator$(Color_Off)\n"
	$(my_Pdf_tool) $(srcdir) $(sandbox)
	@printf "\n$(White)$(On_Blue)run the tool to build the pdf$(Color_Off)\n"
	ls -l $(sandbox)/project_latex/*.pdf
	@printf "\n$(White)$(On_Blue)end of run$(Color_Off)\n"

test1 : build
	$(my_C_tool) $(srcdir) $(sandbox) && \
	rm $(sandbox)/project_1/demo && \
	$(my_C_tool) $(srcdir) $(sandbox) && \
	ls $(sandbox)/project_1/demo

test2 : build
	$(my_C_tool) $(srcdir) $(sandbox) && \
	rm $(sandbox)/project_1/main.o && \
	$(my_C_tool) $(srcdir) $(sandbox) && \
	ls $(sandbox)/project_1/demo

test_pdf : build
	$(my_C_tool) $(srcdir) $(sandbox) && \
	rm $(sandbox)/project_1/main.o && \
	$(my_C_tool) $(srcdir) $(sandbox) && \
	ls $(sandbox)/project_1/demo


test: build
	cd yamake && cargo test

png : run
	@printf "$(Blue)when running$(Color_Off) $(Red)our demo tool$(Color_Off)$(Blue), dot files of the build graph are generated$(Color_Off) => use graphviz to get png files\n"
	dot -Tpng -o $(sandbox)/before-scan.png $(sandbox)/before-scan.dot
	dot -Tpng -o $(sandbox)/after-scan.png  $(sandbox)/after-scan.dot
	cp $(sandbox)/*scan.png doc/src/howto/.

build: ## build our demo tool, written in rust, using yamake. This demo tool is a builder for our demo project, a C project in $(srcdir) directory
	@printf "\n$(White)$(On_Blue)build the demo$(Color_Off)\n"
	( cd yamake ; cargo fmt && 	cargo build --example C_demo_project )
	@printf "finished building $(Red)$(my_C_tool)$(Color_Off)\n"
	( cd yamake ; cargo fmt && 	cargo build --example Latex_demo_project )
	@printf "finished building $(Red)$(my_Pdf_tool)$(Color_Off)\n"


clean:
	rm -rf $(sandbox)
	mkdir $(sandbox)

show-report: ## inspect the json result
	jq ".items[] | { target:.target,status:.status}" $(sandbox)/make-report.json

# ANCHOR: json-errors
show-errors: ## use make-report.json to find errors and print relevant logs
	@printf "\n$(White)$(On_Blue)use make-report.json to find error and print relevant logs$(Color_Off)\n"
	@sandbox=$(shell jq '.sandbox ' $(sandbox)/make-report.json) ;\
	srcdir=$(shell jq '.srcdir ' $(sandbox)/make-report.json) ;\
	stdoutfiles=$(shell jq '.items[] | select(.status == "Failed") | .stdout ' $(sandbox)/make-report.json) ; \
	stderrfiles=$(shell jq '.items[] | select(.status == "Failed") | .stderr ' $(sandbox)/make-report.json) ; \
	for f in $$stdoutfiles ; do \
		echo "----------- stdout " ; \
		echo "ERROR while building target ${Red}$$f${Color_Off}" ; \
		echo "----------- " ; \
		cat $$sandbox/$$f | sed "s#$$sandbox#$$srcdir#g" ; \
		echo "----------- end stdout" ; \
	done ; \
	for f in $$stderrfiles ; do \
		echo "----------- stderr " ; \
		echo "ERROR while building target ${Red}$$f${Color_Off}" ; \
		echo "----------- " ; \
		cat $$sandbox/$$f | sed "s#$$sandbox#$$srcdir#g" ; \
		echo "----------- end stderr" ; \
	done ; \
	echo "\n$(White)$(On_Blue)end show-errors$(Color_Off)\n"

# ANCHOR_END: json-errors
