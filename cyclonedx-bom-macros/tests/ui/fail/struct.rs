use cyclonedx_bom_macros::versioned;

#[versioned("1.3")]
struct Foo; //~ ERROR: cannot parse module

fn main() {}
