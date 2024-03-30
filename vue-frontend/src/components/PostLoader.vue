<template>
    <div v-for="index in 10" :key="index">
    </div>
</template>

<style scoped>
div{
    height:70px;
    width:100%;
    margin-bottom: 30px;
    border-radius: 15px;
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
</style>