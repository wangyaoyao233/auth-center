# jwt 设计

为 MFA 系统设计 jwt 时，应该确保

`在所有的认证步骤都完成之后才发行`

## 两阶段

1. 阶段 1（密码验证）：用户提交用户名和密码
2. 阶段 2（MFA 验证）：用户提交 MFA 代码
3. 发行（jwt）：两个阶段都通过后，才签发最终的，可用于访问 API 的 jwt

## 使用`临时凭证`来串联 2 个阶段

### 阶段 1

1. 用户发送用户名和密码
2. 服务器验证密码是否正确
3. 关键点：服务器不在此时发行“完全访问权限的 jwt”
4. 服务器签发一个“部分认证令牌”（Partial-Auth-Token）

这个“部分认证令牌”也是一个 jwt，但他有严格的限制：

- 极短的有效期（exp）：比如 3-5 分钟，足够输入 MFA
- 受限的权限（scope 或 aud）：它的 aud (Audience) 可能是 "mfa-verification"，这意味着它 只能 用来访问“提交 MFA 代码”这一个 API，不能用于访问其他任何受保护的资源
- 包含用户标识（sub）：必须包含 sub，这样服务器才知道是哪个用户在进行 MFA

### 阶段 2

1. 客户端收到这个“部分令牌”，并将其存储
2. 前端显示 MFA 输入框
3. 用户输入 MFA 代码，客户端将“部分令牌”放在`Authorization`中，和 MFA 代码一起提交
4. 服务器后端：a. 验证这个“部分令牌的签名”，有效期和 aud（确保它确实是用于 MFA 验证的） b. 从令牌中取出 sub，并验证该用户提交的 MFA 代码是否正确
5. MFA 验证成功
6. 正式发行：此时，服务器才签发最终的 Access Token 和 Refresh Token

## 关键设计

在最终的 Access Token 的 Payload 中添加特定的声明 Claims，用来证明这个令牌是 MFA 认证过的。

最标准、最推荐的 Claim 是：

amr (Authentication Methods References)

amr 是一个字符串数组，用于列出用户在认证过程中使用过的所有方法。这是 OIDC（OpenID Connect）规范中定义的标准声明。

示例 JWT Payload

如果一个用户只用了密码登录（非 MFA）：

```json
{
  "iss": "https://api.my-app.com",
  "sub": "user-12345",
  "aud": "https://web.my-app.com",
  "exp": 1678886400,
  "amr": ["pwd"] // "pwd" = Password
}
```

如果一个用户通过了 MFA 登录：

```json
{
  "iss": "https://api.my-app.com",
  "sub": "user-12345",
  "aud": "https://web.my-app.com",
  "exp": 1678886400,
  "amr": ["pwd", "mfa"] // <-- 关键设计！
}
```

## 后续的 API 请求

当你的 API 后端（资源服务器）收到这个 JWT 时，它可以（也应该）检查这个 amr 声明。

这允许你实现“分级授权”：

- 普通 API（如：读取文章）：
  - GET /api/articles
  - 后端检查：只要 JWT 有效即可。
- 高风险 API（如：修改密码、发起转账）：
  - POST /api/user/change-password
  - 后端检查：
    - JWT 签名是否有效？
    - JWT 是否过期？
    - amr 声明中是否包含 "mfa"？
  - 如果 JWT 有效，但 amr 中没有 "mfa"，服务器必须拒绝该请求，并可以返回 403 Forbidden，提示需要 MFA。
