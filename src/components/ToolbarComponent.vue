<template>
  <v-app-bar
    elevation="0"
    app
  >
    <v-toolbar-title>
      <v-row
        ref="title"
        fluid
        height="100%"
        no-gutters
        class="mr-6"
        align="center"
      >
        <v-tooltip
          text="Open Hamburguer Menu"
          location="bottom"
        >
          <template #activator="{ props }">
            <v-icon
              class="mr-2"
              icon="mdi:mdi-source-branch"
              v-bind="props"
              @click="toggleNavbar"
            />
            <p
              class="text-h5"
              v-bind="props"
              @click="toggleNavbar"
            >
              {{ title }}
            </p>
          </template>
        </v-tooltip>
        <v-spacer />
        <v-tooltip
          :text="user.name"
          location="bottom"
        >
          <template #activator="{ props }">
            <v-btn
              class="mr-2"
              icon
              v-bind="props"
            >
              <v-avatar>
                <v-img :src="user.avatar" />
              </v-avatar>
            </v-btn>
          </template>
        </v-tooltip>
        <v-tooltip
          text="Notifications"
          location="bottom"
        >
          <template #activator="{ props }">
            <v-btn
              icon
              class="mr-2"
              v-bind="props"
            >
              <v-icon>mdi:mdi-bell</v-icon>
            </v-btn>
          </template>
        </v-tooltip>
        <v-btn icon>
          <v-icon>mdi:mdi-dots-vertical</v-icon>
          <v-menu activator="parent">
            <v-list>
              <v-list-item
                v-for="(item, index) in menuItems"
                :key="index"
                @click="item.function"
              >
                {{ item.title }}
              </v-list-item>
              <v-list-item class="text-caption text-center">
                {{ getAppVersion }}
              </v-list-item>
            </v-list>
          </v-menu>
        </v-btn>
      </v-row>
    </v-toolbar-title>
    <v-dialog
      v-model="showExitDialog"
      persistent
    >
      <v-card>
        <v-card-title class="headline">
          Exit Application
        </v-card-title>
        <v-card-text>Are you sure you want to exit the application?</v-card-text>
        <v-card-actions>
          <v-spacer />
          <v-btn
            rounded
            @click="showExitDialog = false"
          >
            Cancel
          </v-btn>
          <v-btn
            rounded
            @click="exit"
          >
            Exit
          </v-btn>
        </v-card-actions>
      </v-card>
    </v-dialog>
  </v-app-bar>
</template>

<script lang="ts">
import { CreateComponentPublicInstance, defineComponent } from "vue";
import { useAppStore } from "../stores/app";
import { mapState } from "pinia";
import { exit } from "@tauri-apps/plugin-process";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
const appWindow = getCurrentWebviewWindow()

type ClickCallback = (event: Event) => void;

interface IMenuItem {
    title: string;
    function: ClickCallback;
}

export default defineComponent({
    name: "ToolbarComponent",
    data() {
        return {
            menuItems: [
                { title: "Minimize", function: () => appWindow.minimize() },
                { title: "Maximize", function: () => appWindow.maximize() },
                { title: "Exit", function: () => this.switchExitDialog() },
            ] as IMenuItem[],
            showExitDialog: false,
            x: 0,
            y: 0,
        };
    },
    computed: {
        ...mapState(useAppStore, ["title", "user", "getAppVersion"]),
    },
    mounted() {
        (this.$refs.title as CreateComponentPublicInstance).$el.addEventListener("mousedown", this.startDragging);
    },
    methods: {
        switchExitDialog() {
            this.showExitDialog = true;
        },
        async startDragging(event: Event) {
            if (event.target === (this.$refs.title as CreateComponentPublicInstance).$el) {
                event.preventDefault();
                await appWindow.startDragging();
            }
        },
        toggleNavbar() {
            useAppStore().toggleNavbar();
        },
        exit() {
            exit();
        },
    },
});
</script>

<style scoped>
.v-icon,
.text-h5 {
    cursor: pointer;
}
</style>
