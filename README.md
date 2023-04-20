## abun.ch

This is a tool to compile a list of multiple links, called a bunch, under a short URL with features like an optional description, password, link titles, automatic expiry and more.

It is very much still under development.

### Motivation
While this might not be what the world has asked for, I did not find a similar project anywhere and concluded it could make for a fun little project which I could notoriously neglect to my liking. I also took a look at AlpineJS and wanted to try that out. Last but not least, I liked the challenge of implementing JWT based authentication myself.


### Techstack
The frontend is built using Bootstrap 5.0, AlpineJS and a library called AlpineJS-Router. It is a Single Page Application, not because it makes sense, but because I wanted this to be as low budget as possible. This means that the SPA is hosted by Github Pages, while I employ a free Cloudflare Proxy to rewrite `HTTP GET` requests like `/<bunch_url>` or `/new` to `/index.html`. 

The backend is written in Rust using a library called Rocket. It is connected to a PostgreSQL database, both of which run on a Raspberry Pi 3B+ (see what I mean by low budget)

### FAQ

#### Will there be ...?
There likely won't be anything in the near future except for the very core functionality, since I can't seem to get around to code more than a few lines every now and then. Don't expect anything.

#### Is there documentation?
Not yet and there's no reason to believe there ever will be.

#### Did you build this the proper way?
Absolutely not. There are little to no code comments, the coding style is ugly as hell, there's no concise variable naming and no proper tooling involved. I am pretty sure I am not correctly using Bootstrap given all the CSS I wrote manually and I am also not sure whether AlpineJS is meant to be used this way. The project is basically a compilation of shortcuts.

#### Can I get an account?
I'll start by giving access to friends to try if this works properly. Until then, not really, but if you really really want to, you can reach out and ask anyways.

#### What if I discover any security vulnerabilities?
Please tell me and do not try to exploit them pretty please.

#### Do you think anyone will read this?
Not really but it's fun to come up with bogus question and interview myself.


