<script lang="ts">
// MyUploader.vue
import { createUploaderComponent } from "quasar";
import { computed } from "vue";
import { invoke } from "@tauri-apps/api/core";

function blobToBase64(blob: Blob) {
  return new Promise((resolve, _) => {
    try {
      const reader = new FileReader();
      reader.onloadend = () => resolve(reader.result);
      reader.readAsDataURL(blob);
    } catch (err) {
      console.error(err);
    }
  });
}

function blobToArrayBuffer(blob: Blob) {
  return new Promise((resolve, _) => {
    try {
      const reader = new FileReader();
      reader.onloadend = () => resolve(reader.result);
      reader.readAsArrayBuffer(blob);
    } catch (err) {
      console.error(err);
    }
  });
}

var my_is_uploading = false,
  my_is_busy = false;

export var my_helpers: any = undefined;

// export a Vue component
export default createUploaderComponent({
  // defining the QUploader plugin here

  name: "MyUploader", // your component's name

  props: {
    // ...your custom props
  },

  emits: [
    // ...your custom events name list
  ],

  injectPlugin({ props, emit, helpers }) {
    // can call any other composables here
    // as this function will run in the component's setup()

    // [ REQUIRED! ]
    // We're working on uploading files
    const isUploading = computed(() => {
      return my_is_uploading;
    });

    // [ optional ]
    // Shows overlay on top of the
    // uploader signaling it's waiting
    // on something (blocks all controls)
    const isBusy = computed(() => {
      return my_is_busy;
    });

    // [ REQUIRED! ]
    // Abort and clean up any process
    // that is in progress
    function abort() {
      my_is_uploading = false;
      my_is_busy = false;
    }

    // [ REQUIRED! ]
    // Start the uploading process
    function upload() {
      console.log("upload start");
      my_is_uploading = true;
      my_is_busy = true;
      my_helpers = helpers;
      const queue = helpers.queuedFiles.value.slice(0);
      queue.forEach(async (file: any) => {
        try {
          //transfer all not uploaded files
          // @ts-ignore
          if (file.__status == "uploaded") {
            return;
          }

          helpers.updateFileStatus(file, 'uploading', 0)

          // @ts-ignore
          // window.electronAPI.send("toMain", {
          //   path: "upload-files",
          //   query: "",
          //   data: {
          //     name: file.name,
          //     path: file.path,
          //   },
          // });

          let data_base64_url:String = await blobToBase64(file);
          let data_base64 = data_base64_url.split('base64,')[1]

          invoke("js2rs", {
            message: JSON.stringify({
              path: "upload_files",
              query: "",
              data: JSON.stringify({
                name: file.name,
                data_base64: data_base64,
              }),
            }),
          });

        } catch (err) {
          console.error(err);
        }
      });

      helpers.uploadedFiles.value = helpers.uploadedFiles.value.concat(queue)

      my_is_uploading = false;
      my_is_busy = false;
    }

    return {
      isUploading,
      isBusy,

      abort,
      upload,
    };
  },
});
</script>
<template>
  <div></div>
</template>
