# Oolang Compiler

Compiler for the oolang programming language.

### Goals of Oolang :
- to be portable : Oolang runs on a VM and bytecode files can be shared between all systems an Oolang VM is available on.
- to be secure : Oolang is intented to be a language for running mods, plugins, and untrusted code. VM's memory usage and speed can be limited.
- to be versatile : Oolang's standard library can easily be modified or replaced, to allow Oolang programs to interact with your own program as a mod or plugin.
The VM is available as a library.

### Design and features :
Oolang is (as the name suggests) an object-oriented language. It has features similar to Java with the added convenience of Rust's "expression-oriented" syntax.
For example, this is valid Oolang code :
```rs
let a: u64 = {
    let b: u64 = 5;
    let c: u64 = 6;
    b + c // Just like in Rust, a missing semicolon at the end of a block means the block evaluates to the last expression in it.
};

// Notice the lack of parentheses around the condition
let b: u64 = if a == 11 { 7 } else { 10 }; // "if else" statements, "match" statements as well as "loop" statements are actually expressions;
let c: u64 = loop { break 11; };
```
And this is how an Oolang source file containing the entry point of the program would look like
```rs
mod author::project_name; // This is similar to Java's "package" statements

// Generics are supported
pub inter Wrapper<T> {
    pub fn unwrap() -> T;
}

pub class SimpleWrapper<T> impl Wrapper<T> {
    inside: T;

    pub SimpleWrapper(inside: T) {
        this.inside = inside;
    }

    pub fn unwrap() -> T {
        inside
    }
}

// Generics allow for bounds. "impl" adds an interface as a requirement and ":" adds a super class as a requirement (or super interface depending on the context.)
// U would be anything that implements Wrapper<T>
pub class WrapperWrapper<T, U impl Wrapper<T>>: SimpleWrapper<U> {
    pub WrapperWrapper(inside: U) {
        super(inside);
    }
}

// U Would be anything that extends SimpleWrapper<T>
pub class SimpleWrapperWrapper<T, U: SimpleWrapper<T>>: SimpleWrapper<U> {
    pub SimpleWrapperWrapper(inside: U) {
        super(inside);
    }
}

pub class Main {
    // Almost the same as Java's "main" method
    pub static fn main(String[] args) {
        // I hope to implement type inference for generics at some point
        let wrapped_integer: SimpleWrapper<U64> = new SimpleWrapper(10);
        let wrapped_wrapped_integer: SimpleWrapperWrapper<U64, SimpleWrapper<U64>> = new SimpleWrapperWrapper(wrapped_integer);
    }
}
```

### Where it's currently at :
Oolang is currently being completely reworked for the 5th (and probably final) time. I am currently working on code generation.

### Can I join ?
Sure ! If you need help with something my discord is down there.

### Contact :
Discord : telnobynoyator#3156
