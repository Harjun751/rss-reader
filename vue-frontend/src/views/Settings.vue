<script setup>
import { ref } from 'vue'
import { get_channels, create_channel } from "../lib.js"

const loading = ref(true)
const channels = ref(null)
const error = ref(false)
const channel_name = ref("")

async function getData(){
    try {
        // TODO: uid shared state
        channels.value = await get_channels(1)
    } catch (err) {
        error.value = err.toString()
    } finally {
        loading.value = false
    }
}

async function createChannel(){
    try{
        // TODO: uid shared state
        await create_channel(1, channel_name.value);
        getData();
    } catch{
        swal("Unfortunately, an error occured :(")
    }
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
        <div>
            <table>
                <RouterLink :to="{name:'channel', params: { id: ch.cid }}" v-for="ch in channels">
                    <tr >
                        <td>
                            {{ ch.name }}
                        </td>
                    </tr>
                </RouterLink>
            </table>
        </div>
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
table a{
  background-color: white;
}

table a:nth-child(even) {
  background-color: #dddddd;
}

a, tr{
  display: block;
}
</style>