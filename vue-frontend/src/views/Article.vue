<script setup>
import { ref, watch } from 'vue'
import { get_article, get_scrape_preference } from "../lib.js"
import { useRoute } from 'vue-router';
import ArticleLoader from "@/components/ArticleLoader.vue"

const loading = ref(true)
const article = ref(null)
const error = ref(null)
const route = useRoute();
const small_text = ref(null)

async function getArticle(get_url, to_scrape){
    try{
        article.value = await get_article(get_url,to_scrape);
        const date_object = new Date(Date.parse(article.value.date));
        small_text.value = construct_info(date_object);
    } catch(err){
        error.value = err.toString()
    } finally {
        loading.value = false
    }
}

// populate scrape with initial value from indexeddb
const scrape = ref(false);
get_scrape_preference(Number(route.query.pid)).then((val) => {
    if (val!=null){
        scrape.value = val;
    }
    getArticle(route.params.url, scrape.value)
    watch(scrape, (new_value) => {
        article.value = null
        loading.value = true
        getArticle(route.params.url, new_value)
    })
})

function construct_info(date){
    let day = date.toLocaleDateString("en-SG", { weekday: 'short' });
    let month = date.toLocaleDateString("en-SG", { month: 'short' });
    let date_num = date.getDate();
    let year = date.getFullYear();
    let time = date.toLocaleString('en-US', { hour: 'numeric', minute: 'numeric', hour12: true })

    return `${article.value.publisher_name} • ${day}, ${month} ${date_num} ${year} • ${time}`;
}
</script>

<template>
    <div>
        <div v-if="loading"><ArticleLoader/></div>
        <div v-if="error">{{ error }}</div>
        <div v-if="article">
            <main v-if="article">
                <h2>{{ article.title }}</h2>
                <small>{{ small_text }}</small>
                <p v-html="article.content || article.description"></p>
            </main>
            <div class="center">
                <a :href="article.link">Article Link</a>
            </div>
        </div>
        <div v-if="scrape" class="center">
            <p>Not working too well?</p>
            <button @click="scrape = !scrape">DISENGAGE FALLBACK</button>
        </div>
        <div v-else class="center">
            <p>Not the full article?</p>
            <button @click="scrape = !scrape">ENGAGE FALLBACK</button>
        </div>
    </div>
</template>

<style scoped>
main{
    padding-bottom:20px;
    border-bottom: 3px dashed #808080;
    max-width: 680px;
    margin: auto;

    animation: loaded 0.2s;
}

@keyframes loaded {
    from{
        background-color: var(--light-secondary);
        filter: blur(4px);
        border-radius: 15px;
    }
    to{
        background-color: var(--light-bg);
        filter: blur(0px);
        border-radius: 0px;
    }
}
a{
    font-family: "Patua One", serif;
    font-size: 14px;
    color: #808080;
    text-decoration: underline;
}
.center {
    margin-top:20px;
    text-align: center;
}
.center p{
    text-align: center;
    margin-top: 20px;
    font-family: "Patua One", serif;
    font-size: 14px;
    color: #808080;
}
.center button{
    color: #808080;
    font-family: "Patua One", serif;
    font-size: 14px;
    background-color: white;
    padding: 13px 10px;
    border: 3px solid #808080;
    display: block;
    margin: auto;
    margin-top: 20px;
}
</style>

<style>
img{
    max-width:680px;
    width:100%;
    max-height:70vh;
    display:block;
    margin:auto;
}
</style>