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
cookieEve: cookie "auth"
``

PUT http://localhost:8000/:login/
{"name":"bob","password":"42"}
HTTP 200
[Captures]
cookieBob: cookie "auth"
``

# ID

PUT http://localhost:8000/:auth.test.id/
Cookie: auth={{cookieEve}}
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
Cookie: auth={{cookieEve}}
HTTP 200
Content-Type: application/json
`true`


# Access without authentication

PUT http://localhost:8000/:auth.test.access.read/
{"owner_id":201,"app_id":5}
HTTP 200
Content-Type: application/json
`false`

PUT http://localhost:8000/:auth.test.access.write/
{"owner_id":201,"app_id":5}
HTTP 200
Content-Type: application/json
`false`


# Admin all access

PUT http://localhost:8000/:auth.test.access.read/
Cookie: auth={{cookieEve}}
{"owner_id":200,"app_id":1}
HTTP 200
Content-Type: application/json
`true`

PUT http://localhost:8000/:auth.test.access.write/
Cookie: auth={{cookieEve}}
{"owner_id":200,"app_id":1}
HTTP 200
Content-Type: application/json
`true`


# Simple user access

PUT http://localhost:8000/:auth.test.access.read/
Cookie: auth={{cookieBob}}
{"owner_id":201,"app_id":1}
HTTP 200
Content-Type: application/json
`true`

PUT http://localhost:8000/:auth.test.access.write/
Cookie: auth={{cookieBob}}
{"owner_id":200,"app_id":1}
HTTP 200
Content-Type: application/json
`false`
