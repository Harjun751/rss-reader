<script setup>
import { ref } from 'vue'
import { get_all } from "../lib.js"
import PostListItem from "../components/PostListItem.vue"
import PostLoader from "../components/PostLoader.vue"

const increment = 10;
const offset = ref(0);

const loading = ref(true);
const error = ref(null);
const posts = ref([])

async function getPosts(offset){
    try{
        let list = await get_all(offset)
        posts.value.push.apply(posts.value, list)
        console.log(posts.value)
    } catch (err) {
        error.value = err.toString()
    } finally {
        loading.value = false
    }
}

const handleInfiniteScroll = () => {
    const endOfPage = window.innerHeight + window.scrollY >= document.body.offsetHeight;
    if (endOfPage){
        offset.value += increment;
        getPosts(offset.value)
    }
};
window.addEventListener("scroll", handleInfiniteScroll);

getPosts(offset.value)
</script>

<template>
    <div>
        <div v-if="loading"><PostLoader/></div>
        <div v-if="error">{{ error }}</div>
        <div v-if="posts">
            <div class="post-container">
                <PostListItem :data="post" v-for="post in posts" />
            </div>
        </div>
    </div>

</template>

<style scoped>

</style>