# mungye

Merge JSONs/YAMLs together!

## Example:
`file2.json` will be converted to YAML, then merged with `file1.yaml` and the result will be sent to STDOUT:

```shell
$ mungye file1.yaml file2.json
```

Files are *always* open in read only mode. STDIN can be used as a source too, simply use a `-` (dash) to read from it. The only requirements (at the moment) when using STDIN is that the `--stdin-format` option is then required, because right now the tool can't guess the format of the data coming from STDIN.

There's also the `--force-format` option, to force the output to have a specific format, like (reusing the previous example):

```shell
$ mungye file1.yaml file2.json --force-format=json
```

This will work exactly the same as the previous example except that the result will be JSON instead of YAML.

Since the file arguments list must have a list of one filename, we can also use this tool to convert between formats, like:

```shell
# Convert JSON data to YAML data
$ mungye file2.json --force-format=yaml

# This is essentially the same, now this illustrates the usage with STDIN
$ cat file2.json | mungye - --force-format=yaml
```

# TODO
* Gather more test data
* Add unit tests
* Integration tests (at the command level, look into [assert_cmd](https://crates.io/crates/assert_cmd))
* Add TOML support
* Implement more array merging strategies (like `extend` and `zip`)
* Look into different strategies when folding the argument list
* Look into performance improvements
* Look into `async-std` and see if can have better performance (and if it's worth the trouble)
