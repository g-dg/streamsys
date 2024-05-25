<script setup lang="ts">
import { UsersClient, UserPermission, type User } from "@/api/users";
import clone from "@/helpers/clone";
import router from "@/router";
import { useAuthStore } from "@/stores/auth";
import { computed, onMounted, ref, watch } from "vue";

const props = defineProps({
  id: String,
});
const userId = computed(() => props.id);

const authStore = useAuthStore();
const currentUserId = computed(() => authStore.user!.id);

const defaultUser: User = {
  id: null,
  username: "",
  new_password: "",
  description: "",
  enabled: true,
  permissions:
    UserPermission.MODIFY_SELF |
    UserPermission.SETUP |
    UserPermission.OPERATION,
};

const user = ref<User>(clone(defaultUser));

const password = ref("");
const passwordConfirm = ref("");
const blankPassword = ref(false);

const permissionModifySelf = ref(false);
const permissionUserAdmin = ref(false);
const permissionSystemAdmin = ref(false);
const permissionSetup = ref(false);
const permissionOperation = ref(false);
function setPermissionsFromBitfield() {
  permissionModifySelf.value =
    (user.value.permissions & UserPermission.MODIFY_SELF) != 0;
  permissionUserAdmin.value =
    (user.value.permissions & UserPermission.USER_ADMIN) != 0;
  permissionSystemAdmin.value =
    (user.value.permissions & UserPermission.SYSTEM_ADMIN) != 0;
  permissionSetup.value = (user.value.permissions & UserPermission.SETUP) != 0;
  permissionOperation.value =
    (user.value.permissions & UserPermission.OPERATION) != 0;
}
const permissionsBitfield = computed(
  () =>
    (permissionModifySelf.value ? UserPermission.MODIFY_SELF : 0) |
    (permissionUserAdmin.value ? UserPermission.USER_ADMIN : 0) |
    (permissionSystemAdmin.value ? UserPermission.SYSTEM_ADMIN : 0) |
    (permissionSetup.value ? UserPermission.SETUP : 0) |
    (permissionOperation.value ? UserPermission.OPERATION : 0)
);
watch(
  permissionsBitfield,
  () => (user.value.permissions = permissionsBitfield.value)
);

const loading = ref(0);

async function loadUser() {
  if (props.id == null) {
    // new user
    user.value = clone(defaultUser);

    password.value = "";
    passwordConfirm.value = "";
    blankPassword.value = false;

    setPermissionsFromBitfield();
  } else {
    // existing user
    loading.value++;
    try {
      user.value = await UsersClient.getUser(props.id);

      password.value = "";
      passwordConfirm.value = "";
      blankPassword.value = false;

      setPermissionsFromBitfield();
    } catch (e) {
      console.error(e);
      alert("Error occurred loading user");
    }
    loading.value--;
  }
}
onMounted(loadUser);
watch(userId, loadUser);

async function createUser() {
  if (password.value == passwordConfirm.value) {
    user.value.new_password = password.value;
  } else {
    alert("Passwords do not match");
    return;
  }

  user.value.permissions = permissionsBitfield.value;

  loading.value++;
  try {
    await UsersClient.createUser(user.value);
    await router.push({ name: "user_list" });
  } catch (e) {
    console.error(e);
    alert("Error occurred creating user");
  }
  loading.value--;
}

async function updateUser() {
  if (password.value != "" || blankPassword.value) {
    if (password.value == passwordConfirm.value) {
      user.value.new_password = password.value;
    } else {
      alert("Passwords do not match");
      return;
    }
  } else {
    user.value.new_password = null;
  }

  user.value.permissions = permissionsBitfield.value;

  loading.value++;
  try {
    await UsersClient.updateUser(userId.value!, user.value);
  } catch (e) {
    console.error(e);
    alert("Error occurred updating user");
  }
  await loadUser();

  // update user in auth store if editing the current user
  if (userId.value == currentUserId.value) {
    let authUser = clone(user.value);
    authUser.new_password = null;
    authStore.user = authUser;
  }

  loading.value--;
}

async function deleteUser() {
  if (confirm(`Really delete user "${user.value.username}"?`)) {
    loading.value++;
    try {
      await UsersClient.deleteUser(userId.value!);
    } catch (e) {
      console.error(e);
      alert("Error occurred deleting user");
    }
    await router.push({ name: "user_list" });
    loading.value--;
  }
}

async function invalidateSessions() {
  if (
    confirm(`Really invalidate sessions for user "${user.value.username}"?`)
  ) {
    loading.value++;
    try {
      await UsersClient.invalidateSessions(userId.value!);
    } catch (e) {
      console.error(e);
      alert("Error occurred invalidating sessions");
    }
    await loadUser();
    loading.value--;
  }
}
</script>

<template>
  <main>
    <h1>User Administration</h1>

    <RouterLink :to="{ name: 'user_list' }">&lt; All Users</RouterLink>

    <template v-if="userId != null">
      <br />
      <button v-if="userId != null" @click="loadUser" :disabled="loading > 0">
        Revert / Reload
      </button>
      <br />
    </template>

    <form v-if="loading == 0" @submit.prevent>
      <div v-if="userId != null">
        User ID: <code>{{ user.id }}</code>
        <br />
      </div>

      <label for="username">Username: </label>
      <input v-model="user.username" type="text" id="username" />

      <br />

      <label for="password">Password: </label>
      <input
        v-model="password"
        type="password"
        autocomplete="off"
        id="password"
      />

      <template v-if="userId != null">
        <label for="blank_password"> Blank: </label>
        <input v-model="blankPassword" type="checkbox" id="blank_password" />
      </template>

      <br />

      <label for="password_confirm">Confirm password: </label>
      <input
        v-model="passwordConfirm"
        type="password"
        autocomplete="off"
        id="password_confirm"
      />

      <br />

      <label for="description">Description: </label>
      <textarea v-model="user.description" id="description"></textarea>

      <br />

      <label for="enabled">Can Login: </label>
      <input
        v-model="user.enabled"
        :disabled="userId == currentUserId"
        type="checkbox"
        id="enabled"
      />

      <br />

      <label for="permission_modify_self">Manage Self: </label>
      <input
        v-model="permissionModifySelf"
        type="checkbox"
        id="permission_modify_self"
      />

      <br />

      <label for="permission_user_admin">Manage Users: </label>
      <input
        v-model="permissionUserAdmin"
        :disabled="userId == currentUserId"
        type="checkbox"
        id="permission_user_admin"
      />

      <br />

      <label for="permission_system_admin">Manage System: </label>
      <input
        v-model="permissionSystemAdmin"
        type="checkbox"
        id="permission_system_admin"
      />

      <br />

      <label for="permission_setup">Setup: </label>
      <input v-model="permissionSetup" type="checkbox" id="permission_setup" />

      <br />

      <label for="permission_operation">Operation: </label>
      <input
        v-model="permissionOperation"
        type="checkbox"
        id="permission_operation"
      />

      <br />

      <button v-if="userId == null" @click="createUser" type="submit">
        Create
      </button>
      <button v-if="userId != null" @click="updateUser" type="submit">
        Update
      </button>
      <button
        v-if="userId != null"
        @click="deleteUser"
        :disabled="userId == currentUserId"
      >
        Delete
      </button>
      <button v-if="userId != null" @click="invalidateSessions">Logout</button>
    </form>
    <em v-else>Loading...</em>
  </main>
</template>

<style lang="scss" scoped></style>
