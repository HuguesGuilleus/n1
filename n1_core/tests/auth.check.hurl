# Login

PUT http://localhost:8000/:login/
{"name":"eve","password":"56"}
HTTP 200
[Captures]
cookieEve: cookie "auth"
``

PUT http://localhost:8000/:login/
{"name":"bob","password":"42"}
HTTP 200
[Captures]
cookieBob: cookie "auth"
``

# Is connected

PUT http://localhost:8000/:auth.check.auth/
Cookie: auth={{cookieEve}}
HTTP 200
``

PUT http://localhost:8000/:auth.check.auth/
Cookie: auth={{cookieBob}}
HTTP 200
``

PUT http://localhost:8000/:auth.check.auth/
HTTP 401


# IsAdmin

PUT http://localhost:8000/:auth.check.isadmin/
Cookie: auth={{cookieEve}}
HTTP 200
``

PUT http://localhost:8000/:auth.check.isadmin/
Cookie: auth={{cookieBob}}
HTTP 403

PUT http://localhost:8000/:auth.check.isadmin/
HTTP 401


# Access read

PUT http://localhost:8000/:auth.check.access.read/
Cookie: auth={{cookieEve}}
{"owner_id":201,"app_id":1}
HTTP 200
``

PUT http://localhost:8000/:auth.check.access.read/
Cookie: auth={{cookieBob}}
{"owner_id":201,"app_id":1}
HTTP 200
``

PUT http://localhost:8000/:auth.check.access.read/
{"owner_id":201,"app_id":1}
HTTP 401


# Access write

PUT http://localhost:8000/:auth.check.access.write/
Cookie: auth={{cookieEve}}
{"owner_id":201,"app_id":1}
HTTP 200
``

PUT http://localhost:8000/:auth.check.access.write/
Cookie: auth={{cookieBob}}
{"owner_id":201,"app_id":1}
HTTP 403

PUT http://localhost:8000/:auth.check.access.write/
{"owner_id":201,"app_id":1}
HTTP 401
