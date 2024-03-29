import { createRouter, createWebHistory } from 'vue-router'
import Today from '@/views/Today.vue'
import Article from '@/views/Article.vue'
import All from '@/views/All.vue'
import Settings from '@/views/Settings.vue'
import ChannelSettings from '@/views/ChannelSettings.vue'

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
      component: Article
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
  ]
})

export default router
