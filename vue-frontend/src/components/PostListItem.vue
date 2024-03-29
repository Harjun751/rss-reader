<template>
    <RouterLink :to="{ name: 'article', params: { url: data.link }}">
    <div>
        <h2>{{ data.title }}</h2>
        <p>{{  data.description }}</p>
        <small>{{  data.date }}</small>
    </div>
    </RouterLink>
</template>

<script setup>
const props = defineProps(['data'])
</script>

<style scoped>
div {
    margin-bottom: 30px;
}
p{
    margin-top:2px;
}
</style>