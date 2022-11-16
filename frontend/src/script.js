document.addEventListener('alpine:init', async () => {
    Alpine.store('links', [])
    Alpine.store('passwordWall', false)
    Alpine.store('selected', new Map())
    Alpine.store('bunch', new Bunch())
    Alpine.store('path', '')
})

let headers = new Headers();

class Bunch {
    title;
    description;
    username;
    open_graph;

    constructor(){
        this.title = ''
        this.description = null
        this.username = null
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


    get title (){
        return this.titleVal ? this.titleVal : this.url
    }
    set title (t){
        this.titleVal = t;
    }
}

const determinePath = (router) => {

    Alpine.store('path', router.path.slice(1))
    
    if (router.is('/:uri([a-zA-Z0-9_-]{6})')) {
        fetchBunch()
    } else if (router.is('/new')){
        let cookies = document.cookie.split(";");;
        let found = false;

        for (c of cookies){
            if (c.split("=")[0] == 'logged_in') {
                found = true;
                break;
            }
        }

        if (found) return;

        Alpine.store('passwordWall', true); 
    }
}; 

const fetchBunch = async () => {
    
    let response = await fetch(`/api/${Alpine.store('path')}`, {headers})
    
    if (response.status === 401){
        let p = localStorage.getItem(Alpine.store('path'))
        if (p){
            usePassword(p)
            return;
        }

        Alpine.store('passwordWall', true)
        return;
    }
    Alpine.store('passwordWall', false)

    let bunch = await response.json()

    window.bunch = bunch
    bunch = Object.assign(new Bunch, bunch);
    
    Alpine.store('links', bunch.entries.map(e => Object.assign(new Link, e)))
    Alpine.store('bunch', bunch)
}



const selectAll = e => {

    for (const li of e.parentNode.parentNode.querySelectorAll("input")) {
        li.checked = e.checked;
    }

    let map = Alpine.store('selected');

    if (e.checked) {
        for (const link of Alpine.store('links')) {
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

const sendClicked = (entry) => {
    fetch(`/api/${Alpine.store('path')}/clicked/${entry.id}`, {method: 'POST', headers})
    window.open(entry.url, '_blank')
}

const usePassword = p =>{
    localStorage.setItem(Alpine.store('path'), p);
    headers.set('Authorization', p); 
    fetchBunch();
}

const login = async (u, p) =>{
    let response = await fetch('/api/login', { method: 'POST', body: JSON.stringify({password: p, username: u})})
    if (response.ok){
        Alpine.store('passwordWall', false)
    }
}