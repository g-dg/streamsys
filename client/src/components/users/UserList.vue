<script setup lang="ts">
import { UserPermission, UsersClient, type User } from "@/api/users";
import { natcasecmp } from "@/helpers/sort";
import { useAuthStore } from "@/stores/auth";
import { computed, onMounted, ref } from "vue";

const authStore = useAuthStore();
const currentUserId = computed(() => authStore.user!.id);

const users = ref<User[] | null>(null);

const loading = ref(0);

async function loadUsers() {
  loading.value++;
  try {
    users.value = (await UsersClient.listUsers()).sort((a, b) =>
      natcasecmp([a.username, b.username])
    );
  } catch (e) {
    console.error(e);
    alert("Error occurred loading users");
  }
  loading.value--;
}
onMounted(loadUsers);
</script>

<template>
  <main>
    <h1>User Administration</h1>

    <button @click="loadUsers">Reload</button>
    <br />

    <RouterLink :to="{ name: 'user_create' }">Create</RouterLink>
    <table v-if="!loading">
      <thead>
        <tr>
          <th></th>
          <th>Username</th>
          <th>Login</th>
          <th>Account</th>
          <th>Users</th>
          <th>System</th>
          <th>Setup</th>
          <th>Operation</th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="user in users" :key="user.id ?? ''">
          <td>
            <RouterLink :to="{ name: 'user_edit', params: { id: user.id } }">
              Edit
            </RouterLink>
          </td>
          <td>
            {{ user.username }}
          </td>

          <td v-if="user.enabled" class="permission-allowed">Yes</td>
          <td v-else class="permission-denied">No</td>

          <td
            v-if="user.permissions & UserPermission.MODIFY_SELF"
            class="permission-allowed"
          >
            Yes
          </td>
          <td v-else class="permission-denied">No</td>

          <td
            v-if="user.permissions & UserPermission.USER_ADMIN"
            class="permission-allowed"
          >
            Yes
          </td>
          <td v-else class="permission-denied">No</td>

          <td
            v-if="user.permissions & UserPermission.SYSTEM_ADMIN"
            class="permission-allowed"
          >
            Yes
          </td>
          <td v-else class="permission-denied">No</td>

          <td
            v-if="user.permissions & UserPermission.SETUP"
            class="permission-allowed"
          >
            Yes
          </td>
          <td v-else class="permission-denied">No</td>

          <td
            v-if="user.permissions & UserPermission.OPERATION"
            class="permission-allowed"
          >
            Yes
          </td>
          <td v-else class="permission-denied">No</td>
        </tr>
      </tbody>
    </table>
    <div v-else style="text-align: center">Loading...</div>
  </main>
</template>

<style lang="scss" scoped>
.permission-allowed {
  // background-color: #9EFF9E;
  color: #007f00;
}
.permission-denied {
  // background-color: #FFC7C7;
  color: #7f0000;
}
.permission-allowed,
.permission-denied {
  font-weight: bold;
}
</style>
