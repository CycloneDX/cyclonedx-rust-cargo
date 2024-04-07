use cyclonedx_bom_macros::versioned;

#[versioned("1.0", "2.0")]
mod base {
    #[versioned("1.0")]
    #[versioned("2.0")]
    struct Foo;
}

fn main() {}
