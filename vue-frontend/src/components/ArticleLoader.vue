<template>
    <div></div>
</template>

<style scoped>
div{
    max-width:680px;
    margin: auto;
    height: 120vh;
    border-radius:15px;

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