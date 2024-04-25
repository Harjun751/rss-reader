import { createRouter, createWebHistory } from 'vue-router'
import { ref } from 'vue'
import Today from '@/views/Today.vue'
import Article from '@/views/Article.vue'
import All from '@/views/All.vue'
import Settings from '@/views/Settings.vue'
import ChannelSettings from '@/views/ChannelSettings.vue'
import { useScrollStore } from "@/stores/state.js";

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: '/',
      name: 'home',
      component: Today
    },
    {
      path: '/article/:url',
      name: 'article',
      component: Article,
      props: route => ({query:route.query.pid})
    },
    {
      path: '/all',
      name: 'all',
      component: All
    },
    {
      path: '/settings',
      name: 'settings',
      component: Settings
    },
    {
      path: '/settings/channel/:id',
      name: 'channel',
      component: ChannelSettings
    },
  ],
  scrollBehavior (to, from, savedPosition) {
    let store = useScrollStore()
    if (to.name == "all"){
      console.log("using store position. x:"+store.position.x+" y:"+store.position.y)
      return {top: store.position.y, behavior:"smooth"}
    } else if (to.name == "article") {
      return {top:0}
    }
  }
})

export default router