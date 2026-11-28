PUT http://localhost:8000/:auth.login/
{"name":"eve","password":"56"}
HTTP 200
[Captures]
cookie: cookie "auth"
``

PUT http://localhost:8000/:auth.org.new/
Cookie: auth={{cookie}}
"section"
HTTP 200
[Captures]
adminID: body

PUT http://localhost:8000/:auth.org.edit.name/
Cookie: auth={{cookie}}
{"id":{{adminID}},"name":"section-troyes"}
HTTP 200
`null`
