PUT http://localhost:8000/:auth.login/
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

PUT http://localhost:8000/:fs.direct.resolve/
Cookie: auth={{cookie}}
{"owner":101,"path":"/dir/"}
HTTP 200
[Captures]
dirRootDir: body

PUT http://localhost:8000/:fs.direct.tree/
Cookie: auth={{cookie}}
101
HTTP 200
[Asserts]
jsonpath "$.id" == 0
jsonpath "$.is_dir" == true
jsonpath "$.content" == 0
jsonpath "$.children.dir.id" toString == {{dirRootDir}}
jsonpath "$.children.dir.is_dir" == true
jsonpath "$.children.dir.content" == 0
jsonpath "$.children.dir.children.sub.id" != 0
jsonpath "$.children.dir.children.sub.is_dir" == true
jsonpath "$.children.dir.children.sub.content" == 0
jsonpath "$.children.dir.children.sub.children['file.txt'].id" toString == {{fileID}}
jsonpath "$.children.dir.children.sub.children['file.txt'].is_dir" == false
jsonpath "$.children.dir.children.sub.children['file.txt'].content" == 266668
jsonpath "$.children.dir.children.subdir1.id" toString == {{dirID}}
jsonpath "$.children.dir.children.subdir1.is_dir" == true
jsonpath "$.children.dir.children.subdir1.content" == 0

PUT http://localhost:8000/:fs.direct.rm/
Cookie: auth={{cookie}}
{"owner":101,"object":{{dirRootDir}}}
HTTP 200
`null`

PUT http://localhost:8000/;fs.direct.get/101,{{dirRootDir}}
Cookie: auth={{cookie}}
HTTP 404
