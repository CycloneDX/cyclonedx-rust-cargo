use cyclonedx_bom_macros::versioned;

#[versioned("1.0", "2.0")]
mod base {
    pub struct Foo {
        #[versioned("2.0")]
        // This field only exists in version 2.0.
        pub bar: u32,
    }
}

fn main() {
    // Version 1.0 does not have the `bar` field but 2.0 does.
    let _old_foo = v1_0::Foo {};
    let _new_foo = v2_0::Foo { bar: 0 };
}
