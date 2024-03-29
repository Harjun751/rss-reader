const API_URL = "http://localhost:3000/";

export async function get_channels(id) {
    const url = API_URL + "channel?uid=" + id;
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
    const url = API_URL + "all?cid=2&offset=" + offset;
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

export async function create_channel(uid, name){
    const url = API_URL + "channel";
    const response = await fetch(url, {
        method: "POST",
        headers: {
            "Content-Type": "Application/json",
        },
        body: JSON.stringify(
            {
                uid: uid,
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

export async function delete_channel(uid, cid){
    const url = API_URL + "channel?uid=" +uid +"&cid="+cid;
    const response = await fetch(url,{
        method: "DELETE"
    });
    return response;
// TODO: MODIFY DB TO ACCEPT CASCADING DELETE
}