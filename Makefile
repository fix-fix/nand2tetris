.PHONY: test_watch
test_watch:
	cargo watch -c -x 'test'

.PHONY: test_compiler_watch
test_compiler_watch:
	cargo watch -c -x 'run --package compiler -- src/compiler/tests/inputs/11/Pong'
	# cargo watch -c -s clear -x 'run --package compiler -- src/compiler/tests/inputs/11/Pong'

.PHONY: examples_watch
examples_watch:
	cargo watch -q -c -x 'run --package compiler -- examples/snake'


.PHONY: os_watch
os_watch:
	cargo watch -q -c -s 'cp src/os/{,SysTest/}Sys.jack && cargo run --package compiler -- src/os/SysTest'

