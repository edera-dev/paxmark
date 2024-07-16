# paxmark

A utility for setting PaX markings on binaries.

## Why?

Some programs need to have certain features of PaX disabled because they either have
bugs or they require specific features like executable pages for JIT compilation.

`paxmark` provides the ability to set appropriate markings on systems which use the
`CONFIG_OPENPAX_XATTR_PAX_FLAGS` feature in their kernels.

## Usage

```
% paxmark -[pP|eE|mM|rR|sS] <binary>
```

Capitalized letters enable specific PaX feature flags; lower case letters disable the
feature flag.  Use `--help` to get a full list of controllable feature flags with this
tool.

## License

MIT
