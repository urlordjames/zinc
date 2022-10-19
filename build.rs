#[cfg(not(feature = "jit"))]
fn main() {}

#[cfg(feature = "jit")]
fn main() {
    cc::Build::new().file("zinc_std_c.c").compile("zinc_std_c");
}