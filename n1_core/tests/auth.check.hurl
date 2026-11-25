# Login

PUT http://localhost:8000/:auth.login/
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

PUT http://localhost:8000/:auth.login/
{"name":"sarah","password":"21"}
HTTP 200
[Captures]
cookieSarah: cookie "auth"
``

# Get ID

PUT http://localhost:8000/:auth.get.id/
Cookie: auth={{cookieEve}}
HTTP 200
Content-Type: application/json
`101`

PUT http://localhost:8000/:auth.get.id/
HTTP 200
Content-Type: application/json
`0`

# Is connected

PUT http://localhost:8000/:auth.check.auth/
Cookie: auth={{cookieEve}}
HTTP 200
`null`

PUT http://localhost:8000/:auth.check.auth/
Cookie: auth={{cookieSarah}}
HTTP 200
`null`

PUT http://localhost:8000/:auth.check.auth/
HTTP 401


# IsAdmin

PUT http://localhost:8000/:auth.check.isadmin/
Cookie: auth={{cookieEve}}
HTTP 200
`null`

PUT http://localhost:8000/:auth.check.isadmin/
Cookie: auth={{cookieSarah}}
HTTP 403

PUT http://localhost:8000/:auth.check.isadmin/
HTTP 401


# Access read

PUT http://localhost:8000/:auth.check.access.read/
Cookie: auth={{cookieEve}}
{"owner_id":201,"app_id":1}
HTTP 200
`null`

PUT http://localhost:8000/:auth.check.access.read/
Cookie: auth={{cookieSarah}}
{"owner_id":201,"app_id":1}
HTTP 200
`null`

PUT http://localhost:8000/:auth.check.access.read/
{"owner_id":201,"app_id":1}
HTTP 401


# Access write

PUT http://localhost:8000/:auth.check.access.write/
Cookie: auth={{cookieEve}}
{"owner_id":201,"app_id":1}
HTTP 200
`null`

PUT http://localhost:8000/:auth.check.access.write/
Cookie: auth={{cookieSarah}}
{"owner_id":201,"app_id":1}
HTTP 403

PUT http://localhost:8000/:auth.check.access.write/
{"owner_id":201,"app_id":1}
HTTP 401
