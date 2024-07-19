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

#[no_mangle]
fn _start() {
    std::println("hello from wlang!");

    let twenty_one = 21;

    if 9 + 10 == twenty_one - 2 {
        std::println("This will be printed");
    } else {
        std::println("This will not be printed");
    }

    let text = if true {"this will also be printed"} else {"this wont"};

    std::println(text);

    std::exit(0);
}
```

### Error messages
Some sample errors:

src:
```
#![declare_crate(test_error)]

#[no_mangle]
fn _start() {

    std::exit(0);
)
```

error:
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
6 |     std::exit(0);
7 | )
  | ^
  | closing bracket here
```
##
src:
```
#![declare_crate(test_error)]

fn add_ten(num: i32) -> i32 {
    num + 10
}

#[no_mangle]
fn _start() {
    let cool_number = add_ten("nine");

    std::exit(0);
}
```
error:
```
 Unexpected type: expected `i32`; got `str`
---------------------------------------------
7 | #[no_mangle]
8 | fn _start() {
9 |     let cool_number = add_ten("nine");
  |                               ^^^^^^
  | value here of type `str`
```
