alias d='cargo build && rust-gdb --args target/debug/wlab -l -a -i -S -o compiler_output wlang_src/*.wlang'
alias c='./clean.sh'
alias r='c && cargo run -- -l -a -i -S -o compiler_output wlang_src/*.wlang'

alias rr='r && ld ./compiler_output/*.o -o ./compiler_output/a.out && printf "=======\n" && ./compiler_output/a.out'
alias rd='r && ld ./compiler_output/*.o -o ./compiler_output/a.out && gdb ./compiler_output/a.out'
