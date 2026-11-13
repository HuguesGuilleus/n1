PUT http://localhost:8000/:auth.login/
{"name":"eve","password":"56"}
HTTP 200
[Captures]
cookie: cookie "auth"
``

PUT http://localhost:8000/:home.set/
Cookie: auth={{cookie}}
{"title": "TITLE", "desc": "DESC", "content": "CONTENT"}
HTTP 200
`null`

PUT http://localhost:8000/:home.get/
HTTP 200
{"title":"TITLE","desc":"DESC","content":"CONTENT"}

GET http://localhost:8000/
HTTP 200
Content-Type: text/html
[Asserts]
xpath "string(/html/head/title)" == "TITLE"
xpath "string(//h1)" == "TITLE"
xpath "string(/html/head/meta[@name='description']/@content)" == "DESC"
xpath "string(/html/body)" contains "CONTENT"