Simple Zendesk ticket exporter.

I wrote it as "hello world" Rust project.

**Usage:**

```
./zendesk_export [OPTIONS]

OPTIONS:

-c, --config <FILE>    Sets a custom config file [default: config.toml]
-h, --help             Print help information
-t, --tickets          Do fetch tickets
-u, --users            Do fetch users
-V, --version          Print version information
```

You have to specify `--tickets` or `--users` to fetch tickets
or users to `tickets.json` and `users.json` respectively.

**Config:**

Read [Zendesk documentation](https://support.zendesk.com/hc/en-us/articles/4408889192858-Generating-a-new-API-token)
for more information about API tokens. 

`config.toml.dist`:
```toml
# your zendesk domain
domain=''
# login and password for zendesk
# by default login/password authentication is disabled, and you have to generate API token
# see: https://support.zendesk.com/hc/en-us/articles/4408889192858-Generating-a-new-API-token
# in such case login will be "your@email.com/token" and password will be "your_api_token"
login=''
password=''
```