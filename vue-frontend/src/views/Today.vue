<template>
    <div>
        <div v-if="loading">Loading...</div>
        <div v-if="error">{{ error }}</div>
        <div class="channel-select-container" v-if="channels">
            <select v-model="selected" class="channel-select">
                <option :value="channel.cid" v-for="channel in channels">
                    {{ channel.name }}
                </option>
            </select>
        </div>
        <div class="post-container">
            <PostListItem :data="post" v-for="post in posts" />
        </div>
    </div>
</template>

<script setup>
import { ref, watch } from 'vue'
import { get_channels, get_posts } from '../lib.js'
import PostListItem from "../components/PostListItem.vue"

const loading = ref(true)
const posts = ref(null)
const channels = ref(null)
const selected = ref(null)
const error = ref(false)

async function getData(){
    try {
        // TODO: uid shared state
        channels.value = await get_channels(1)
        let x = channels.value;
        selected.value = x[0].cid;
    } catch (err) {
        error.value = err.toString()
    } finally {
        loading.value = false
    }
}

async function getPost(val){
    try{
        posts.value = await get_posts(val)
    } catch (err) {
        error.value = err.toString()
    } finally {
        loading.value = false
    }
}

watch(selected, (cid) => {
    getPost(cid)
})

getData()
</script>

<style scoped>
.channel-select {
    all: unset;
    background: var(--light-secondary);
    padding: 5px 32px 5px 10px;
    font-family: "Patua One", "serif";
    font-size: 20px;
}
.channel-select-container::after {
    content: "";
    border: solid black;
    border-width: 0 2px 2px 0;
    display: inline-block;
    padding: 5px;
    transform: rotate(45deg);
    -webkit-transform: rotate(45deg);
    position: relative;
    right: 22px;
    bottom: 3px;
}
.channel-select-container{
    margin-bottom: 20px;
}
</style>