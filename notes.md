# abun.ch

### Schema

bunch: id, title, description, date, expiration, clickcounter, uri, **creator**, password, open_graph, incognito

entries: id, url, clickcounter, description, title, **bunch**

creator: id, password, username, admin

### API Calls

`GET /bunch_uri` - {title, (description), date, (creator.username), open_graph, [id, url, title, description]}

`POST /bunch_uri/clicked` - entry_id

`POST /new` - {title, description, expiration, password, fetchopengraph, incognito, [{url, title, description}, ...]}


#### Later
`POST /login` 

`PATCH /bunch_uri/`

`GET /stat-panel`

...

## Unobvious to do:
* Check expiration date before serving bunch (and delete)
* worry about transactions
* validate url in front and backend
* add placeholder with bootstrap
* consider api caching
* check localStorage for every uri before first 401 response
* make meta robots tag conditional on route
* set opengraph info (difficult without js)
* enforce string lenghts in frontend
* implement account creation via token
* provide visual feedback for failed login
* worry about crsf
* validate URLs to be actuals links
* refactor so stringlength is enforced for either all utf-8 or visible characters