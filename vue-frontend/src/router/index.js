import { createRouter, createWebHistory } from 'vue-router'
import Today from '@/views/Today.vue'
import Article from '@/views/Article.vue'
import All from '@/views/All.vue'
import Settings from '@/views/Settings.vue'
import ChannelSettings from '@/views/ChannelSettings.vue'

const savedPage = null;

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
    if (to.name == "all"){
      if (savedPage!=null){
        this.history.replace(to, savedPage);
        savedPage = null;
        return;
      } else { 
        return { top: 0 }
      }
    } else if (from.name == "all") {
      savedPage = savedPosition;
    }
    return { top: 0 }
  }
})

export default router
