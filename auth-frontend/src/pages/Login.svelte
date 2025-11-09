<script lang="ts">
  import { login, verifyMfa } from "../api/login";

  let email = "";
  let password = "";
  let mfaCode = "";
  let errorMessage = "";
  let mfaStep: "login" | "mfa" = "login";
  let mfaToken = "";

  async function handleLogin() {
    errorMessage = "";

    if (!email || !password) {
      errorMessage = "请输入邮箱和密码。";
      return;
    }

    console.log("尝试登录...", { email, password });
    const response = await login({ email, password });

    if (response.status === "success" && response.mfa_token) {
      mfaToken = response.mfa_token;
      mfaStep = "mfa";
    } else {
      errorMessage = response.message || "登录失败，请重试。";
    }
  }

  async function handleMfa() {
    errorMessage = "";

    if (!mfaCode) {
      errorMessage = "请输入MFA验证码。";
      return;
    }

    console.log("尝试验证MFA...", { mfa_token: mfaToken, mfa_code: mfaCode });
    const response = await verifyMfa({
      mfa_token: mfaToken,
      mfa_code: mfaCode,
    });

    if (response.status === "success") {
      console.log("MFA验证成功，重定向到主页...");
      window.location.href = "/";
    } else {
      errorMessage = response.message || "MFA验证失败，请重试。";
    }
  }
</script>

<div class="min-h-screen flex items-center justify-center bg-gray-100 p-4">
  <div class="bg-white p-8 rounded-lg shadow-xl w-full max-w-md">
    {#if mfaStep === "login"}
      <h2 class="text-3xl font-extrabold text-gray-900 text-center mb-6">登录</h2>
      <form on:submit|preventDefault={handleLogin} class="space-y-6">
        <div>
          <label for="email" class="block text-sm font-medium text-gray-700">邮箱</label>
          <input
            id="email"
            name="email"
            type="email"
            autocomplete="email"
            required
            bind:value={email}
            class="mt-1 block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm placeholder-gray-400 focus:outline-none focus:ring-blue-500 focus:border-blue-500 sm:text-sm"
            placeholder="请输入您的邮箱"
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
    {:else}
      <h2 class="text-3xl font-extrabold text-gray-900 text-center mb-6">输入MFA验证码</h2>
      <form on:submit|preventDefault={handleMfa} class="space-y-6">
        <div>
          <label for="mfaCode" class="block text-sm font-medium text-gray-700">MFA验证码</label>
          <input
            id="mfaCode"
            name="mfaCode"
            type="text"
            autocomplete="one-time-code"
            required
            bind:value={mfaCode}
            class="mt-1 block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm placeholder-gray-400 focus:outline-none focus:ring-blue-500 focus:border-blue-500 sm:text-sm"
            placeholder="请输入您的MFA验证码"
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
            验证
          </button>
        </div>
      </form>
    {/if}
  </div>
</div>
