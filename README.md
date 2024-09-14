WLAB (WLAng Bootstrap) is an LLVM-based compiler written from scratch.

### Features
- [Helpful error messages](#error-messages)
- Name Mangling
- Visibility
- Function and struct attributes
- Multi-file support
- If statements
- Type inference
- Structs

### Example project
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

fn main() {
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
}
```

### Error messages
Example error messages:

src:
```rust
#![declare_crate(test_error)]

fn main() {
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

![Screenshot from 2024-09-14 16-01-08](https://github.com/user-attachments/assets/3c46d346-6c96-46cc-9acb-e184823119e2)

src:
```rust
#![declare_crate(a)]

struct Foo {
    x: i32,
    y: i32,
};

fn main() {
    let foo = Foo { x: 6, y: 12 }; // by default all variables are immutable

    foo.y = 10;
}
```
error:

![Screenshot from 2024-09-14 16-03-48](https://github.com/user-attachments/assets/70c69168-00f5-4c8d-bfb3-9b5841b8f5ae)
