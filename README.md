# PostWoman
A CLI api tester and request builder, totally not born out of frustration from some other tool...

## Why
I'd much rather edit my test routes in my text editor as bare config files and fire them via a CLI than fumble around some GUI application.

# Usage
`postwoman` expects a `postwoman.toml` collection in your cwd. A different file or path can be specified with the global `-c` option.

Use `postwoman run <filter>` to send requests to all routes in current config matching given filter (regex). Use `.` as filter to run all.

## Examples
A collection can be super simple

```toml
[route.test]
url = "https://api.alemi.dev/debug"
```

But more complex options are available

```toml
[client]
user_agent = "api-tester@alemi.dev"

[route.debug]
url = "https://api.alemi.dev/debug"
method = "PUT"
headers = ["Content-Type: application/json"]
body = { hello = "world!", success = true }
extract = { type = "body" }

[route.cookie]
url = "https://api.alemi.dev/getcookie"
method = "GET"
headers = [
	"Authorization: Basic ...",
	"Accept: application/json"
]
extract = { type = "header", key = "Set-Cookie" }
```
