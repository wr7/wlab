args='--lex --parse --llvm-ir --asm --output-dir=compiler_output wlang_src/*.wlang std/std.wlang'

alias c='./clean.sh'
alias d='c && path="`./build_with_path.sh`" && rust-gdb --args "$path" '"$args"''
alias r="c && cargo run -- $args"
alias wtool="cargo run -p wtool --"

alias rr='r && ld ./compiler_output/*.o -o ./compiler_output/a.out && printf "=======\n" && ./compiler_output/a.out'
alias rd='r && ld ./compiler_output/*.o -o ./compiler_output/a.out && gdb ./compiler_output/a.out'
