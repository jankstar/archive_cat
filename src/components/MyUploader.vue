<script lang="ts">
// MyUploader.vue
import { createUploaderComponent } from "quasar";
import { computed } from "vue";

var my_is_uploading = false,
  my_is_busy = false;

export var my_helpers = undefined;

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
      queue.forEach((file) => {
        //alle nicht uploaded Files übertragen
        // @ts-ignore
        if (file.__status == "uploaded") {
          return;
        }
        // @ts-ignore
        window.electronAPI.send("toMain", {
          path: "upload-files",
          query: "",
          data: {
            name: file.name,
            path: file.path,
          },
        });
      });
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
<template><div></div></template>
