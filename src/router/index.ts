import { createRouter, createWebHistory } from 'vue-router';
import Users from '../views/Users.vue';
import Secrets from '../views/Secrets.vue';

const routes = [
  { path: '/', redirect: '/secrets' },
  { path: '/users', component: Users },
  { path: '/secrets', component: Secrets }
];

const router = createRouter({
  history: createWebHistory(),
  routes
});

export default router;
