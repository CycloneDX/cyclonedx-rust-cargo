use cyclonedx_bom_macros::versioned;

#[versioned("1.0", "2.0", "3.0")]
mod base {
    pub struct Foo {
        #[versioned("2.0", "3.0")]
        // This field only exists in versions 2.0 and 3.0.
        pub bar: u32,
    }
}

fn main() {
    // Version 1.0 does not have the `bar` field but 2.0 and 3.0 do.
    let _old_foo = v1_0::Foo {};
    let _new_foo = v2_0::Foo { bar: 0 };
    let _new_new_foo = v3_0::Foo { bar: 0 };
}
