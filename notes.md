# abun.ch

### Schema

bunch: title, description, date, expiration, clickcounter, uri, **creator**, password, fetchOpenGraph

entries: url, clickcounter, description, title, **bunch**

creator: name, password, username, is_admin

### API Calls

`GET /bunch_uri`

`POST /bunch_uri/clicked` - entry_id

`POST /new` - {title, description, expiration, password, fetchopengraph, uri, [{url, title, description}, ...]}


#### Later
`POST /login` 

`PATCH /bunch_uri/`

`GET /stat-panel`

...