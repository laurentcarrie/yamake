.PHONY: run build doc png test1 test2 tests help json json-errors runPdf doc-serve \
demo-cluster
.DEFAULT: help


include scripts/colorprint

sandbox:=sandbox
srcdir:=demo_projects
my_C_tool:=yamake/target/debug/examples/C_demo_project
expand_demo:=yamake/target/debug/examples/expand_demo
my_Pdf_tool:=yamake/target/debug/examples/Latex_demo_project


help: ## Show this help message
	@echo "Available commands:"
# 	echo $(MAKEFILE_LIST)
#	was $(MAKEFILE_LIST) but fails if there is an include. So we use Makefile
	@grep -E '^[ a-zA-Z_0-9-]+:.*?## .*$$' Makefile | awk 'BEGIN {FS = ":.*?## "}; {printf "$(Red)%-20s$(Color_Off) : $(Blue)%s$(Color_Off)\n", $$1, $$2}'

run_1: build ## demo : use our demo C tool, to compile our demo C project
## ANCHOR: run_1
	@rm -rf $(sandbox)
	@mkdir -p $(sandbox)
	@git checkout HEAD -- demo_projects
	@printf "\n$(White)$(On_Blue)run yamake tool to build the C program$(Color_Off)\n"
	$(my_C_tool) $(srcdir) $(sandbox)
	@printf "\n$(White)$(On_Blue)run the compiled program$(Color_Off)\n"
	@./$(sandbox)/project_1/demo
	@printf "\n$(White)$(On_Blue)end of run$(Color_Off)\n"
	@cp $(sandbox)/make-report.json doc/src/make-reports/run_1.json
## ANCHOR_END: run_1

run_2: build ## demo : build, delete an artefact, build again
## ANCHOR: run_2
	@rm -rf $(sandbox)
	@mkdir -p $(sandbox)
	@git checkout HEAD -- demo_projects
	@printf "\n$(White)$(On_Blue)run yamake tool to build the C program$(Color_Off)\n"
	$(my_C_tool) $(srcdir) $(sandbox)
	@printf "\n$(White)$(On_Blue)delete add.o, build again$(Color_Off)\n"
	rm $(sandbox)/project_1/add.o
	$(my_C_tool) $(srcdir) $(sandbox)
	@printf "\n$(White)$(On_Blue)end of run$(Color_Off)\n"
	@cp $(sandbox)/make-report.json doc/src/make-reports/run_2.json
## ANCHOR_END: run_2

run_3: build ## demo : make a change in the source, that has no effect (eg, add a comment)
## ANCHOR: run_3
	@rm -rf $(sandbox)
	@mkdir -p $(sandbox)
	@git checkout HEAD -- demo_projects
	@printf "\n$(White)$(On_Blue)run yamake tool to build the C program$(Color_Off)\n"
	$(my_C_tool) $(srcdir) $(sandbox)
	@printf "\n$(White)$(On_Blue)add a comment$(Color_Off)\n"
	echo "// a C comment " >> $(srcdir)/project_1/add.c
	$(my_C_tool) $(srcdir) $(sandbox)
	@printf "\n$(White)$(On_Blue)end of run$(Color_Off)\n"
	@cp $(sandbox)/make-report.json doc/src/make-reports/run_3.json
## ANCHOR_END: run_3

run_4: build ## demo : make a change in the source, that is an coding error
## ANCHOR: run_4
	@rm -rf $(sandbox)
	@mkdir -p $(sandbox)
	@git checkout HEAD -- demo_projects
	@printf "\n$(White)$(On_Blue)run yamake tool to build the C program$(Color_Off)\n"
	$(my_C_tool) $(srcdir) $(sandbox)
	@printf "\n$(White)$(On_Blue)add a comment$(Color_Off)\n"
	echo "blah blah " >> $(srcdir)/project_1/add.c
	$(my_C_tool) $(srcdir) $(sandbox)
	@printf "\n$(White)$(On_Blue)end of run$(Color_Off)\n"
	@cp $(sandbox)/make-report.json doc/src/make-reports/run_4.json
## ANCHOR_END: run_4

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
	$(my_Pdf_tool) --nb-workers 1 $(srcdir) $(sandbox)
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


test: build
	cd yamake && cargo test

png : run
	@printf "$(Blue)when running$(Color_Off) $(Red)our demo tool$(Color_Off)$(Blue), dot files of the build graph are generated$(Color_Off) => use graphviz to get png files\n"
	dot -Tpng -o $(sandbox)/before-scan.png $(sandbox)/before-scan.dot
	dot -Tpng -o $(sandbox)/after-scan.png  $(sandbox)/after-scan.dot
	cp $(sandbox)/*scan.png doc/src/howto/.

build: ## build our demo tool, written in rust, using yamake. This demo tool is a builder for our demo project, a C project in $(srcdir) directory
	@printf "\n$(White)$(On_Blue)build the demo$(Color_Off)\n"
	( cd yamake ; cargo fmt && 	cargo build --examples  )
	@printf "finished building\n"

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


doc-serve: ## run mdbook and serve
	cd doc && mdbook serve


demo-expand: build ## demo : demo for a cluster of generated files
	@rm -rf $(sandbox)
	@mkdir -p $(sandbox)
	@git checkout HEAD -- demo_projects
	@printf "\n$(White)$(On_Blue)run the builder$(Color_Off)\n" && \
	$(expand_demo) $(srcdir) $(sandbox) && \
	printf "\n$(White)$(On_Blue)run the builded demo program$(Color_Off)\n" && \
	./$(sandbox)/demo_expand/demo && \
	printf "\n$(White)$(On_Blue)end of run$(Color_Off)\n" && \
	cp $(sandbox)/make-report.json doc/src/make-reports/run_1.json
