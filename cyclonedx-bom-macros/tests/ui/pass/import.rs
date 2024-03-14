use cyclonedx_bom_macros::versioned;

#[versioned("1.0", "2.0")]
mod base {
    // Define `PI` for 1.0
    #[versioned("1.0")]
    pub const PI: f64 = 3.0;

    // Import `PI` for 2.0
    #[versioned("2.0")]
    pub use std::f64::consts::PI;
}

fn main() {
    assert!(v1_0::PI < v2_0::PI);
}
