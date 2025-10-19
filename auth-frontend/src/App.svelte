<script lang="ts">
  import Router from "svelte-spa-router";

  import Home from "./pages/Home.svelte";
  import Login from "./pages/Login.svelte";

  const routes = {
    "/": Home,
    "/login": Login,
    "*": Home,
  };
</script>

<main>
  <Router {routes} />
</main>
