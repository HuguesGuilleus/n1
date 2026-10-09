PUT http://localhost:8000/:login/
{"name":"eve","password":"56"}
HTTP 200
[Captures]
cookie: cookie "auth"

# New page
PUT http://localhost:8000/:bio.set/
Cookie: auth={{cookie}}
{"oid":101,"content":"https://github.blog/ My blog"}
HTTP 200
`null`

# Get page
PUT http://localhost:8000/:bio.get/
101
HTTP 200
{"content":"https://github.blog/ My blog","last_edit":1780998183}

# Get page
GET http://localhost:8000/@eve/
HTTP 200
[Asserts]
xpath "string(/html/head/title)" == "@eve"
xpath "string(/html/body/main/a)" == "My blog"
xpath "string(/html/body/main/a/@href)" == "https://github.blog/"
