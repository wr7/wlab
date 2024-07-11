WLAB (WLAng Bootstrap) is an LLVM-based compiler written from scratch.

Current features of WLang include:
- [Helpful error messages](#error-messages)
- Name Mangling
- Visibility
- Function attributes
- Multi-file support
- If statements
- Type inference

Simple example project:
```
#![declare_crate(hello_world)]

fn println(val: str) {
    write(1, val);
    write(1, "\n");
}

#[no_mangle]
fn _start() {
    println("hello from wlang!");

    let twenty_one = 21;

    if 9 + 10 == twenty_one - 2 {
        println("This will be printed");
    } else {
        println("This will not be printed");
    }

    let text = if true {"this will also be printed"} else {"this wont"};

    println(text);

    exit(0);
}
```

### Error messages
The following code:
```
#![declare_crate(test_error)]

#[no_mangle]
fn _start() {

    exit(0);
)
```

Will make WLAB give the following error:
```
 Error while parsing code: mismatched brackets
------------------------------------------------
2 |
3 | #[no_mangle]
4 | fn _start() {
  |             ^
  | opening bracket here
 ...
5 |
6 |     exit(0);
7 | )
  | ^
  | closing bracket here
```
