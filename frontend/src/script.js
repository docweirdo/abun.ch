document.addEventListener('alpine:init', async () => {
    
    init_stores(); // init here because within effect triggers loop

    Alpine.effect(() => {
        path = Alpine.$router.path;
        console.log('router_effect')
        path_change()
    })
})

// Global State
let headers;
let failedTries;
let backend = "https://api.abun.ch"

function init_stores(){
    Alpine.store('passwordWall', {value: false}) // object because of fucked up simple stores

    Alpine.store('bunch', new Bunch()) 
    Alpine.store('selected', new Map())
}

function path_change(){
    router = Alpine.$router;
    headers = new Headers();
    failedTries = 0;

    if (router.is('/:uri([a-zA-Z0-9_-]{6})')) {
        Alpine.store('state', 1)
        set_uri(router.path.slice(1));
    } else if (router.is('/new')){
        Alpine.store('state', 2)
        set_new();
    }else {
        Alpine.store('state', 0)
        set_start();
    }
}

function set_new(){    
    Alpine.store('links', {value: []})

    let cookies = document.cookie.split(";");;
    let found = false;

    for (c of cookies){
        if (c.split("=")[0] == 'logged_in') {
            found = true;
            break;
        }
    }

    if (found) return;

    Alpine.store('passwordWall').value = true;
}

async function set_uri(uri){
    password = localStorage.getItem(uri);
    let bunch;

    try {
        if (password){
            headers.set('Authorization', password);
        }
        bunch = await fetchBunch(uri);
    } catch (error) {
        failedTries +=1

        Alpine.store('passwordWall').value = true;
        localStorage.removeItem(uri)
        
        if (failedTries == 2){
            document.getElementById('otp-password').classList.add('is-invalid')
        }
        return;
    }
    Alpine.store('passwordWall').value = false;
    document.getElementById('otp-password').classList.remove('is-invalid');
    failedTries = 0;

    Alpine.store('links', {value: bunch.entries.map(e => Object.assign(new Link, e))})
    Alpine.store('bunch', bunch)
}

function set_start(){
    // TODO
}

class Bunch {
    title;
    description;
    username;
    open_graph;

    constructor(){
        this.title = ''
        this.description = ''
        this.open_graph = false,
        this.date = ''
    }

    set date (a){
        let d = new Date(a[0], 0, a[1])
        this.dateVal = d;
    }
    get date (){
        return this.dateVal.toLocaleDateString('de-DE', {dateStyle: "medium"})
    }
}

class Link {
    description;
    url;
    id;

    constructor(id, url, t, description){
        this.titleVal = t;
        this.url = url;
        this.description = description;
        this.id = id;
    }

    get title (){
        return this.titleVal ? this.titleVal : this.url
    }
    set title (t){
        this.titleVal = t;
    }
} 

const fetchBunch = async (uri) => {
    let response = await fetch(`${backend}/${uri}`, {headers, credentials: 'include'})
    
    if (response.status === 401){
        return Promise.reject('unauthorized');
    }
    
    let bunch = await response.json()
    bunch = Object.assign(new Bunch, bunch);

    return bunch;
}



const selectAll = e => {

    for (const li of e.parentNode.parentNode.querySelectorAll("input")) {
        li.checked = e.checked;
    }

    let map = Alpine.store('selected');

    let links = Alpine.store('links').value

    if (e.checked) {
        for (const link of links) {
            map.set(link.id, link);
        }
    } else {
        map = map.clear();
    }

    Alpine.store('selected', map);

};

const openLinks = () => {

    for (const [id, link] of Alpine.store('selected')) {
        sendClicked(link)
    }

}

const createNewBunch = async () => {

    let anonymous_box = document.getElementById("checkbox-anonymous");
    let password_input = document.getElementById("new-bunch-password");

    let bunch = Alpine.store('bunch');
    bunch.title = bunch.title.trim();
    bunch.description = bunch.description.trim();
    bunch.incognito = anonymous_box.checked;
    bunch.open_graph = false;
    bunch.password = password_input.value.length > 0 ? password_input.value : null;

    let new_links = Alpine.store('links').value;

    // TODO: visual feedback for validation 

    if (bunch.title.length == 0) return;
    if (bunch.password && bunch.password.length > 20) return; //TODO: refactoring to utf8 or visible chars
    if (new_links.length < 1) return;

    new_links = new_links.map((link) => {
        delete link.id;
        return link;
    });

    console.log(JSON.stringify(bunch), JSON.stringify(new_links))

    bunch.entries = new_links;

    let newBunchResponse = await fetch(`${backend}/new`, {method: 'POST', headers, credentials: "include", body: JSON.stringify(bunch)});

    if (newBunchResponse.ok){
        newBunchUrl = await newBunchResponse.json();
        window.location.pathname = newBunchUrl;
    }
    
}

const sendClicked = (entry) => {
    fetch(`${backend}/${Alpine.store('path')}/clicked/${entry.id}`, {method: 'POST', headers})
    window.open(entry.url, '_blank')
}

const usePassword = p => {
    let uri = Alpine.$router.path.slice(1);

    localStorage.setItem(uri, p)
    set_uri(uri)
}

const login = async (u, p) => {
    let response = await fetch(`${backend}/login`, { method: 'POST', body: JSON.stringify({password: p, username: u}), credentials: "include"})
    if (response.ok){
        Alpine.store('passwordWall').value = false;
    }
}

const logout = async () => {
    let response = await fetch(`${backend}/logout`, { method: 'POST', credentials: "include"});
    if (response.ok){
        window.location.pathname = "";
    }
}

let newLinkID = 0;

const parseNewEntry = (newEntry) => {
    // parse entry
    newEntry = newEntry.trim();
    let index = newEntry.search(/\s+/)

    index = index == -1 ? newEntry.length+1 : index
    
    let url = newEntry.slice(0, index);

    if (url.length < 1) return;

    let title;

    if (index <= newEntry.length){
        title = newEntry.slice(index).trim();
    }

    let parsedEntry = new Link(newLinkID, window.normalizeUrl(url), title) // TODO: description
    newLinkID = newLinkID + 1;

    let links = Alpine.store('links').value;
    links.push(parsedEntry)
}

const deleteNewEntry = (entry) => {

    let links = Alpine.store('links').value;

    filtered_links = links.filter((link) => link != entry)

    Alpine.store('links').value = filtered_links;
    
}