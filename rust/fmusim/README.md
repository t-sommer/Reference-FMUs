# fmusim

A command-line FMU simulation tool written in Rust. It can display information
about an FMU, validate it, and run a simulation.

```bash
Usage: fmusim <COMMAND>

Commands:
  info      Display information about an FMU
  validate  Validate an FMU
  simulate  Simulate an FMU
  help      Print this message or the help of the given subcommand(s)
```

## Building on Linux

`fmusim` depends on the sibling crates `fmi-rs`, `fmi-rs-xsd` and
`fmi-rs-cvode`. Their build scripts compile some C sources and, on first build,
**download and build libxml2 and SUNDIALS/CVODE from source** into a per-target
`vendor/` directory. This means the first build needs network access and takes a
few minutes; subsequent builds reuse the vendored static libraries.

### Prerequisites

Install a Rust toolchain (via [rustup](https://rustup.rs)) and the following
system packages:

| Tool          | Used for                                      |
|---------------|-----------------------------------------------|
| C compiler    | compiling the C sources, libxml2 and SUNDIALS |
| `cmake`       | configuring/building libxml2 and SUNDIALS     |
| `make`        | the CMake "Unix Makefiles" generator          |
| `curl`        | downloading the libxml2 and SUNDIALS sources  |

On Debian/Ubuntu:

```bash
sudo apt-get update
sudo apt-get install build-essential cmake curl
```

On Fedora/RHEL:

```bash
sudo dnf install gcc gcc-c++ make cmake curl
```

> Note: flex/bison are **not** required — the generated parser sources for the
> structured variable name validator are checked into the repository.

### Build

From the `rust/fmusim` directory (or anywhere inside the `rust/` workspace):

```bash
cargo build            # debug build
cargo build --release  # optimized build
```

The first invocation prints warnings like
`Sundials not found. Downloading and building CVODE ...` and
`libxml2s.lib not found. Downloading and building libxml2 ...` — this is
expected. The vendored libraries are cached under
`fmi-rs/fmi-rs-xsd/vendor/<target>` and `fmi-rs/fmi-rs-cvode/vendor/<target>`,
so later builds are fast.

The resulting binary is at `target/debug/fmusim` (or `target/release/fmusim`).

### Run

```bash
./../target/debug/fmusim --help
./../target/debug/fmusim info <path/to/model.fmu>
./../target/debug/fmusim validate <path/to/model.fmu>
./../target/debug/fmusim simulate <path/to/model.fmu>
```

### Test

The integration tests run `fmusim` against the `Feedthrough` Reference FMU for
FMI 2.0 and 3.0. These `.fmu` files are build artifacts (they are not checked
in), so you have to build them first and copy them into the test resources
directory where the tests look for them:

```text
fmusim/tests/resources/Reference-FMUs/2.0/Feedthrough.fmu
fmusim/tests/resources/Reference-FMUs/3.0/Feedthrough.fmu
```

Build them with CMake from the repository root, once per FMI version (see
[Build the FMUs](../../README.md#build-the-fmus) in the top-level README for the
full instructions), then copy them into place:

```bash
# Run from the Reference-FMUs repository root
for v in 2 3; do
  cmake -S . -B build/fmi$v -DFMI_VERSION=$v -DWITH_FMUSIM=OFF
  cmake --build build/fmi$v --target Feedthrough --config Release
done

mkdir -p rust/fmusim/tests/resources/Reference-FMUs/2.0 \
         rust/fmusim/tests/resources/Reference-FMUs/3.0
cp build/fmi2/fmus/Feedthrough.fmu rust/fmusim/tests/resources/Reference-FMUs/2.0/
cp build/fmi3/fmus/Feedthrough.fmu rust/fmusim/tests/resources/Reference-FMUs/3.0/
```

Building the FMUs needs the same toolchain as building `fmusim` (a C compiler,
`cmake` and `make`). Then run the tests:

```bash
cargo test
```
