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

GET http://localhost:8000/;fs.direct.get/101,{{fileID}}
Cookie: auth={{cookie}}
Range: bytes=3-13
HTTP 206
Content-Length: 11
Content-Range: bytes 3-13/266668
`tUu6NJw8E5/`

GET http://localhost:8000/;fs.direct.get/101,{{fileID}}
Cookie: auth={{cookie}}
Range: bytes=265842-
HTTP 206
Content-Length: 826
Content-Range: bytes 265842-266667/266668
`5FWqQjEEVuNSo2smmn66jK3Ta2vnQ3KXrww3EaKb33mOgbARJIN40umDPpggg9VtLSb5VRwlYKxS86R4fk/65q9VZJc+0A4oIG0xE3ojEh6RYk2wHobgC6GcKlK5iuAD11byglK/fEivNq7CN0lGRtPbK5PWO8oS17bYQavBThHVs8Yo/133J2FRuF6wXjBfpc5HesJbT+8ZmLZa6iA4ErjKrMYAFJQZwDIJ47IkEZGV6IjbYqkoIPpeIKHAEuqg0lUHRwqlgTbe4Wt1G6/aEUcqM+IlobZzo7Cx7RoV7OzjJ+Rlx0bX7zFn0LbJ3o32RN1KB+2gNiGug43euK92E74kRUTyFQ0qvpJHB8p2KRGTfqaEFmHLU8IZFXbLq7APM7P+ygFhY/RcN43w5GhqBNhEVlWIat52P3TtCIvuHA8ZX1fpyIiUBQoBWkUClWF4VOLCQCRaiYuwYzmlLySZZDQN1T4sF715slVV1ROWpdgx885XJ4bSlrWkXIYH5i2Bq7tFClnezAYOT6lnyFxHc9O4fA0RYLmc7nWRdyc21gu/6N2tYZw9KnOR6/+L93mNHILwBNrnZSkExiWPBLV/8sysRn8tQlMgImibv4fjkruaUx7dp44RN6GUjEeUowgvD7k1MbLK23WPmkiS1MzM3DAKxQfsRaeTGpIq0jU3JNPecBNR+I+OtcNhYqh6tHXm2+JkoLqj9OE42N8w6uP2gEENpysO3Aut6tNaKx3yaUxUpar7ApyFkABA0fJQUkGYBBCufAkjLp4ykU6LhY8CsRvyZK4kuuEKCp+29Ls+k/8dgfKFE1LBwjKMU=`

PUT http://localhost:8000/:fs.direct.resolve/
Cookie: auth={{cookie}}
{"owner":101,"path":"/dir/"}
HTTP 200
[Captures]
dirRootDir: body

PUT http://localhost:8000/:fs.direct.move/
Cookie: auth={{cookie}}
{"owner":101,"object":{{fileID}},"path":"/dir/sub/demo.txt"}
HTTP 200
`null`

PUT http://localhost:8000/:fs.direct.tree/
Cookie: auth={{cookie}}
101
HTTP 200
[Asserts]
jsonpath "$.id" == 0
jsonpath "$.is_dir" == true
jsonpath "$.children.dir.id" toString == {{dirRootDir}}
jsonpath "$.children.dir.is_dir" == true
jsonpath "$.children.dir.children.sub.id" != 0
jsonpath "$.children.dir.children.sub.is_dir" == true
jsonpath "$.children.dir.children.sub.children['demo.txt'].id" toString == {{fileID}}
jsonpath "$.children.dir.children.sub.children['demo.txt'].is_dir" == false
jsonpath "$.children.dir.children.sub.children['demo.txt'].content" == 266668
jsonpath "$.children.dir.children.subdir1.id" toString == {{dirID}}
jsonpath "$.children.dir.children.subdir1.is_dir" == true

PUT http://localhost:8000/:fs.direct.rm/
Cookie: auth={{cookie}}
{"owner":101,"object":{{dirRootDir}}}
HTTP 200
`null`

PUT http://localhost:8000/;fs.direct.get/101,{{dirRootDir}}
Cookie: auth={{cookie}}
HTTP 404
