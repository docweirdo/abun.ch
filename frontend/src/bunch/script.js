document.addEventListener('alpine:init', () => {
    Alpine.store('links', [])
    Alpine.store('selected', new Map())

    Alpine.store('links', [{ id: 1, url: 'http://google.com', title: 'Google' }, { id: 2, url: 'http://bing.com', title: 'Bing' }, { id: 3, url: 'http://yahoo.com', title: 'Yahoo' }])
})


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
