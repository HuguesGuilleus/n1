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

# Get page
GET http://localhost:8000/@eve/
HTTP 200

GET http://localhost:8000/@eve
HTTP 302
Location: /@eve/
