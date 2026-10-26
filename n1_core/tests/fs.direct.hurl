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

GET http://localhost:8000/;fs.direct.get/101,{{fileID}}
Cookie: auth={{cookie}}
HTTP 200
``

PUT http://localhost:8000/;fs.direct.set/101,{{fileID}}
Cookie: auth={{cookie}}
file,data.txt;
HTTP 200
``

GET http://localhost:8000/;fs.direct.get/101,{{fileID}}
Cookie: auth={{cookie}}
HTTP 200
file,data.txt;

PUT http://localhost:8000/:fs.direct.rm/
Cookie: auth={{cookie}}
{"owner":101,"object":{{dirID}}}
HTTP 200
`null`
