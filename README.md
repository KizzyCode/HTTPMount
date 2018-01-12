[![License](https://img.shields.io/badge/License-BSD%202--Clause-blue.svg)](https://opensource.org/licenses/BSD-2-Clause)

# About
This application allows you to mount a HTTP-resource like a file.

# Dependencies
This library relies on a working FUSE-installation.

## Build-Dependencies
If you want to build the library, you'll also need [libselect](https://github.com/KizzyCode/libselect) which is required
for network-operations.

### Cargo-Dependencies
Own libraries: 
 - [http_file](https://github.com/KizzyCode/http_file)
    - [network_io](https://github.com/KizzyCode/network_io)
    - [http](https://github.com/KizzyCode/http)

Foreign libraries:
 - fuse
 - libc
 - time

 
# Build Binary and Sourcecode-Documentation
To build the documentation, go into the projects root-directory and run `cargo doc --release`; to open the documentation
in your web-browser, run `cargo doc --open`.

To build the library, go into the projects root-directory and run `cargo build --release`; you can find the build in
target/release.

# Build and Install using [Homebrew](https://brew.sh)
I provide a custom homebrew-formula ("kc-httpfs") in [my cask](https://github.com/KizzyCode/homebrew-formulas).

To install this application verify that you have a working FUSE-installation (you can install OSXFUSE with
`brew cask install osxfuse`). Then add my cask to your homebrew-installation using `brew tap KizzyCode/formulas` and
install it using `brew install --HEAD kc-httpfs` (this will install the binary in "/usr/local/bin")