import { createRouter, createWebHistory } from "vue-router";
import { useAuthStore } from "@/stores/auth";

const login_route = { name: "login" };

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      name: "home",
      path: "/",
      component: () => import("@/views/HomeView.vue"),
    },
    {
      name: "login",
      path: "/login",
      component: () => import("@/views/LoginView.vue"),
      meta: { requiresAuth: false },
    },
    {
      name: "logout",
      path: "/logout",
      component: () => import("@/views/LogoutView.vue"),
      meta: { requiresAuth: false },
    },
    {
      name: "about",
      path: "/about",
      component: () => import("@/views/AboutView.vue"),
      meta: { requiresAuth: false },
    },
    {
      name: "account",
      path: "/account",
      component: () => import("@/views/AccountView.vue"),
    },
    {
      name: "user_list",
      path: "/users",
      component: () => import("@/components/users/UserList.vue"),
    },
    {
      name: "user_edit",
      path: "/users/:id",
      props: true,
      component: () => import("@/components/users/UserEdit.vue"),
    },
    {
      name: "user_create",
      path: "/users/new",
      props: false,
      component: () => import("@/components/users/UserEdit.vue"),
    },
  ],
});

router.beforeEach((to, from, next) => {
  if (to.meta?.requiresAuth) {
    const authStore = useAuthStore();

    if (!authStore.isAuthenticated) {
      next(login_route);
      return;
    }
  }

  next();
});

export default router;
