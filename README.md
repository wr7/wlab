
WLAB (WLAng Bootstrap) is an LLVM-based compiler written from scratch.

Current features of WLang include:
- [Helpful error messages](#error-messages)
- Name Mangling
- Visibility
- Function and struct attributes
- Multi-file support
- If statements
- Type inference
- Structs

Simple example project:
```rust
#![declare_crate(hello_world)]

#[no_mangle]
fn _start() {
    std::println("hello from wlang!");

    let twenty_one = 21;

    // Parenthesis are not needed for if statements
    if 9 + 10 == twenty_one - 2 {
        std::println("This will be printed");
    } else {
        std::println("This will not be printed");
    }

    // If statements can act like the C ternary operator
    let text = if true {"this will also be printed"} else {"this wont"};

    std::println(text);

    std::exit(0);
}
```

### Error messages
Example error messages:

src:
```rust
#![declare_crate(test_error)]

#[no_mangle]
fn _start() {
    /****************\
    |*              *|
    |*              *|
    |*  This long   *|
    |*   comment    *|
    |*   will be    *|
    |* omitted from *|
    |*  the error   *|
    |*    message   *|
    |*              *|
    |*              *|
    \****************/
)
```

error:

![Screenshot from 2024-08-17 20-21-42](https://github.com/user-attachments/assets/e1ef3444-da6e-469c-8164-cdec40fe36c7)

src:
```rust
#![declare_crate(a)]

#[no_mangle]
fn _start() {
    let var = 5; // by default all variables are immutable

    var = 10;
}

```
error:

![Screenshot from 2024-08-17 20-24-14](https://github.com/user-attachments/assets/0a3bc7c5-7f02-4775-ab58-c935ff27ecb6)

