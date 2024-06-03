MultiMoon Version 0.1.2 (2024-05-30)
===============================

 - fix: upgrade yanked `zip` v1.x package to avoid errors on `cargo install multimoon` without `--locked` parameter. 

MultiMoon Version 0.1.1 (2024-05-21)
===============================

 - fix: failed to create `.moon/bin` and `.moon/lib` on fresh new installations

MultiMoon Version 0.1.0 (2024-05-20)
===============================

MultiMoon is a version manager for the [MoonBit language][moonbitlang], directly inspired from [rustup][rustup]

Initial version. Supports following commands:

 - `show` to validate current toolchain version and intergrity against online registry.
 - `update` to install the latest MoonBit toolchain.
 - `toolchain list` to show all available toolchains.
 - `toolchain update` to install a certain version of toolchain.
 - `core list`, `core backup` & `core restore` to backup and restore core lib.

An online registry powered by CloudFlare (Worker, D1 & R2) and GitHub (Actions) is provided at <https://multimoon.lopt.dev/>. Any new version of MoonBit toolchain is tracked and archived daily. 

[moonbitlang]: https://www.moonbitlang.com/
[rustup]: https://rustup.rs/
