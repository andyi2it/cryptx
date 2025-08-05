import { createRouter, createWebHistory } from 'vue-router';

const routes = [
  {
    path: '/',
    redirect: '/secrets'
  },
  {
    path: '/secrets',
    name: 'Secrets',
    component: () => import('../views/Secrets.vue')
  },
  {
    path: '/users',
    name: 'Users',
    component: () => import('../views/Users.vue')
  }
];

const router = createRouter({
  history: createWebHistory(),
  routes
});

export default router;
