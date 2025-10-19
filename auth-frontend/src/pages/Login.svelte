<script lang="ts">
  import { login } from "../api/login";

  let username = "";
  let password = "";
  let errorMessage = "";

  async function handleSubmit() {
    errorMessage = "";

    if (!username || !password) {
      errorMessage = "请输入用户名和密码。";
      return;
    }

    console.log("尝试登录...", { username, password });
    const response = await login({ username, password });

    if (response.token) {
      console.log("登录成功，重定向到主页...");
      window.location.href = "/";
    } else {
      errorMessage = "登录失败，请重试。";
    }
  }
</script>

<div class="min-h-screen flex items-center justify-center bg-gray-100 p-4">
  <div class="bg-white p-8 rounded-lg shadow-xl w-full max-w-md">
    <h2 class="text-3xl font-extrabold text-gray-900 text-center mb-6">登录</h2>

    <form on:submit|preventDefault={handleSubmit} class="space-y-6">
      <div>
        <label for="username" class="block text-sm font-medium text-gray-700">用户名</label>
        <input
          id="username"
          name="username"
          type="text"
          autocomplete="username"
          required
          bind:value={username}
          class="mt-1 block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm placeholder-gray-400 focus:outline-none focus:ring-blue-500 focus:border-blue-500 sm:text-sm"
          placeholder="请输入您的用户名"
        />
      </div>

      <div>
        <label for="password" class="block text-sm font-medium text-gray-700">密码</label>
        <input
          id="password"
          name="password"
          type="password"
          autocomplete="current-password"
          required
          bind:value={password}
          class="mt-1 block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm placeholder-gray-400 focus:outline-none focus:ring-blue-500 focus:border-blue-500 sm:text-sm"
          placeholder="您的密码"
        />
      </div>

      {#if errorMessage}
        <p class="text-red-600 text-sm text-center">{errorMessage}</p>
      {/if}

      <div>
        <button
          type="submit"
          class="w-full flex justify-center py-2 px-4 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
        >
          登录
        </button>
      </div>
    </form>
  </div>
</div>
