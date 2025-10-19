<script lang="ts">
  import { link } from 'svelte-spa-router';
</script>

<div class="p-8 text-center">
  <h1 class="text-3xl font-bold text-green-600">
    欢迎来到主页！
  </h1>
  <p class="mt-4">
    你已成功登录。
  </p>
  <a href="/login" use:link class="mt-6 text-blue-500 hover:underline">
    返回登录页
  </a>
</div>