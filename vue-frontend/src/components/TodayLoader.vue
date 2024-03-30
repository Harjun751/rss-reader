<script setup>
import PostLoader from '@/components/PostLoader.vue';
</script>
<template>
    <div class="channel-select-container">
        <select class="channel-select">
            <option>
                
            </option>
        </select>
    </div>
    <PostLoader/>
</template>

<style scoped>
.channel-select {
    all: unset;
    padding: 5px 32px 5px 10px;
    font-family: "Patua One", "serif";
    font-size: 20px;
    width: 150px;
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
    display: inline-block;
    margin-bottom: 20px;
    filter: blur(4px);

    background: linear-gradient(-45deg,  var(--light-secondary) 40%, white 50%, var(--light-secondary) 60%);
    background-size:300%;
    background-position-x: 100%;
    animation: shimmer 1s infinite linear;
}

@keyframes shimmer {
    to{
        background-position-x :0%;
    }
}
/* credit to https://www.codeguage.com/blog/shimmer-effect-html-css for shimmer anim */
</style>