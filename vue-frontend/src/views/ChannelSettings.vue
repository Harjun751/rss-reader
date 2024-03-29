<script setup>
import { ref } from 'vue'
import { get_subscriptions, create_subscription, delete_channel, delete_subscription } from "../lib.js"
import { useRoute } from 'vue-router';
import router from '@/router';

const loading = ref(true)
const subscriptions = ref(null)
const error = ref(false)
const feed_name = ref("")
const route = useRoute();

async function getData(cid){
    try {
        subscriptions.value = await get_subscriptions(cid)
    } catch (err) {
        error.value = err.toString()
    } finally {
        loading.value = false
    }
}

async function unsubscribe(pid){
    try{
        await delete_subscription(route.params.id, pid);
        getData(route.params.id);
    } catch (err) {
        swal("Unfortunately, an error occured :(\n" + err.toString())
    }
}

async function subscribe(){
    create_subscription(route.params.id, feed_name.value)
    .then(() => {
        console.log("bruh")
        getData(route.params.id)
    })
    .catch((err) => {
        swal("Unfortunately, an error occured :(\n\n" + err.toString())
    })
}

async function delete_ch(){
    try{
        // TODO: HARDCODED UID
        swal("Are you sure you want to delete this channel?", {
            buttons: {
                cancel: {
                    text: "Cancel",
                    value: false,
                    visible: true,
                },
                    confirm: {
                    text: "Delete",
                    value: true,
                    visible: true,
                }
            },
        })
        .then((value) => {
            switch (value) {
                case true:
                    const del_res = delete_channel(1, route.params.id);
                    del_res.then(() => {
                        router.push({ name: 'settings'})
                    })
                case false:
                    console.log(false);
            }
        })
    } catch {
        // TODO: proper catch error handling & error showing
        swal("Unfortunately, an error occured.")
    }
}

getData(route.params.id)
</script>

<template>
    <div class="container">
        <h1>Settings</h1>
        <div><h2>Subscribed Feeds</h2></div>
        <div class="fw-text-input">
            <input id="text" v-model="feed_name" placeholder="Enter feed url..."/>
            <input id="submit" @click="subscribe" type="submit" value="Add"/>
        </div>
        <div>
            <table v-if="subscriptions">
                <tr v-for="sub in subscriptions" @click="unsubscribe(sub.pid)">
                    <td>
                        {{ sub.name }} ({{ sub.url }})
                    </td>
                </tr>
            </table>
        </div>
        <div><hr/></div>
        <div id="danger">
            <h2>DANGER ZONE!!!</h2>
            <button @click="delete_ch">DELETE CHANNEL</button>
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

table tr{
  background-color: white;
  cursor:pointer;
}

table tr:nth-child(even) {
  background-color: #dddddd;
}
#danger{
    text-align: center;
}
#danger > *{
    margin-top:40px;
}
#danger button{
    color: black;
    cursor: pointer;
    font-size: 14px;
    font-family: "Patua One", serif;
    background-color: white;
    padding: 13px 10px;
    border: 3px solid black;
}

</style>