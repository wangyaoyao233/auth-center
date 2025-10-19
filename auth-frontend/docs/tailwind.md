# 安装和配置 tailwind

## 安装依赖

```bash
npm install tailwindcss @tailwindcss/vite
```

## 初始化

在 vite.config.ts 中添加

```ts
import { defineConfig } from "vite";
import tailwindcss from "@tailwindcss/vite";

export default defineConfig({
  plugins: [tailwindcss()],
});
```

在 css 中导入

```css
@import "tailwindcss";
```
