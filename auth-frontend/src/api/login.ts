const API_BASE_URL = import.meta.env.VITE_API_BASE_URL || "";

interface LoginRequest {
  email: string;
  password: string;
}

interface LoginResponse {
  status: string;
  message?: string;
  mfa_token?: string;
}

interface MfaRequest {
  mfa_token: string;
  mfa_code: string;
}

interface MfaResponse {
  status: string;
  message?: string;
  access_token?: string;
  refresh_token?: string;
}

export async function login(credentials: LoginRequest): Promise<LoginResponse> {
  const response = await fetch(`${API_BASE_URL}/api/auth/login`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify(credentials),
  });
  const data: LoginResponse = await response.json();
  return data;
}

export async function verifyMfa(data: MfaRequest): Promise<MfaResponse> {
  const body = {
    user_id: "", // Placeholder, actual user_id should be extracted from mfa_token if needed
    token: data.mfa_code,
  };
  const response = await fetch(`${API_BASE_URL}/api/auth/otp/validate`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      Authorization: `Bearer ${data.mfa_token}`,
    },
    body: JSON.stringify(body),
  });
  const responseData: MfaResponse = await response.json();
  return responseData;
}
