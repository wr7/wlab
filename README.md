
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

struct Messages {
    first: str,
    second: str,
    override: bool,
};

fn get_messages() -> Messages {
    /*
     * The last statement in a code block will be implicitly returned if it is 
     * not terminated by a semicolon
     */

    Messages {
        override: true,
        first: "This will be printed first",
        second: "This will be not printed",
    }
}

#[no_mangle]
fn _start() {
    // Variable types are automatically inferred //
    let mut messages = get_messages();

    // `if` statements do not require parenthesis //
    if messages.override {
        messages.second = "This will be printed second";
    }

    std::println(messages.first);
    std::println(messages.second);

    // `if` can be used as an expression //
    let third_message = if 2 + 2 == 4 {
        "This will be printed third"
    } else {
        "This will not be printed"
    };

    std::println(third_message);

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

struct Foo {
    x: i32,
    y: i32,
};

#[no_mangle]
fn _start() {
    let foo = Foo { x: 6, y: 12 }; // by default all variables are immutable

    foo.y = 10;
}
```
error:

![Screenshot from 2024-08-22 14-58-16](https://github.com/user-attachments/assets/6b141d75-68c9-4274-937a-9864d397486f)

