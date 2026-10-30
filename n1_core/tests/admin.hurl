PUT http://localhost:8000/:auth.login/
{"name":"eve","password":"56"}
HTTP 200
[Captures]
cookie: cookie "auth"
``

PUT http://localhost:8000/:auth.admin.new/
Cookie: auth={{cookie}}
"syndicate"
HTTP 200
[Captures]
adminID: body

PUT http://localhost:8000/:auth.admin.edit.name/
Cookie: auth={{cookie}}
{"id":{{adminID}},"name":"new-syndicate"}
HTTP 200
`null`