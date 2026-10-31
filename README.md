# JQyr0

A small file bundle and update client. `init` writes `JQyr0.toml` in the current directory. Its `[default]` section stores the destination root, server URL, endpoints, ordinary file paths, and base64 encoded embedded raw files. By default, `root = "."`, so `recreate` writes under the recipient's current working directory. Absolute sender paths are converted to bundle-relative paths: the final component of the common parent is retained as the top-level directory. For example, `/home/A/mydir/f1.py` and `/home/A/mydir/a/f2.py` become `mydir/f1.py` and `mydir/a/f2.py`.

```toml
[default]
root = "/path/to/output"
src = "https://files.example.test"
pull_endpoint = "/pull"
check_update_endpoint = "/check/{checksum}"
files = ["assets/theme.css"]

[[default.raw_files]]
path = "settings.bin"
content = "AAECAw==" # base64 encoded bytes
```

## Commands

- `jqyr0 init --working-dir=PATH --source=URL [--pull-endpoint=/pull] [--check-update-endpoint=/check/{checksum}] [--file=PATH] [--raw-file=PATH]`
- `jqyr0 check [--config=PATH]` validates the config and sends a GET to the check endpoint with `{checksum}` replaced by the SHA-256 of configured local files.
- `jqyr0 recreate [--config=PATH]` writes embedded files and POSTs the configured ordinary file paths as a JSON array to the pull endpoint.
- `jqyr0 packer [--config=PATH] --file=PATH` adds an ordinary file path; `--raw-file=PATH` embeds the file's bytes in the TOML.

The pull endpoint responds with JSON records shaped like `[{"path":"mydir/f1.py","content":"<base64 bytes>"}]`. The client rejects unexpected or duplicate paths and writes returned files beneath `root`.
