<script setup>
import { get_subscriptions_for_user, get_all_scrape_preferences, set_scrape_preference } from '@/lib';
import { computed, ref } from 'vue'

const subscription = ref(null)
get_subscriptions_for_user().then((val) => {
    subscription.value = val
}).catch((err) => {
    console.log(err);
})
// const prefs = ref([{pid:3, to_scrape:true}])
const prefs = ref(null)
get_all_scrape_preferences().then((val) => {
    prefs.value = val;
}).catch((err) => {
    console.log(err);
});



const full = computed(() => {
    subscription.value.map((x) => {
        const val = prefs.value.find((element) => {
            return element.pid === x.pid
        });
        if (val!=null){
            x.to_scrape = val.to_scrape;
        } else {
            x.to_scrape = null;
        }
    })
    return subscription.value;
})
</script>

<template>
    <div style="margin-top:40px;">
        <h2>Fallback Settings</h2>
        <div v-if="subscription" class="table-wrapper">
            <table>
                <tr v-for="sub in full">
                    <td>
                        <span>{{ sub.name }} | {{ sub.url }}</span>
                        <label class="switch">
                            <input v-model="sub.to_scrape" v-on:change="set_scrape_preference(sub.pid, sub.to_scrape)" type="checkbox">
                            <span class="slider"></span>
                        </label>
                    </td>
                </tr>
            </table>
        </div>
        <div v-else>
            Loading..
        </div>
    </div>
</template>

<style scoped>
div{
    margin-top:20px;
}
td{
    padding-right:0;
    display:flex;
}
td > span{
    margin-right:20px;
}
td label{
    margin-left:auto;
}
tr{
    cursor:default;
}

.switch {
    position: relative;
    width: 60px;
    height: 34px;
}

/* Hide default HTML checkbox */
.switch input {
  opacity: 0;
  width: 0;
  height: 0;
}

/* The slider */
.slider {
  position: absolute;
  cursor: pointer;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: #ccc;
  -webkit-transition: .4s;
  transition: .4s;
}

.slider:before {
  position: absolute;
  content: "";
  height: 26px;
  width: 26px;
  left: 4px;
  bottom: 4px;
  background-color: white;
  -webkit-transition: .4s;
  transition: .4s;
}

input:checked + .slider {
  background-color: #808080;
}

input:focus + .slider {
  box-shadow: 0 0 1px #808080;
}

input:checked + .slider:before {
  -webkit-transform: translateX(26px);
  -ms-transform: translateX(26px);
  transform: translateX(26px);
}

</style>