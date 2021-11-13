# hline

`hline` is a very small command line utility designed to highlight lines in log files. In practice, I've found that tuning the context that `grep` gives me when `tail -f`ing a log can be quite cumbersome. Oftentimes, all I really care about is seeing that a certain message happened and some surrounding context. `hline` fills that niche!

## Usage

```
hline 0.1.0
Highlights lines that match the given regular expression

USAGE:
    hline [FLAGS] <pattern> [filename]

FLAGS:
    -i, --ignore-case    Ignore case when performing matching. If not specified, the matching is case-sensitive.
    -h, --help           Prints help information

ARGS:
    <pattern>     The regular expression to search for. Note that this is not anchored, and if anchoring is desired,
                  should be done manually with ^ or $.
    <filename>    The file to scan. If not specified, reads from stdin
```

### [Changelog](CHANGELOG.md)
