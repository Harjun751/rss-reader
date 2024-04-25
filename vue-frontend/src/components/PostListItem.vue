<template>
    <RouterLink visited-class="visited" :to="{ name: 'article', params: { url: data.link }, query: { pid: data.pid }}" @click="storeScrollPosition">
    <div>
        <h2>{{ data.title }}</h2>
        <p>{{  data.description }}</p>
        <small>{{  small_text }}</small>
    </div>
    </RouterLink>
</template>

<script setup>
import { useScrollStore } from "../stores/state";

const store = useScrollStore();

function storeScrollPosition(){
    store.position = {x:0, y:window.scrollY}
    console.log("Saving scroll position: " + store.position.y);
}

const props = defineProps(['data'])
const date_object = new Date(Date.parse(props.data.date));
function construct_info(date){
    let day = date.toLocaleDateString("en-SG", { weekday: 'short' });
    let month = date.toLocaleDateString("en-SG", { month: 'short' });
    let date_num = date.getDate();
    let year = date.getFullYear();
    let time = date.toLocaleString('en-US', { hour: 'numeric', minute: 'numeric', hour12: true })

    return `${props.data.publisher_name} • ${day}, ${month} ${date_num} ${year} • ${time}`;
}
const small_text = construct_info(date_object);
</script>

<style scoped>
div {
    margin-bottom: 30px;

    animation: loaded 0.2s;
}
p{
    margin-top:2px;
}
</style>