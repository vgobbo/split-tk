# split-tk
Helpers for splitting stuff, like files.

Simply install with `cargo install split-tk`, and start using it.

For example, suppose you want to split a sequence of numbers in sets of ten:
```bash
$ seq 1 35 | split-tk --size 10 -- echo '{}'
1,2,3,4,5,6,7,8,9,10
11,12,13,14,15,16,17,18,19,20
21,22,23,24,25,26,27,28,29,30
31,32,33,34,35
```

Use `{}` (just like GNU parallel) to specify where the batch should be placed in the command. It can be specified as many times as you want.

If playing with `json` this can be inconvenient since `{}` is a valid `json` itself, you can specify a different replacement tag with `-g`, for example:
```bash
$ seq 1 35 | split-tk --size 10 -g '!!' -- echo '!!'
1,2,3,4,5,6,7,8,9,10
11,12,13,14,15,16,17,18,19,20
21,22,23,24,25,26,27,28,29,30
31,32,33,34,35
```

## To-Do

The following are some changes and features being considered in future versions. Feel free to suggest others.

- **[FEAT]** File as an input.
- **[FEAT]** Parallel command execution.
- **[FEAT]** Redirect to command stdin instead of command line.