# wee

Help organize your workspace scripts!

```bash
cargo install wee
```



```bash
USAGE:
    wee [FLAGS] [cmd]

ARGS:
    <cmd>    Sets the input file to use

FLAGS:
    -d, --dump       Use dump to avoid repeatedly scanning configuration files
    -h, --help       Prints help information
    -s, --show       Show all available scripts
    -t, --time       Show execution time
    -V, --version    Prints version information
```

## Priority order

The priority above takes effect in the case of conflict.

```bash
# workspace level
wee.arc        # arc
package.json   # js/ts
cargo.toml     # rust
# user level
~/.wee.arc
```

## secrets


```js
env = {
    password: "pkpkppk"
}


```

## Todo

- [ ] Auto Completion 
- [ ] Auto Correction