<script lang="js">
import { defineComponent, ref } from "vue";
import { appWindow } from "@tauri-apps/api/window";
import { listen } from '@tauri-apps/api/event'
import { invoke } from "@tauri-apps/api/tauri";

import md5 from "md5";



export default defineComponent({
  name: "MainLayout",

  components: {},

  data: () => {
    return {
      localeOptions: [
        { value: "en-US", label: "English" },
        { value: "de-DE", label: "Deutsch" },
      ],
      dialogMe: false,
      dialogMeData: {},

      me: { name: "", avatar: "" },

      //here the data from the server
      title: "ArchiveCat",

      dialogLogout: false,
    };
  },
  computed: {},
  async created() {
    console.log(`MainLayout created()`);

    const that = this;

    this.loading = true;

    invoke("js2rs", {
      message: JSON.stringify({
        path: "user",
        query: "?json=true",
        data: "me",
      })
    }).then((data) => that.doFromMain(data));

  },
  //
  mounted() {
    // based on prepared DOM, initialize echarts instance
    console.log(this.$router.currentRoute.value.path);
  },
  //
  methods: {
    async doFromMain(iData) {
      console.log(`MainLayout doFromMain()`);
      console.log(iData.substring(0, 150));
      this.loading = false;

      try {

        let data = JSON.parse(iData);
        if (data.data) {
          data.data = JSON.parse(data.data);
        }

        let { dataname: lDataName, data: lData, error: lError } = data;

        console.log("listen rs2js event ", lDataName);

        if (lError) {
          this.$q.notify({
            message: "Error: " + lError,
            color: "negative",
            icon: "warning",
          });
          console.error(`Error listen rs2js event ${lError}`)
          return;
        }

        if (!lData || !lDataName) {
          return;
        }

        if (lDataName == "me") {
          this[lDataName] = lData;
          if (this.me.email) {
            this.me.avatar = this.getGravatarURL(this.me.email);
          }
        }
      } catch (err) {
        console.error(err);
        return
      }
    },

    getGravatarURL(email) {
      console.log(`MainLayout getGravatarURL()`);

      if (!email) return "";

      // Trim leading and trailing whitespace from
      // an email address and force all characters
      // to lower case
      const address = String(email).trim().toLowerCase();

      // Create an MD5 hash of the final string
      const hash = md5(address);

      // Grab the actual image URL
      return `https://www.gravatar.com/avatar/${hash}`;
    },

    onDialogMe() {
      console.log(`MainLayout onDialogMe()`);

      if (!this.dialogMe) {
        this.dialogMeData.name = this.me && this.me.name ? this.me.name : "";
        this.dialogMeData.email = this.me.email || "";
        this.dialogMeData.path_name = this.me.path_name || "";
        this.dialogMeData.clone_path = this.me.clone_path || "";
        this.dialogMeData.scan_path = this.me.scan_path || "";
        this.dialogMeData.scan_filter = this.me.scan_filter || "";

      }
      this.dialogMe = !this.dialogMe;
    },

    saveDialogMe() {
      console.log(`MainLayout saveDialogMe()`);

      this.me.name = this.dialogMeData.name || "";
      this.me.email = this.dialogMeData.email || "";
      this.me.path_name = this.dialogMeData.path_name || "";
      this.me.clone_path = this.dialogMeData.clone_path || "";
      this.me.scan_path = this.dialogMeData.scan_path || "";
      this.me.scan_filter = this.dialogMeData.scan_filter || "";
      this.me.avatar = this.getGravatarURL(this.me.email);
      let that = this;
      invoke("js2rs", {
        message: JSON.stringify({
          path: "save_user",
          query: "",
          data: JSON.stringify(this.me),
        })
      }).then((data) => that.doFromMain(data));
      this.dialogMe = false;
    },

    async onLogout(iData) {
      console.log(`MainLayout onLogout(${iData})`);

      if (iData == "1") {
        await appWindow.close();
      } else {
        this.dialogLogout = true;
      }
    },
    saveLanguData() { },
    saveDarkData() { },
  },
});
</script>

