# multimoon

An installer of [MoonBit language][moonbitlang] toolchain, directly inspired by '[rust-lang/rustup][rustup]'.

 > This is a [Lone Outpost Tech Open Source](https://github.com/lone-outpost-oss) project.

## Installation

Cargo install can be used if you already have the Rust language toolchain installed:

```shell
cargo install multimoon
```

Or download the executable file of your platform on the [release page][github-repo-releases]. 

You may need to make it executable and copy to proper path in your user or system path. An example for most Linux distros:

```shell
tar -Jxvf multimoon-0.1.0-amd64-linux.tar.xz
chmod +x multimoon-0.1.0-x64-linux/multimoon
sudo cp multimoon-0.1.0-x64-linux/multimoon /usr/local/bin/multimoon    # need root privileges
```

## Usage

Get the help:

```shell
multimoon
```

Update to the latest MoonBit toolchain:

```shell
multimoon update
```

Revert to an old version of toolchain:

```shell
multimoon toolchain update 2024-05-07
```

Use, backup and restore the core library: (basically used in core development)

```shell
multimoon core use ./path-to-my-core-repository-path
multimoon core backup my-core-dev-1
multimoon core restore my-core-dev-1
```



[moonbitlang]: https://www.moonbitlang.com/
[rustlang]: https://www.rust-lang.org/
[rustup]: https://github.com/rust-lang/rustup
[github-repo-releases]: https://github.com/lone-outpost-oss/multimoon/releases
