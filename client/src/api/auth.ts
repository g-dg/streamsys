import { api } from "./api";
import type { User } from "./users";

export interface LoginResponse {
  token: string;
  user: User;
}

export class AuthClient {
  static async login(
    username: string,
    password: string
  ): Promise<LoginResponse | false> {
    try {
      let response = await api("auth", "POST", { username, password });
      return response as LoginResponse;
    } catch (e) {
      return false;
    }
  }

  static async getCurrentUser(): Promise<User> {
    let response = await api("auth", "GET");
    return response as User;
  }

  static async logout(): Promise<void> {
    await api("auth", "DELETE");
  }
}
