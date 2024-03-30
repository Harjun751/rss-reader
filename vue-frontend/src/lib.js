import { useUserStore } from "./stores/state";
const API_URL = "http://localhost:3000/";

export async function get_channels() {
    const store = useUserStore();
    const url = API_URL + "channel?uid=" + store.uid;
    const response = await fetch(url);
    const channels = await response.json();
    return channels;
}

export async function get_posts(id) {
    const url = API_URL + "feed?cid=" + id;
    const response = await fetch(url);
    const posts = await response.json();
    return posts;
}


export async function get_article(get_url, to_scrape) {
    const delay = ms => new Promise(res => setTimeout(res, ms));
    await delay(500);
    const url = API_URL + "read";
    const response = await fetch(url, {
        method: "POST",
        headers: {
            "Content-Type": "application/json",
        },
        body: JSON.stringify(
            {
                id: null,
                url: get_url,
                scrape: to_scrape,
            }
        )
    });
    const article = await response.json();
    return article;
}


export async function get_all(offset){
    const store = useUserStore();
    const url = API_URL + "all?uid=" + store.uid + "&offset=" + offset;
    const response = await fetch(url);
    const posts = await response.json();
    return posts
}

export async function get_subscriptions(id){
    const url = API_URL + "sub?cid=" + id;
    const response = await fetch(url);
    const subs = await response.json();
    return subs
}

export async function get_subscriptions_for_user(){
    const store = useUserStore();
    const url = API_URL + "sub?uid=" + store.uid;
    const response = await fetch(url);
    const subs = await response.json();
    return subs
}

export async function create_channel(name){
    const store = useUserStore();
    const url = API_URL + "channel";
    const response = await fetch(url, {
        method: "POST",
        headers: {
            "Content-Type": "Application/json",
        },
        body: JSON.stringify(
            {
                uid: store.uid,
                name: name
            }
        )
    });
    return response;
}

export async function create_subscription(cid, url_to_subscribe){
    cid = Number(cid);
    const url = API_URL + "sub";

    return fetch(url, 
    {
        method: "POST",
        headers: {
            "Content-Type": "Application/json",
            "Accept": "text/html", 
        },
        body: JSON.stringify(
            {
                cid: cid,
                url: url_to_subscribe
            }
        )
    })
    .then((resp) => {
        if (!resp.ok){
            return resp.text().then((text) => {throw new Error(text)})
        }
    })
    .catch(error => {
        throw(error)
    });
}

export async function delete_subscription(cid, pid){
    const url = API_URL + "sub";
    const response = await fetch(url,{
        method: "DELETE",
        headers:{
            "Content-Type": "Application/json",
        },
        body: JSON.stringify(
            {
                cid: Number(cid),
                pid: Number(pid)
            }
        )
    });
}

export async function delete_channel(cid){
    const store = useUserStore();
    const url = API_URL + "channel?uid=" + store.uid +"&cid="+cid;
    return fetch(url,{
        method: "DELETE"
    }).then((resp) => {
        if (!resp.ok){
            return resp.text().then((text) => {throw new Error(text)})
        }
    })
    .catch((err) => {
        throw(err);
    })
}
// re-write using IDB and asynchronous. I see why now.
export async function set_scrape_preference(pid, value){
    const request = window.indexedDB.open("scrape-preferences");
    request.onsuccess = (event) => {
        var db = event.target.result;
        db.onerror = (event) => {
            console.log(`database err ${event.target.error}`)
        }
        const prefObjStore = db.transaction("prefs", "readwrite").objectStore("prefs");
        prefObjStore.put({ pid: pid, to_scrape: value })

    }
    request.onupgradeneeded = (event) => {
        console.log("UPGRADE NEEDED!")
        const db = event.target.result;
        const objectStore = db.createObjectStore("prefs", {keyPath: "pid"});
        objectStore.transaction.oncomplete = (event) => {
            const prefObjStore = db.transaction("prefs", "readwrite").objectStore("prefs");
            prefObjStore.put({ pid: pid, to_scrape: value })
        }
    }
    request.onerror = (event) => {
        console.log(`Error: ${event.target}`)
    }
}

export async function get_scrape_preference(pid) {
    return new Promise((resolve, reject) => {
        const request = window.indexedDB.open("scrape-preferences");
        request.onsuccess = (event) => {
            var db = event.target.result;
            db.onerror = (event) => {
                reject(event.target);
            }
            const prefObjStore = db.transaction("prefs", "readonly").objectStore("prefs");
            const request = prefObjStore.get(pid);
            request.onerror = (event) => {
                console.log(`error: ${event.target}`);
                reject(event.target);
            };
            request.onsuccess = (event) => {
                if (request.result!=null){
                    resolve(request.result.to_scrape);
                } else {
                    resolve(null);
                }
            }
        }
        request.onupgradeneeded = (event) => {
            console.log("UPGRADE NEEDED!")
            const db = event.target.result;
            const objectStore = db.createObjectStore("prefs", {keyPath: "pid"});
            objectStore.transaction.oncomplete = (event) => {
                const prefObjStore = db.transaction("prefs", "readonly").objectStore("prefs");
                const request = prefObjStore.get(pid);
                request.onerror = (event) => {
                    resolve(event.target)
                };
                request.onsuccess = (event) => {
                    resolve(request.result.to_scrape);
                }
            }
        }
        request.onerror = (event) => {
            resolve(event.target)
        }
    })
}

export async function get_all_scrape_preferences() {
    return new Promise((resolve, reject) => {
        const request = window.indexedDB.open("scrape-preferences");
        request.onsuccess = (event) => {
            var db = event.target.result;
            db.onerror = (event) => {
                reject(event.target);
            }
            const prefObjStore = db.transaction("prefs", "readonly").objectStore("prefs");
            const request = prefObjStore.getAll();
            request.onerror = (event) => {
                console.log(`error: ${event.target}`);
                reject(event.target);
            };
            request.onsuccess = (event) => {
                if (request.result!=null){
                    resolve(request.result);
                } else {
                    resolve(null);
                }
            }
        }
        request.onupgradeneeded = (event) => {
            console.log("UPGRADE NEEDED!")
            const db = event.target.result;
            const objectStore = db.createObjectStore("prefs", {keyPath: "pid"});
            objectStore.transaction.oncomplete = (event) => {
                const prefObjStore = db.transaction("prefs", "readonly").objectStore("prefs");
                const request = prefObjStore.getAll();
                request.onerror = (event) => {
                    resolve(event.target)
                };
                request.onsuccess = (event) => {
                    resolve(request.result);
                }
            }
        }
        request.onerror = (event) => {
            resolve(event.target)
        }
    })
}