<template>
    <RouterLink :to="{ name: 'article', params: { url: data.link }, query: { pid: data.pid }}">
    <div>
        <h2>{{ data.title }}</h2>
        <p>{{  data.description }}</p>
        <small>{{  small_text }}</small>
    </div>
    </RouterLink>
</template>

<script setup>
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