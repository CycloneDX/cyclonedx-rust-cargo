use cyclonedx_bom_macros::versioned;

#[versioned("1.0", "2.0")]
mod base {
    // Define `Foo` for 1.0 twice
    #[versioned("1.0")]
    pub struct Foo;

    #[versioned("1.0")]
    pub struct Foo;
}

fn main() {
    let _ = v1_0::Foo;
}
