fn main() {
    cc::Build::new()
        .file("src/c/src/logger_proxy.c")
        .include("src/c/include")
        .compile("logger_proxy");
}