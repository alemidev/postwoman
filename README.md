# PostWoman
[![Actions Status](https://github.com/alemidev/postwoman/actions/workflows/test.yml/badge.svg)](https://github.com/alemidev/postwoman/actions)
[![Crates.io Version](https://img.shields.io/crates/v/postwoman)](https://crates.io/crates/postwoman)
[![Crates.io Downloads (latest version)](https://img.shields.io/crates/dv/postwoman)](https://crates.io/crates/postwoman)
[![GitHub last commit](https://img.shields.io/github/last-commit/alemidev/postwoman)](https://github.com/alemidev/postwoman/commits/dev/)
[![GitHub Issues or Pull Requests](https://img.shields.io/github/issues/alemidev/postwoman)](https://github.com/alemidev/postwoman/issues)

A CLI api tester and request builder, totally not born out of frustration from some other tool...

## Why
I'd much rather edit my test routes in my text editor as bare config files and fire them via a CLI than fumble around some GUI application.

As an example, most API test tools have features to automatically split down query arguments from a built url. PostWoman doesn't bother, as your editor will most likely offer multi-cursors and search/replace to easily split off query parameters.

While PostWoman will never be as fully featured as other graphical tools, it doesn't need to be to provide a solid API testing framework.

# Usage
Install with `cargo install postwoman`

`postwoman` expects a `postwoman.toml` collection in your cwd. A different file or path can be specified with the global `-c` option.

Use `postwoman run <filter>` to send requests to all routes in current config matching given filter (regex). Use `.` as filter to run all.

## Examples
A collection can be super simple

```toml
[route.test]
url = "https://api.alemi.dev/debug"
```

But more complex options are available, check out provided `postwoman.toml` for some (ready to run!) examples.

More complex collection trees can be achieved with `include` top level field.
Includes are idempotent and always resolved relative from parent collection's directory.

```toml
include = [
	"other.toml",
	"even/more.toml"
]
```

### Running
Show collection summary
```
$ postwoman
~@ postwoman/0.3.1
-> postwoman.toml
 + PW_TOKEN=set-me-as-and-environment-variable!
 - healthcheck 	GET 	https://api.alemi.dev/
 - debug 	PUT 	https://api.alemi.dev/debug
 - benchmark 	GET 	https://api.alemi.dev/look/into/the/void
 - notfound 	GET 	https://api.alemi.dev/not-found
 - payload 	POST 	https://api.alemi.dev/debug
 - cookie 	GET 	https://api.alemi.dev/getcookie
```

Run all endpoints matching `.` (aka all of them)
```
$ postwoman run .
~@ postwoman/0.3.1
 : [05:14:47.241122] postwoman.toml::healthcheck 	sending request...
 + [05:14:47.411708] postwoman.toml::healthcheck 	done in 170ms
{
  "example": [
    "https://api.alemi.dev/debug",
    "https://api.alemi.dev/msg",
    "https://api.alemi.dev/mumble/ping"
  ],
  "time": "Sunday, 20-Oct-2024 03:14:47 GMT",
  "up": true
}
 : [05:14:47.411807] postwoman.toml::debug 	sending request...
 + [05:14:47.574391] postwoman.toml::debug 	done in 162ms
/debug?body=json&cache=0
 : [05:14:47.574474] postwoman.toml::benchmark 	sending request...
 + [05:14:47.726527] postwoman.toml::benchmark 	done in 152ms
 : [05:14:47.726605] postwoman.toml::notfound 	sending request...
 + [05:14:47.878922] postwoman.toml::notfound 	done in 152ms
nginx/1.26.2
 : [05:14:47.879000] postwoman.toml::payload 	sending request...
 + [05:14:48.039053] postwoman.toml::payload 	done in 160ms
{
  "body": "{\n\t\"complex\": {\n\t\t\"json\": \"payloads\",\n\t\t\"can\": \"be\",\n\t\t\"expressed\": \"this\",\n\t\t\"way\": true\n\t}\n}",
  "headers": {
    "accept": [
      "*/*"
    ],
    "connection": "close",
    "content-length": "94",
    "user-agent": "postwoman@sample/0.3.1",
    "x-forwarded-proto": "https",
    "x-real-ip": "93.34.149.115",
    "x-real-port": 46945,
    "x-user-agent": "postwoman@sample/0.3.1"
  },
  "method": "POST",
  "path": "/debug",
  "time": 1729394088.0156112,
  "version": "HTTP/1.0"
}
 : [05:14:48.039099] postwoman.toml::cookie 	sending request...
 + [05:14:48.168725] postwoman.toml::cookie 	done in 129ms
SGF2ZSBhIENvb2tpZSE=
```

Debug a specific route passing `--debug`:
```
$ postwoman run notfound --debug
~@ postwoman/0.3.1
 : [05:15:16.960147] postwoman.toml::notfound 	sending request...
 + [05:15:17.120647] postwoman.toml::notfound 	done in 160ms
Response {
    url: "https://api.alemi.dev/not-found",
    status: 404,
    headers: {
        "server": "nginx/1.26.2",
        "date": "Sun, 20 Oct 2024 03:15:17 GMT",
        "content-type": "text/html",
        "content-length": "153",
        "connection": "keep-alive",
        "vary": "Accept-Encoding",
    },
}
Body: <html>
<head><title>404 Not Found</title></head>
<body>
<center><h1>404 Not Found</h1></center>
<hr><center>nginx/1.26.2</center>
</body>
</html>
```
