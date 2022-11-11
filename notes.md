# abun.ch

### Schema

bunch: id, title, description, date, expiration, clickcounter, uri, **creator**, password, open_graph, incognito

entries: id, url, clickcounter, description, title, **bunch**

creator: id, password, username, admin

### API Calls

`GET /bunch_uri` - {title, (description), date, (creator.username), open_graph, [id, url, title, description]}

`POST /bunch_uri/clicked` - entry_id

`POST /new` - {title, description, expiration, password, fetchopengraph, incognito, uri, [{url, title, description}, ...]}


#### Later
`POST /login` 

`PATCH /bunch_uri/`

`GET /stat-panel`

...