import { defineStore } from 'pinia'
import { ref } from 'vue'

export const useUserStore = defineStore('user', () => {
    const uid = ref(1)

    return { uid }
})

export const useScrollStore = defineStore('scroll', () => {
    const position = ref({x:0, y:0})
    return { position }
})

export const usePostStore = defineStore('post', () => {
    const posts = ref([])
    const offset = ref(0)
    function increment() {
        offset.value+=10
    }
    return {posts, offset, increment}
})