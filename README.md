# PostWoman
A CLI api tester and request builder, compatible with Postman collection format

## Why
Since newer postman requires registration to use most useful features, I decided to build my own tool reusing their format, to be compatible with my coworkers' collections.
Also, I'd much rather use a CLI tool than a bundled webpage.

# Usage
Add `alias pw=postwoman` to your `.bashrc` because I'll be referring to it as `pw` from now on.

`pw` expects a `postwoman.json` collection in your cwd. A different file or path can be specified with the global `-c` option.

`pw` supports 2 actions as of now:

* `show` which will display on your CLI the collection structure
* `test` which will execute all requests concurrently

Both actions support a `-v` switch to print more stuff (body, headers, descriptions...) and a `-p` switch to prettify json outputs.

Both actions also work with a filter: just type your regex as argument and only requests with matching urls will be displayed/executed

# Examples
All coming examples are run with provided example `postwoman.json` in their cwd.

### Show
```
$ pw show
─┐ Sample Postman Collection
 ├ * GET https://api.alemi.dev/dump?source=sample-collection
 ├─┐ POST requests
 │ ├ * POST https://api.alemi.dev/dump
 │ ├ * POST https://api.alemi.dev/dump
 │ ╵
 ╵
```

```
$ pw show -v
─┐ Sample Postman Collection
 │   A sample collection to demonstrate collections as a set of related requests
 │
 ├ * GET https://api.alemi.dev/dump?source=sample-collection
 │
 ├─┐ POST requests
 │ │
 │ ├ * POST https://api.alemi.dev/dump
 │ │   [ content-type:text/plain ]
 │ │   Duis posuere augue vel cursus pharetra. In luctus a ex nec pretium...
 │ │
 │ ├ * POST https://api.alemi.dev/dump
 │ │   [ content-type:application/json ]
 │ │   {"length":100,"text":"Lorem ipsum"}
 │ │
 │ ╵
 ╵
```

```
$ pw show -v -p
─┐ Sample Postman Collection
 │   A sample collection to demonstrate collections as a set of related requests
 │
 ├ * GET https://api.alemi.dev/dump?source=sample-collection
 │
 ├─┐ POST requests
 │ │
 │ ├ * POST https://api.alemi.dev/dump
 │ │   [
 │ │     content-type:text/plain
 │ │   ]
 │ │   Duis posuere augue vel cursus pharetra. In luctus a ex nec pretium...
 │ │
 │ ├ * POST https://api.alemi.dev/dump
 │ │   [
 │ │     content-type:application/json
 │ │   ]
 │ │   {
 │ │     "length": 100,
 │ │     "text": "Lorem ipsum"
 │ │   }
 │ │
 │ ╵
 ╵
```

### Test

```
$ pw test
─┐ Sample Postman Collection
 ├ ✓ 200 >> GET https://api.alemi.dev/dump?source=sample-collection
 ├─┐ POST requests
 │ ├ ✓ 200 >> POST https://api.alemi.dev/dump
 │ ├ ✓ 200 >> POST https://api.alemi.dev/dump
 │ ╵
 ╵
```

```
$ pw test -v -p
─┐ Sample Postman Collection
 │   A sample collection to demonstrate collections as a set of related requests
 │
 ├ ✓ 200 >> GET https://api.alemi.dev/dump?source=sample-collection
 │   {
 │     "body": "",
 │     "headers": {
 │       "accept": [
 │         "*/*"
 │       ],
 │       "connection": "close",
 │       "user-agent": "postwoman/0.2.0",
 │       "x-forwarded-proto": "https",
 │       "x-real-ip": "xxx.xxx.xxx.xxx",
 │       "x-real-port": xxxxx
 │     },
 │     "method": "GET",
 │     "path": "/dump?source=sample-collection",
 │     "time": 0.2629528,
 │     "version": "HTTP/1.0"
 │   }
 │
 ├─┐ POST requests
 │ │
 │ ├ ✓ 200 >> POST https://api.alemi.dev/dump
 │ │   {
 │ │     "body": "Duis posuere augue vel cursus pharetra. In luctus a ex nec pretium...",
 │ │     "headers": {
 │ │       "accept": [
 │ │         "*/*"
 │ │       ],
 │ │       "connection": "close",
 │ │       "content-length": "69",
 │ │       "content-type": "text/plain",
 │ │       "user-agent": "postwoman/0.2.0",
 │ │       "x-forwarded-proto": "https",
 │ │       "x-real-ip": "xxx.xxx.xxx.xxx",
 │ │       "x-real-port": xxxxx
 │ │     },
 │ │     "method": "POST",
 │ │     "path": "/dump",
 │ │     "time": 0.2708838,
 │ │     "version": "HTTP/1.0"
 │ │   }
 │ │
 │ ├ ✓ 200 >> POST https://api.alemi.dev/dump
 │ │   {
 │ │     "body": "{\"text\":\"Lorem ipsum\", \"length\":100}",
 │ │     "headers": {
 │ │       "accept": [
 │ │         "*/*"
 │ │       ],
 │ │       "connection": "close",
 │ │       "content-length": "36",
 │ │       "content-type": "application/json",
 │ │       "user-agent": "postwoman/0.2.0",
 │ │       "x-forwarded-proto": "https",
 │ │       "x-real-ip": "xxx.xxx.xxx.xxx",
 │ │       "x-real-port": xxxxx
 │ │     },
 │ │     "method": "POST",
 │ │     "path": "/dump",
 │ │     "time": 0.2888672,
 │ │     "version": "HTTP/1.0"
 │ │   }
 │ │
 │ ╵
 ╵
```
