alias d='cargo build && rust-gdb target/debug/wlab'
alias c='./clean.sh'
alias r='c && cargo run -- -l -a -i -S -o compiler_output wlang_src/*.wlang'

alias rr='r && ld ./compiler_output/*.o -o ./compiler_output/a.out && printf "=======\n" && ./compiler_output/a.out'
