<script setup>
import { ref } from 'vue'
import { get_all } from "../lib.js"
import PostListItem from "../components/PostListItem.vue"
import PostLoader from "../components/PostLoader.vue"
import { usePostStore } from '@/stores/state.js'

const increment = 10;

const loading = ref(true);
const error = ref(null);
const store = usePostStore();
const posts = store.posts;


async function getPosts(offset){
    console.log("loading posts with offset: " + offset)
    try{
        let list = await get_all(offset)
        posts.push.apply(posts, list)
    } catch (err) {
        error.value = err.toString()
    } finally {
        loading.value = false
    }
}

let shouldScroll = false;
setTimeout(() => {
    shouldScroll = true;
}, 500);

const handleInfiniteScroll = () => {
    const endOfPage = window.innerHeight + window.scrollY >= document.body.offsetHeight;
    if (endOfPage && shouldScroll){
        shouldScroll = false;
        store.increment()
        getPosts(store.offset)
        setTimeout(()=>{
            shouldScroll = true;
        }, 1000)
    }
};
window.addEventListener("scroll", handleInfiniteScroll);

if (posts.length == 0){
    getPosts(store.offset)
} else {
    loading.value = false
}
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