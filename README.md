# u8pls 🚀🚀🚀

> convert file from any encoding to utf8

***⚠️⚠️⚠️ ALWAYS BACKUP YOUR FILE BEFORE CONVERT !!!***

***⚠️⚠️⚠️ THIS TOOL USE AS YOUR OWN RISK !!!***

## Usage

```bash
> u8pls help
Usage: u8pls [OPTIONS] <DIR> [MAX_DEPTH] [MAX_CONCURRENCY] <COMMAND>

Commands:
  suffix  match file suffix like: ".txt"
  prefix  match file prefix eg: "data_" will match: data_1.docx,data_2.bin
  regexp  advance match pattern eg: `202\d_[a-z]{3}\.txt` match `2023_abc.txt`
  help    Print this message or the help of the given subcommand(s)

Arguments:
  <DIR>              target dir
  [MAX_DEPTH]        max sub dir depth
  [MAX_CONCURRENCY]  max concurrency, increase if you change system `max_open_file` property [default: 32]

Options:
  -r             recursive on sub dir
  -h, --help     Print help
  -V, --version  Print version

```

