PUT http://localhost:8000/:login/
{"name":"eve","password":"56"}
HTTP 200
[Captures]
cookie: cookie "auth"

# New page
PUT http://localhost:8000/:wiki.article.new/
Cookie: auth={{cookie}}
{"owner_id":201,"title":"Title 1","slug":"slug1"}
HTTP 200
[Captures]
article_id: body

# Get init page
PUT http://localhost:8000/:wiki.article.get/
{"owner_id":201,"article_id":{{article_id}}}
HTTP 200
{"oid":{{article_id}},"slug":"slug1","title":"Title 1","last_edit":1780998183,"content":"..."}

# Set title
PUT http://localhost:8000/:wiki.article.setTitle/
Cookie: auth={{cookie}}
{"owner_id":201,"article_id":{{article_id}},"title":"Title 2"}
HTTP 200
``

# Set slug
PUT http://localhost:8000/:wiki.article.setSlug/
Cookie: auth={{cookie}}
{"owner_id":201,"article_id":{{article_id}},"slug":"slug2"}
HTTP 200
``

# Set content
PUT http://localhost:8000/:wiki.article.setContent/
Cookie: auth={{cookie}}
{"owner_id":201,"article_id":{{article_id}},"content":"Lorem ipsum."}
HTTP 200
``

# Get edited article
PUT http://localhost:8000/:wiki.article.get/
{"owner_id":201,"article_id":{{article_id}}}
HTTP 200
{"oid":{{article_id}},"slug":"slug2","title":"Title 2","last_edit":1780998183,"content":"Lorem ipsum."}

# Check article is removed
PUT http://localhost:8000/:wiki.article.rm/
Cookie: auth={{cookie}}
{"owner_id":201,"article_id":{{article_id}}}
HTTP 200
``

# Check article is removed
PUT http://localhost:8000/:wiki.article.get/
{"owner_id":201,"article_id":{{article_id}}}
HTTP 404