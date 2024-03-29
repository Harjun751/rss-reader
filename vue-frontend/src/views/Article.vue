<script setup>
import { ref, watch } from 'vue'
import { get_article } from "../lib.js"
import { useRoute } from 'vue-router';

const loading = ref(false)
const article = ref(null)
const error = ref(null)
const scrape = ref(false)

async function getArticle(get_url, to_scrape){
    try{
        article.value = await get_article(get_url,to_scrape);
        console.log(article.value)
    } catch(err){
        error.value = err.toString()
    } finally {
        loading.value = false
    }
}
const route = useRoute();
getArticle(route.params.url, scrape.value)

watch(scrape, (new_value) => {
    article.value = null
    loading.value = true
    getArticle(route.params.url, new_value)
})
</script>

<template>
    <div>
        <div v-if="loading">loading...</div>
        <div v-if="error">{{ error }}</div>
        <div v-if="article">
            <main v-if="article">
                <h2>{{ article.title }}</h2>
                <small>{{ article.date }}</small>
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