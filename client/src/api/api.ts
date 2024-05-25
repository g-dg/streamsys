import router from "@/router";
import { useAuthStore } from "@/stores/auth";

export const API_URI =
  (import.meta.env.VITE_API as string | undefined) ??
  `${window.location.protocol}//${window.location.host}/`;

interface ApiOptions {
  bodyJson?: boolean;
  returnType?: "json" | "text" | "blob" | "response";
  redirectOnUnauthorized?: boolean;
}

export async function api(
  endpoint: string,
  method: "GET" | "POST" | "PUT" | "DELETE",
  body?: any,
  options?: ApiOptions
): Promise<any> {
  const authStore = useAuthStore();
  const apiToken = authStore.token;

  const response = await fetch(API_URI + "api/" + endpoint, {
    body:
      body !== undefined
        ? options?.bodyJson ?? true
          ? JSON.stringify(body)
          : body
        : undefined,
    credentials: "include",
    headers: {
      "Content-Type": "application/json",
      ...(apiToken != null ? { Authorization: `Bearer ${apiToken}` } : {}),
    },
    method,
    mode: "cors",
    redirect: "follow",
  });

  const returnType = options?.returnType ?? "json";

  // if the raw response is requested as a return type, return it
  if (returnType == "response") {
    return response;
  }

  // go to login page if unauthorized
  if (response.status == 401) {
    await router.push({ name: "logout" });
  }

  if (response.ok) {
    // if response is "No Content", return nothing
    if (response.status == 204) {
      return null;
    }

    // if response is ok, return the requested type
    switch (options?.returnType ?? "json") {
      case "json":
        return await response.json();
      case "text":
        return await response.text();
      case "blob":
        return await response.blob();
    }
  } else {
    // throw error if response is not ok
    throw new Error(await response.text());
  }
}
