# abun.ch

### Schema

bunch: id, title, description, date, expiration, clickcounter, uri, **creator**, password, fetchOpenGraph, incognito

entries: id, url, clickcounter, description, title, **bunch**

creator: id, password, username, admin

### API Calls

`GET /bunch_uri`

`POST /bunch_uri/clicked` - entry_id

`POST /new` - {title, description, expiration, password, fetchopengraph, incognito, uri, [{url, title, description}, ...]}


#### Later
`POST /login` 

`PATCH /bunch_uri/`

`GET /stat-panel`

...