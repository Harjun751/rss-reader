<script setup>
import { RouterLink, RouterView } from 'vue-router'
</script>

<template>
   <nav>
    <div><RouterLink activeClass="active" to="/">Today</RouterLink></div>
    <div><RouterLink activeClass="active" to="/all">All</RouterLink></div>
    <div id="settings"><RouterLink to="/settings"><img src="./assets/cog.png"/></RouterLink></div>
  </nav>
  <RouterView/>
</template>

<style scoped>
nav{
    height: 46px;
    background-color: var(--light-secondary);
    margin-top: -8px;
    margin-left: -8px;
    width: calc(100% + 16px);
    margin-bottom:20px;
}

nav div{
    display:inline-block;
    height:46px;
    font-size:20px;
    line-height: 46px;
    font-family: "Patua One", serif;
    margin-left:20px;
}

nav .active {
  text-decoration: underline;
}

#settings {
  float: right;
  margin-right:20px;
}

#settings img{
  height:30px;
  position: relative;
  top: 7px;
}
</style>
