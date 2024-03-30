<script setup>
import { ref } from 'vue'
import { get_channels, create_channel } from "../lib.js"
import FallbackSettings from '@/components/FallbackSettings.vue'
import router from '@/router';

const loading = ref(true)
const channels = ref(null)
const error = ref(false)
const channel_name = ref("")

async function getData(){
    try {
        channels.value = await get_channels()
    } catch (err) {
        error.value = err.toString()
    } finally {
        loading.value = false
    }
}

async function createChannel(){
    try{
        await create_channel(channel_name.value);
        getData();
    } catch{
        swal("Unfortunately, an error occured :(")
    }
}
function navigate(id){
    router.push({ name: 'channel', params: { id: id } })
}
getData()
</script>

<template>
    <div class="container">
        <h1>Settings</h1>
        <div><h2>Channels</h2></div>
        <div class="fw-text-input">
            <input id="text" v-model="channel_name" placeholder="Enter channel name..."/>
            <input @click="createChannel" id="submit" type="submit" value="Add"/>
        </div>
        <div class="table-wrapper">
            <table>
                <tr v-for="ch in channels" @click="navigate(ch.cid)">
                    <td>
                        {{ ch.name }}
                    </td>
                </tr>
            </table>
        </div>
        <FallbackSettings />
    </div>
</template>

<style scoped>
.container{
    margin: 20px auto;
    width: 96%;
}
div{
    margin-top:20px;
}
.fw-text-input{
    margin-top:20px;
}
.fw-text-input #text{
    width:80%;
    height:33px;
    box-sizing: border-box;
    border: 1px solid black;
    border-right: none;
}

.fw-text-input #submit{
    width:20%;
    height:33px;
    border: 1px solid black;
    border-left: none;
}
</style>