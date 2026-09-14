PUT http://localhost:8000/:login/
{"name":"eve","password":"56"}
HTTP 200
[Asserts]
cookie "auth" startsWith "T1."
cookie "auth[Max-Age]" == 604800 # 1 week
cookie "auth[Domain]" not exists
cookie "auth[Path]" not exists
cookie "auth[Secure]" exists
cookie "auth[HttpOnly]" exists
cookie "auth[SameSite]" == "Strict"
[Captures]
cookie: cookie "auth"
``

# ID

PUT http://localhost:8000/:auth.test.id/
Cookie: auth={{cookie}}
HTTP 200
Content-Type: application/json
`101`

PUT http://localhost:8000/:auth.test.id/
HTTP 200
Content-Type: application/json
`0`


# IsAdmin

PUT http://localhost:8000/:auth.test.isadmin/
HTTP 200
Content-Type: application/json
`false`

PUT http://localhost:8000/:auth.test.isadmin/
Cookie: auth={{cookie}}
HTTP 200
Content-Type: application/json
`true`


# Access

PUT http://localhost:8000/:auth.test.access/
{"id":201,"app":5,"can_write":true}
HTTP 200
Content-Type: application/json
`false`
