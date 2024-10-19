# PostWoman
A CLI api tester and request builder, totally not born out of frustration from some other tool...

## Why
I'd much rather edit my test routes in my text editor as bare config files and fire them via a CLI than fumble around some GUI application.

As an example, most API test tools have features to automatically split down query arguments from a built url. PostWoman doesn't bother, as your editor will most likely offer multi-cursors and search/replace to easily split off query parameters.

While PostWoman will never be as fully featured as other graphical tools, it doesn't need to be to provide a solid API testing framework.

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
[client] # HTTP client configuration
user_agent = "postwoman@sample/0.2.0"

[env] # these will be replaced in routes options. environment vars overrule these
PW_TOKEN = "set-me-as-and-environment-variable!"



[route.healthcheck] # the simplest possible route: just name and url
url = "https://api.alemi.dev/"

[route.debug]
url = "https://api.alemi.dev/debug"
method = "PUT" # specify request method
query = [ # specify query parameters in a more friendly way
	"body=json",
	"cache=0"
]
headers = [ # add custom headers to request
	"Content-Type: application/json",
	"Authorization: Bearer ${PW_TOKEN}",
]
body = { hello = "world!", success = true } # body can be a bare string, or an inline table (will be converted to json)
extract = ".path" # extract from json responses with JQ syntax
# note that a bare extractor string is equivalent to `{ type = "jq", query = ".path" }`

[route.benchmark]
url = "https://api.alemi.dev/look/into/the/void"
extract = { type = "discard" } # if you don't care about the output, discard it!

[route.notfound]
url = "https://api.alemi.dev/not-found"
expect = 404 # it's possible to specify expected status code, will fail if doesn't match
extract = { type = "regex", pattern = 'nginx/[0-9\.]+' } # extract from response with regex

[route.payload]
url = "https://api.alemi.dev/debug"
method = "POST"
body = '''{
	"complex": {
		"json": "payloads",
		"can": "be",
		"expressed": "this",
		"way": true
	}
}'''
extract = { type = "body" } # get the whole response body, this is the default extractor

[route.cookie]
url = "https://api.alemi.dev/getcookie"
method = "GET"
extract = { type = "header", key = "Set-Cookie" } # get a specific response header, ignoring body
```

### Running
Show collection summary
```
$ postwoman
> postwoman@sample/0.2.0
+ PW_TOKEN: set-me-as-and-environment-variable!

- healthcheck: 	GET 	https://api.alemi.dev/
- debug: 	PUT 	https://api.alemi.dev/debug
- benchmark: 	GET 	https://api.alemi.dev/look/into/the/void
- notfound: 	GET 	https://api.alemi.dev/not-found
- payload: 	POST 	https://api.alemi.dev/debug
- cookie: 	GET 	https://api.alemi.dev/getcookie
```

Run all endpoints matching `.` (aka all of them)
```
$ postwoman run .
 : [22:27:17.960461] sending healthcheck ...
 + [22:27:18.109843] healthcheck done in 149ms
{
  "example": [
    "https://api.alemi.dev/debug",
    "https://api.alemi.dev/msg",
    "https://api.alemi.dev/mumble/ping"
  ],
  "time": "Saturday, 19-Oct-2024 20:27:18 GMT",
  "up": true
}
 : [22:27:18.109924] sending debug ...
 + [22:27:18.268383] debug done in 158ms
/debug?body=json&cache=0
 : [22:27:18.268477] sending benchmark ...
 + [22:27:18.422707] benchmark done in 154ms
 : [22:27:18.422775] sending notfound ...
 + [22:27:18.575942] notfound done in 153ms
nginx/1.26.2
 : [22:27:18.576010] sending payload ...
 + [22:27:18.732582] payload done in 156ms
{
  "body": "{\n\t\"complex\": {\n\t\t\"json\": \"payloads\",\n\t\t\"can\": \"be\",\n\t\t\"expressed\": \"this\",\n\t\t\"way\": true\n\t}\n}",
  "headers": {
    "accept": [
      "*/*"
    ],
    "connection": "close",
    "content-length": "94",
    "user-agent": "postwoman@sample/0.2.0",
    "x-forwarded-proto": "https",
    "x-real-ip": "93.34.149.115",
    "x-real-port": 46695,
    "x-user-agent": "postwoman@sample/0.2.0"
  },
  "method": "POST",
  "path": "/debug",
  "time": 1729369638.7079802,
  "version": "HTTP/1.0"
}
 : [22:27:18.732676] sending cookie ...
 + [22:27:18.886862] cookie done in 154ms
SGF2ZSBhIENvb2tpZSE=
```

Debug a specific route passing `--debug`:
```
$ postwoman run notfound --debug
 : [22:26:59.045642] sending notfound ...
 + [22:26:59.220103] notfound done in 174ms
Response {
    url: "https://api.alemi.dev/not-found",
    status: 404,
    headers: {
        "server": "nginx/1.26.2",
        "date": "Sat, 19 Oct 2024 20:26:59 GMT",
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