<template>
  <q-layout view="hHh lpR fFf" container class="shadow-2 rounded-borders tw-font-sans" style="height: 99vh">
    <!-- header / toolbar -->
    <q-header class="tw-bg-blue-600">
      <q-toolbar inset>
        <q-icon name="img:icons/favicon-128x128.png" size="24px" />

        <q-toolbar-title>
          <strong>{{ title }}</strong>
        </q-toolbar-title>

        <q-btn type="a" @click="$router.push('/dashboard')" outline icon="show_chart"
          :disable="$router.currentRoute.value.path == '/dashboard'">{{ $t("Dashboard") }}</q-btn>
        <q-btn type="a" @click="$router.push('/')" outline icon="table_view"
          :disable="$router.currentRoute.value.path == '/'">{{ $t("SearchAEdit") }}</q-btn>
        <q-space></q-space>

        <div class="row tw-space-x-2">
          <q-chip class="tw-bg-slate-300">
            <q-avatar>
              <q-img v-bind:src="me.avatar" />
            </q-avatar>
            {{ me.name }}
          </q-chip>
          <q-btn type="a" @click="onDialogMe()" :icon="'assignment_ind'"></q-btn>
          <q-select v-model="$i18n.locale" :options="localeOptions" dark :label="$t('Language')" dense emit-value
            map-options options-dense style="min-width: 100px" :popup-content-style="{ backgroundColor: '#99ccff' }"
            @update:model-value="saveLanguData" />
          <q-btn :icon="$q.dark.isActive ? 'light_mode' : 'dark_mode'" aria-label="Dark" @click="
                                                                          {
            $q.dark.toggle();
            saveDarkData();
          }
            ">
            <q-tooltip class="tw-bg-blue-400">{{
              $t("InfoToggle")
            }}</q-tooltip></q-btn>
          <q-btn type="a" @click="onLogout()" :icon="'logout'">
            <q-tooltip>{{ $t("CloseTheApp") }}</q-tooltip>
          </q-btn>
        </div>
      </q-toolbar>
    </q-header>

    <q-page-container>
      <router-view :langu="$i18n.locale" />
    </q-page-container>
  </q-layout>

  <!-- subdialog Logout-->
  <q-dialog v-model="dialogLogout" persistent class="tw-font-sans">
    <q-card style="min-width: 350px">
      <q-card-section>
        <div class="text-h6">{{ $t("Logout") }}</div>
        <q-space />
        {{ $t("CloseTheApp") }}?
      </q-card-section>
      <q-card-actions align="right">
        <q-btn flat :label="$t('OK')" @click="onLogout('1')" class="tw-bg-lime-300">
          <q-tooltip>{{ $t("CloseTheApp") }}</q-tooltip>
        </q-btn>
        <q-btn flat :label="$t('Cancel')" v-close-popup class="tw-bg-red-300"></q-btn>
      </q-card-actions>
    </q-card>
  </q-dialog>
  <!-- subdialog Me-->
  <q-dialog v-model="dialogMe" persistent class="tw-font-sans">
    <q-card style="min-width: 350px">
      <q-card-section>
        <div class="text-h6">{{ $t("User") }}</div>
        <q-space />
        <div class="q-gutter-md col items-start">
          <q-input v-model="dialogMeData.name" label="name"></q-input>
          <q-input v-model="dialogMeData.email" label="Email"></q-input>
          <q-input v-model="dialogMeData.path_name" label="Path"></q-input>
          <q-input v-model="dialogMeData.clone_path" label="Clone"></q-input>
          <q-input v-model="dialogMeData.scan_path" label="Scan"></q-input>
          <q-input v-model="dialogMeData.scan_filter" label="Filter"></q-input>
        </div>
      </q-card-section>
      <q-card-actions align="right">
        <q-btn flat :label="$t('Save')" @click="saveDialogMe()" class="tw-bg-lime-300"></q-btn>
        <q-btn flat :label="$t('Cancel')" v-close-popup class="tw-bg-red-300"></q-btn>
      </q-card-actions>
    </q-card>
  </q-dialog>
</template>

<style lang="scss">
// app global css in SCSS form
.q-pa-xs {
  padding: 1px 1px;
}

.q-checkbox__inner {
  font-size: 25px;
}

.q-table tbody td,
.q-table thead tr {
  height: 32px;
  padding: 3px 8px;
}
</style>
