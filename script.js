document.addEventListener('alpine:init', async () => {
    
    Alpine.store('links', [])
    Alpine.store('selected', new Map())
    Alpine.store('bunch', new Bunch())
})

document.getElementById('bunch-header').addEventListener('build', (e) => {
    router = e.detail.router;
    
    if (router.is('/:uri([a-zA-Z0-9_-]{6})')) {
        path = router.path.slice(1)
        fetchBunch(router.path.slice(1))
    }
}); 

let path = ''

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

const fetchBunch = async uri => {
    let response = await fetch(`/api/${uri}`)
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

        window.open(link.url, '_blank');
    }

}

const sendClicked = (entry) => {
    fetch(`/api/${path}/clicked/${entry.id}`, {method: 'POST'})
    window.open(entry.url, '_blank')
}