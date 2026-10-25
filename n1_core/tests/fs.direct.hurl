PUT http://localhost:8000/:user.login/
{"name":"eve","password":"56"}
HTTP 200
[Captures]
cookie: cookie "auth"

PUT http://localhost:8000/:fs.direct.mkdir/
Cookie: auth={{cookie}}
{"owner":101,"path":"/dir/subdir1/"}
HTTP 200
[Captures]
dirID: body

PUT http://localhost:8000/:fs.direct.mkdir/
Cookie: auth={{cookie}}
{"owner":101,"path":"/dir/subdir1/"}
HTTP 200
[Asserts]
body == {{dirID}}

PUT http://localhost:8000/:fs.direct.mkfile/
Cookie: auth={{cookie}}
{"owner":101,"path":"/dir/sub/file.txt"}
HTTP 200
[Captures]
fileID: body
