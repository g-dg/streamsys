import { api } from "./api";

export interface User {
  id: string | null;
  username: string;
  new_password: string | null;
  description: string;
  enabled: boolean;
  permissions: number;
}

export class UserPermission {
  static ANY = -1;
  static MODIFY_SELF = 1 << 0;
  static USER_ADMIN = 1 << 1;
  static SYSTEM_ADMIN = 1 << 2;
  static SETUP = 1 << 3;
  static OPERATION = 1 << 4;
}

export class UsersClient {
  static async listUsers(): Promise<User[]> {
    const response = await api("users", "GET");
    return response as User[];
  }

  static async getUser(user_id: string): Promise<User> {
    const response = await api(`users/${encodeURIComponent(user_id)}`, "GET");
    return response as User;
  }

  static async createUser(user: User): Promise<string> {
    const response = await api("users", "POST", user);
    return response as string;
  }

  static async updateUser(user_id: string, user: User): Promise<void> {
    await api(`users/${encodeURIComponent(user_id)}`, "PUT", user);
  }

  static async deleteUser(user_id: string): Promise<void> {
    await api(`users/${encodeURIComponent(user_id)}`, "DELETE");
  }

  static async invalidateSessions(user_id: string): Promise<void> {
    await api(`users/${encodeURIComponent(user_id)}/sessions`, "DELETE");
  }

  static async changePassword(
    user_id: string,
    password: string
  ): Promise<void> {
    await api(`users/${encodeURIComponent(user_id)}/password`, "PUT", password);
  }
}
