// router.ts
import { createRouter, createWebHashHistory, RouteRecordRaw } from "vue-router";
import HomeView from "./views/Home.vue";
import ResultPopup from "./views/ResultPopup.vue";

const routes: Array<RouteRecordRaw> = [
  {
    path: "/",
    name: "home",
    component: HomeView,
  },
  {
    path: "/result",
    name: "result",
    component: ResultPopup,
  },
];

const router = createRouter({
  history: createWebHashHistory(),
  routes,
});

export default router;