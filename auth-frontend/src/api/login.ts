const API_BASE_URL = import.meta.env.VITE_API_BASE_URL || "";

interface LoginRequest {
  username: string;
  password: string;
}

interface LoginResponse {
  token: string;
}

export async function login(credentials: LoginRequest): Promise<LoginResponse> {
  const response = await fetch(`${API_BASE_URL}/api/login`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify(credentials),
  });

  if (!response.ok) {
    throw new Error("Login failed");
  }

  const data: LoginResponse = await response.json();
  return data;
}
