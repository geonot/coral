fn main() {
    cc::Build::new()
        .file("runtime/runtime.c")
        .compile("runtime");
}
