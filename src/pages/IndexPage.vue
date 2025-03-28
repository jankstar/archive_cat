<script>
import { exportFile } from "quasar";
import { defineComponent } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from '@tauri-apps/api/event';
import myuploader, { my_helpers } from "./../components/MyUploader.vue";

/**
 * Parse a localized number to a float.
 * @param {string} stringNumber - the localized number
 * @param {string} locale - [optional] the locale that the number is represented in. Omit this parameter to use the current locale.
 */
function parseLocaleNumber(stringNumber, locale) {
  var thousandSeparator = Intl.NumberFormat(locale).format(11111).replace(/\p{Number}/gu, '');
  var decimalSeparator = Intl.NumberFormat(locale).format(1.1).replace(/\p{Number}/gu, '');

  return parseFloat(stringNumber
    .replace(new RegExp('\\' + thousandSeparator, 'g'), '')
    .replace(new RegExp('\\' + decimalSeparator), '.')
  );
}

function wrapCsvValue(val, formatFn) {
  let formatted = formatFn !== void 0 ? formatFn(val) : val;

  formatted =
    formatted === void 0 || formatted === null ? "" : String(formatted);

  formatted = formatted.split('"').join('""');
  /**
   * Excel accepts \n and \r in strings, but some other CSV parsers do not
   * Uncomment the next two lines to escape new lines
   */
  // .split('\n').join('\\n')
  // .split('\r').join('\\r')

  return `"${formatted}"`;
}

// function get_formatter(iLangu, iCurrency) {
//   return new Intl.NumberFormat(iLangu, { style: 'currency', currency: iCurrency })
// }

export default defineComponent({
  name: "IndexPage",
  props: ["langu", "footer"],
  components: {
    myuploader,
  },
  //here are the data of the view
  data: () => {
    return {
      loading: false,
      dialogUpload: false,
      fileUpload: "",
      toggle: "0",
      toggleProtocol: "0",
      dialogPDF: false,
      filter: "",
      search: "*",
      seach_field: "body",
      cat_seach_field: [
        { value: "body", label: "body" },
        { value: "subject", label: "subject" },
        { value: "status", label: "status" },
        { value: "date", label: "date" },
        { value: "amount", label: "amount" },
        { value: "sender_name", label: "sender_name" },
        { value: "recipient_name", label: "recipient_name" },
        { value: "category", label: "category" },
      ],
      maxline: "50",

      detailData: {},

      my_footer: {},

      document: [],
      selected: [],
      category: [""],
      status: [""],
      pagination: { rowsPerPage: 0 },
      visibleColumns: [
        "index",
        "date",
        "amount",
        "status",
        "subject",
        "sender_name",
        "recipient_name",
        "category",
      ],
      columns: [
        {
          name: "index",
          label: "#",
          field: "index",
          align: "left",
          sortable: true,
        },
        {
          name: "id",
          label: "id",
          field: "id",
          align: "left",
          sortable: true,
        },
        {
          name: "date",
          label: "date",
          field: "date",
          align: "right",
          sortable: true,
          format: (val, row) => //`${val ? val.substr(0, 10) : ""}`,
          {
            //console.log("format date")
            if (!(val) || val == "") { return "" };
            let l_data = new Date(val.substr(0, 10));
            let myLangu = document.getElementById("langu")?.innerHTML || 'de-DE';
            return l_data.toLocaleDateString(myLangu);
          },
        },
        {
          name: "amount",
          label: "amount",
          field: "amount",
          align: "right",
          sortable: true,
          format: (val, row) => `${val} ${row.currency || 'EUR'}`,
          //format: (val, row) => `${parseFloat(val ? val : "0.0").toFixed(2)}`,
          // format: (val, row) => {
          //   console.log("format");
          //   let myLangu = document.getElementById("langu")?.innerHTML || 'de-DE';
          //   let myCurrency = row && row.currency && row.currency != "" ? row.currency : 'EUR';
          //   return get_formatter(myLangu, myCurrency).format(val)
          // }
        },
        {
          name: "status",
          label: "status",
          field: "status",
          align: "left",
          sortable: true,
        },
        {
          name: "subject",
          label: "subject",
          field: "subject",
          align: "left",
          sortable: true,
          format: (val, row) => `${val ? val.substr(0, 90) : ""}`,
        },
        {
          name: "sender_name",
          label: "sender name",
          field: "sender_name",
          align: "left",
          sortable: true,
        },
        {
          name: "sender_addr",
          label: "sender addr",
          field: "sender_addr",
          align: "left",
          sortable: true,
        },
        {
          name: "from",
          label: "from",
          field: "from",
          align: "left",
          sortable: true,
        },
        {
          name: "recipient_name",
          label: "recipient name",
          field: "recipient_name",
          align: "left",
          sortable: true,
        },
        {
          name: "recipient_addr",
          label: "recipient addr",
          field: "recipient_addr",
          align: "left",
          sortable: true,
        },
        {
          name: "to",
          label: "to",
          field: "to",
          align: "left",
          sortable: true,
        },
        {
          name: "category",
          label: "category",
          field: "category",
          align: "left",
          sortable: true,
        },
      ],
      Chart1: {},
      option1: {},
      Chart2: {},
      option2: {},
      //
      moneyFormatForComponent: {
        decimal: ".",
        thousands: ",",
        prefix: " ",
        suffix: " Eur",
        precision: 2,
        masked: true,
      },
    };
  },

  computed: {},
  async created() {
    console.log(`IndexPage created()`);
    const that = this;

    await listen('rs2js', (event) => {
      try {
        console.log(`IndexPage rs2js event listen`);

        let data = JSON.parse(event.payload);
        if (data.data) {
          data.data = JSON.parse(data.data);
        }

        that.doFromMain(event.payload);

      } catch (err) {
        console.error(err);
      }
    });

  },

  async mounted() {
    // based on prepared DOM, initialize echarts instance
    console.log(`IndexPage mounted()`);

    this.loading = true;
    let that = this;
    invoke("js2rs", {
      message: JSON.stringify({
        path: "category",
        query: "?json=true",
        data: "category",
      }),
    }).then((data) => that.doFromMain(data));

    this.loading = true;
    invoke("js2rs", {
      message: JSON.stringify({
        path: "status",
        query: "?json=true",
        data: "status",
      }),
    }).then((data) => that.doFromMain(data));

    if (this.search == "") {
      this.search = "*";
    }

    this.onSearch();
  },
  //
  updated() {
    console.log(`IndexPage updated()`);

    //this.my_footer = this.footer;

    // if (this.ServerData !== this.data && this.data) {
    //   this.doFromMain(this.data);
    //   this.ServerData = this.data;
    // } else if (this.document && this.document.length != 0) {
    //   let that = this;
    //   this.document.forEach((row, index) => {
    //     row.amount = new Intl.NumberFormat(that.langu, { minimumFractionDigits: 2 }).format(row.amount_row);
    //   });
    // }

    this.buildCatalogues();
  },
  //
  methods: {
    buildCatalogues() {
      this.cat_seach_field = [
        { value: "body", label: this.$t("body") },
        { value: "subject", label: this.$t("subject") },
        { value: "status", label: this.$t("status") },
        { value: "date", label: this.$t("date") },
        { value: "amount", label: this.$t("amount") },
        { value: "sender_name", label: this.$t("sender_name") },
        { value: "recipient_name", label: this.$t("recipient_name") },
        { value: "category", label: this.$t("category") },
      ];
    },

    async doFromMain(iData) {
      console.log(`IndexPage doFromMain()`);
      console.log(iData.substring(0, 150));
      this.loading = false;
      try {
        let data = JSON.parse(iData);
        if (data.data) {
          data.data = JSON.parse(data.data);
        }

        let { dataname: lDataName, data: lData, error: lError } = data;

        let that = this;

        //let { dataname: lDataName, data: lData, error: lError } = iData;

        if (lError) {
          this.$q.notify({
            message: "Error: " + lError.message,
            color: "negative",
            icon: "warning",
          });

          this.footer.push(`Error: ${lError.message}`);

          return;
        }

        if (lDataName == "info") {
          // this.$q.notify({
          //   type: "info",
          //   message: `${lData}`,
          // });

          this.footer.push(`Info: ${lData}`);

          return;
        }

        if (lDataName == "upload_files") {
          if (my_helpers) {
            console.log(lData);
            // this.$q.notify({
            //   message: "Datei verarbeitet: " + lData.name,
            //   color: "positive",
            // });

            this.footer.push(`Info: Datei ${lData.name} verarbeitet.`);

            console.log(my_helpers);

            const queue = my_helpers.queuedFiles.value.slice(0);
            queue.forEach((file) => {
              if (file.name == lData.name) {
                my_helpers.updateFileStatus(file, "uploaded");
              }
            });
          }
          return;
        }

        if (lDataName == "save_document") {
          this.onToggle("0");
          this.onSearch();
          return;
        }

        if (!lData) {
          return;
        }

        if (lDataName == "status" && lData) {
          this[lDataName] = lData;
        }

        if (lDataName == "category" && lData) {
          this[lDataName] = lData;
        }

        if (lDataName == "document" && lData) {
          this[lDataName] = lData;

          this.loading = false;
          let lClear = true;
          //Generate index
          this[lDataName]?.forEach((row, index) => {
            try {
              row.index = index;
              //row.amount = row.amount ? row.amount : 0.0; //if NaN than 0.0
              row.amount_row = row.amount;
              row.amount = new Intl.NumberFormat(that.langu, { minimumFractionDigits: 2 }).format(row.amount_row);


              //flatten array
              //row.attachment = JSON.stringify(row.attachment);
              row.category = JSON.stringify(JSON.parse(row.category));
              row.from = JSON.stringify(JSON.parse(row.from));
              row.to = JSON.stringify(JSON.parse(row.to));

              if (that.selected?.[0]?.id == row.id) {
                //Apply selected values
                that.selected[0] = row;
                lClear = false;
              }
            } catch (e) {
              console.error(e);
            }
          });
          if (lClear == true) {
            //the selected element no longer exists
            this.selected = [];
          }
        }

        if (lDataName == "save_file" && lData) {
          //safe file without error
          return;
        }

        if (lDataName == "pdfbase64" && lData) {
          this.detailData.pdfbase64 = lData;
          //console.log(lData);
          const b64toBlob = (b64Data, contentType = "", sliceSize = 512) => {
            const byteCharacters = atob(b64Data);
            const byteArrays = [];

            for (
              let offset = 0;
              offset < byteCharacters.length;
              offset += sliceSize
            ) {
              const slice = byteCharacters.slice(offset, offset + sliceSize);

              const byteNumbers = new Array(slice.length);
              for (let i = 0; i < slice.length; i++) {
                byteNumbers[i] = slice.charCodeAt(i);
              }

              const byteArray = new Uint8Array(byteNumbers);
              byteArrays.push(byteArray);
            }

            const blob = new Blob(byteArrays, { type: contentType });
            return blob;
          };

          this.detailData.url_blob = URL.createObjectURL(
            b64toBlob(lData, "application/pdf")
          );
          return;

        } else {
          this.detailData.pdfbase64 = "";
          this.detailData.url_blob = "";
          return;
        }
      } catch (err) {
        console.error(err);
        return;
      }
    },

    //
    table_selection(sel) {
      console.log(`IndexPage table_selection()`);

      if (sel.added) {
        //clone
        this.detailData = { ...sel.rows[0] };

        //de-flatten array
        this.detailData.category = this.detailData.category
          ? JSON.parse(this.detailData.category)
          : [];
        //this.detailData.attachment = this.detailData.attachment ? JSON.parse(this.detailData.attachment) : [];
        this.detailData.from = this.detailData.from
          ? JSON.parse(this.detailData.from)
          : [];
        this.detailData.to = this.detailData.to
          ? JSON.parse(this.detailData.to)
          : [];

        //this.detailData.hasAttachment = this.detailData.attachment.length ? true : false;

        //this.detailData.attachment_filename = this.detailData.hasAttachment ? this.detailData.attachment[0].filename : "";

        this.detailData.pdfbase64 = "";
      } else {
        this.detailData = {};
      }
    },

    //Search in the list
    onSearch(props) {
      console.log(`IndexPage onSearch()`);

      this.selected = [];

      let that = this;
      this.loading = true;
      if (this.search === "" || this.search === null) {
        this.search = "*";
      }

      this.loading = true;
      invoke("js2rs", {
        message: JSON.stringify({
          path: "document",
          query:
            "?q=" +
            this.seach_field +
            ":" +
            this.search.replaceAll("%", "*") +
            "&sort=date%20desc&rows=" +
            this.maxline,
          data: "document",
        }),
      })
        .then((data) => that.doFromMain(data));

      this.toggle = "0";
      this.toggleProtocol = "0";
    },

    //Delete data ->
    onDelete(props) {
      console.log(`IndexPage onDelete()`);

      let lNow = new Date();
      this.detailData["deleted_at"] = lNow.toISOString();
      this.onSubmit(props);
    },

    //Post data
    onSubmit(props) {
      console.log(`IndexPage onSubmit()`);

      //neue Werte in document übernehmen
      let lIndex = this.detailData["index"];
      if (this.document[lIndex]) {
        for (let prop in this.document[lIndex]) {
          this.document[lIndex][prop] = this.detailData[prop];
        }

        //de-flatten array
        this.document[lIndex].category = this.detailData.category
          ? JSON.stringify(this.detailData.category)
          : "[]";
        //this.record[lIndex].attachment = this.detailData.attachment ? JSON.stringify(this.detailData.attachment) : "[]";
        this.document[lIndex].from = this.detailData.from
          ? JSON.stringify(this.detailData.from)
          : "[]";
        this.document[lIndex].to = this.detailData.to
          ? JSON.stringify(this.detailData.to)
          : "[]";

        let send_data = { ...this.document[lIndex] }; //clone this
        send_data.amount = send_data.amount_row;
        delete send_data.index;
        delete send_data.amount_row;

        let that = this;
        invoke("js2rs", {
          message: JSON.stringify({
            path: "save_document",
            query: "",
            data: JSON.stringify(send_data),
          }),
        }).then((data) => that.doFromMain(data));
      }
    },

    //Reset data
    onReset() {
      console.log(`IndexPage onReset()`);

      this.created();
    },

    //Starts display for display PDF
    displayPDF() {
      console.log(`IndexPage displayPDF()`);

      if (this.detailData["file"] != "") {
        this.dialogPDF = true;
      }
    },

    //Record back
    onLeft() {
      console.log(`IndexPage onLeft()`);

      if (this.detailData["index"] && this.detailData["index"] != 0) {
        let lIndex = this.detailData["index"];
        let sel = {};
        sel.added = true;
        sel.rows = [];
        sel.rows.push(this.document[lIndex - 1]);

        //Felder setzen
        this.table_selection(sel);
        //PDF lesen
        if (this.detailData.id && this.detailData.filename) {
          //detail -> jetzt PDF lesen
          let that = this;
          invoke("js2rs", {
            message: JSON.stringify({
              path: "pdf",
              query: JSON.stringify({
                id: this.detailData.id,
                filename: this.detailData.filename,
              }),
              data: "pdfbase64",
            }),
          }).then((data) => that.doFromMain(data));
        }
      }
    },

    //Data set before
    onRight() {
      console.log(`IndexPage onRight()`);

      if (this.detailData["index"] || this.detailData["index"] == 0) {
        let lIndex = this.detailData["index"];
        if (!this.document[lIndex + 1]) {
          return;
        }
        let sel = {};
        sel.added = true;
        sel.rows = [];
        sel.rows.push(this.document[lIndex + 1]);

        //Felder setzen
        this.table_selection(sel);
        //PDF lesen
        if (this.detailData.id && this.detailData.filename) {
          //detail -> jetzt PDF lesen
          let that = this;
          invoke("js2rs", {
            message: JSON.stringify({
              path: "pdf",
              query: JSON.stringify({
                id: this.detailData.id,
                filename: this.detailData.filename,
              }),
              data: "pdfbase64",
            }),
          }).then((data) => that.doFromMain(data));
        }
      }
    },

    //Toggle table <-> Detail
    onToggle(props) {
      console.log(`IndexPage onToggle()`);

      if (
        (props == "1" &&
          this.selected.length != 0 &&
          this.selected[0].id != "") ||
        props == "0"
      ) {
        if (props == "1" && this.detailData.id && this.detailData.filename) {
          //detail -> jetzt PDF lesen
          let that = this;
          invoke("js2rs", {
            message: JSON.stringify({
              path: "pdf",
              query: JSON.stringify({
                id: this.detailData.id,
                filename: this.detailData.filename,
              }),
              data: "pdfbase64",
            }),
          }).then((data) => that.doFromMain(data));
        }

        this.toggle = props;
      } else {
        this.$q.notify({
          progress: true,
          message: "Please mark one line.",
          color: "warning",
          actions: [
            {
              label: "OK",
              color: "black",
              handler: () => {
                /* ... */
              },
            },
          ],
        });
      }
      if (this.toggle == "0") {
        this.toggleProtocol = "0";
        //List - if no save, reload the values from the list
        if (this.detailData["index"]) {
          let lIndex = this.detailData["index"];
          let sel = {};
          sel.added = true;
          sel.rows = [];
          sel.rows.push(this.document[lIndex]);

          //Felder setzen
          this.table_selection(sel);
        }
      }
    },

    //Toggle table <-> Detail
    onToggleProtocol(iView) {
      console.log(`IndexPage onToggleProtocol()`);

      if (this.toggleProtocol == "0") {
        if (iView == 2) {
          this.toggleProtocol = "2";
        } else {
          this.toggleProtocol = "1";
        }
      } else {
        this.toggleProtocol = "0";
      }
    },


    async onDownload(blob) {
      console.log(`IndexPage onDownload()`);
      if (!blob) {
        return;
      }
      let that = this;
      invoke("js2rs", {
        message: JSON.stringify({
          path: "save_file",
          query: "",
          data: JSON.stringify({
            id: this.detailData.id,
            file: this.detailData.file,
          }),
        }),
      }).then((data) => that.doFromMain(data));
    },

    //starts for selection doStatus on the server
    doStatus() {
      console.log(`IndexPage doStatus()`);

      if (this.selected.length != 0 && this.selected[0].id != "") {
        this.toggle = "0";

        let that = this;
        invoke("js2rs", {
          message: JSON.stringify({
            path: "dostatus",
            query: "",
            data: this.selected[0].id,
          }),
        }).then((data) => that.doFromMain(data));

      } else {

        this.$q.notify({
          progress: true,
          message: "Please mark a line.",
          color: "warning",
          actions: [
            {
              label: "OK",
              color: "black",
              handler: () => {
                /* ... */
              },
            },
          ],
        });
      }
    },

    getHTML() {
      console.log(`getHTML`)
      if (this.selected.length == 0) { return "" }
      let my_string = this.selected[0]['body'].replaceAll("<script", "<!--script").replaceAll("/script>", "/script-->").replaceAll("eval(", "uneval(");
      return my_string;
    },

    //startet für Selektion doLoop auf dem Server
    doLoop() {
      console.log(`IndexPage doLoop()`);

      let that = this;
      invoke("js2rs", {
        message: JSON.stringify({
          path: "doloop",
          query: "?json=true",
          data: "doloop",
        }),
      }).then((data) => that.doFromMain(data));
    },

    //if amount is changed, also change the raw value
    change_amount(value) {
      console.log(`change_amount(${value})`);
      this.detailData.amount_row = parseLocaleNumber(value, this.langu);
    },

    //Export the table as CSV
    exportTable() {
      console.log(`IndexPage exportTable()`);

      // naive encoding to csv format
      const content = [this.columns.map((col) => wrapCsvValue(col.label))]
        .concat(
          this.document.map((row) =>
            this.columns
              .map((col) =>
                wrapCsvValue(
                  typeof col.field === "function"
                    ? col.field(row)
                    : row[col.field === void 0 ? col.name : col.field],
                  col.format
                )
              )
              .join(",")
          )
        )
        .join("\r\n");

      const status = exportFile("table-export.csv", content, "text/csv");

      if (status !== true) {
        this.$q.notify({
          message: "Browser denied file download...",
          color: "negative",
          icon: "warning",
        });
      }
    },
  },
});
</script>

<template>
  <q-page padding>
    <p id="langu" style="display: none;">{{ langu }}</p>
    <div v-if="toggle == '0'">
      <!-- Button-row if 0 - table -->
      <div class="row">
        <q-space></q-space>
        <q-select flat dense v-model="seach_field" hint="field" outlined :options="cat_seach_field" emit-value
          map-options class="q-ml-sm">
        </q-select>

        <q-input flat dense v-model="search" hint="value" style="min-width: 50%" outlined clearable class="q-ml-sm">
          <template v-slot:after>
            <q-btn flat @click="onSearch()" icon="search"></q-btn>
          </template>
        </q-input>

        <q-input flat dense v-model="maxline" hint="max" style="max-width: 50px" outlined class="q-ml-sm"></q-input>
      </div>

      <!-- table if 0 - table -->
      <q-table style="height: calc(100vh - 230px); margin-top: 10px" :title="$t('Documents')" :rows="document"
        :columns="columns" row-key="index" :no-data-label="$t('empty')" separator="cell" :loading="loading"
        :filter="filter" :visible-columns="visibleColumns" selection="single" v-model:selected="selected"
        :rows-per-page-options="[0]" @selection="table_selection">
        <template v-slot:top-left>
          <div>{{ $t("Documents") }}</div>
          <q-space></q-space>
          <q-btn icon-right="read_more" @click="onToggle('1')" flat class="q-ml-sm">
            <q-tooltip>Detail</q-tooltip>
          </q-btn>
          <q-btn icon-right="elevator" @click="doStatus()" flat class="q-ml-sm">
            <q-tooltip>Process status</q-tooltip>
          </q-btn>
          <q-btn icon-right="all_inclusive" @click="doLoop()" flat class="q-ml-sm">
            <q-tooltip>Loop - scan new Data</q-tooltip>
          </q-btn>
          <q-btn icon-right="cloud_upload" flat @click="dialogUpload = true" class="q-ml-sm">
            <q-tooltip>Upload PDF-File.</q-tooltip>
          </q-btn>
        </template>

        <template v-slot:top-right="props">
          <q-input dense debounce="300" v-model="filter" placeholder="Filter" outlined clearable flat class="q-ml-sm">
            <template v-slot:append>
              <q-icon name="filter_alt"></q-icon>
            </template>
          </q-input>

          <q-space></q-space>

          <q-select v-model="visibleColumns" multiple outlined dense options-dense
            :display-value="$q.lang.table.columns" emit-value map-options :options="columns" option-value="name"
            options-cover style="min-width: 150px">
            <template v-slot:append>
              <q-icon name="settings"></q-icon>
            </template>
          </q-select>

          <q-space></q-space>

          <q-btn icon-right="archive" no-caps @click="exportTable" flat class="q-ml-sm">
            <q-tooltip>CSV Export</q-tooltip>
          </q-btn>
          <q-btn :icon="props.inFullscreen ? 'fullscreen_exit' : 'fullscreen'" flat @click="props.toggleFullscreen"
            class="q-ml-sm">
            <q-tooltip>toggle fullscreen</q-tooltip>
          </q-btn>
        </template>
      </q-table>
    </div>

    <div v-if="toggle == '1'">
      <!-- Button-row if 1 - Detail -->
      <q-btn icon-right="landscape" @click="onToggle('0')" flat>
        <q-tooltip>Tabelle</q-tooltip>
      </q-btn>

      <q-btn icon-right="chevron_left" @click="onLeft()" flat>
        <q-tooltip>Satz zurück</q-tooltip>
      </q-btn>

      <q-btn icon-right="chevron_right" @click="onRight()" flat>
        <q-tooltip>Satz vorwärts</q-tooltip>
      </q-btn>

      <q-btn icon-right="elevator" @click="doStatus()" flat>
        <q-tooltip>Process status</q-tooltip>
      </q-btn>

      <q-btn label="Save" @click="onSubmit" class="tw-bg-lime-300 q-ml-sm" flat></q-btn>
      <q-btn label="Delete" @click="onDelete" class="tw-bg-red-300 q-ml-sm" flat></q-btn>


      <div class="fit row">
        <q-card style="width: 50%; height: calc(100vh - 200px)">
          <q-card-section>
            <div v-if="detailData['id'] != ''" class="q-pa-md">
              <q-form @submit="onSubmit" @reset="onReset" class="q-gutter-md">
                <div class="q-gutter-xs row">

                  <q-input v-model="detailData['index']" label="#" style="max-width: 30px" readonly></q-input>
                  <q-input v-model="detailData['id']" label="id" style="width: 300px" readonly></q-input>
                  <q-select v-model="detailData['status']" label="status" :options="status">
                  </q-select>

                  <q-input v-model="detailData['date']" label="date" style="width: 200px"></q-input>
                  <q-input v-model="detailData['amount']" label="amount" prefix="EUR"
                    @update:model-value="change_amount">
                    <!--template v-slot:control="{
                              id,
                              floatingLabel,
                              value,
                              emitValue,
                            }">
                            <money :id="id" class="q-field__input text-right" :value="value" @input="emitValue"
                              v-bind="moneyFormatForComponent" v-show="floatingLabel"></money>
                          </template-->
                  </q-input>
                  <!--q-select
                        v-model="detailData.filename"
                        :options="detailData['attachment']"
                        options-value="filename"
                        option-label="filename"
                        label="attachment.filename"
                        style="width: 350px"
                        emit-value
                        map-options
                      >
                      </q-select-->
                  <q-select v-model="detailData['category']" label="category" multiple :options="category"
                    style="width: 350px"></q-select>

                </div>

                <q-input v-model="detailData['subject']" label="subject"></q-input>
                <div class="q-gutter-xs row">
                  <q-input v-model="detailData['sender_name']" label="sender_name" style="width: 49%"></q-input>
                  <q-input v-model="detailData['recipient_name']" label="recipient_name" style="width: 49%">
                  </q-input>
                </div>
                <div class="q-gutter-xs row">
                  <q-input v-model="detailData['sender_addr']" label="sender_addr" style="width: 49%"></q-input>
                  <q-input v-model="detailData['recipient_addr']" label="recipient_addr" style="width: 49%">
                  </q-input>
                </div>
                <q-input v-model="detailData['body']" label="body" filled type="textarea" input-style="height: 13em; ">
                </q-input>

              </q-form>
            </div>
          </q-card-section>
        </q-card>

        <!-- Sub-Dialog if 1 - Detail -->
        <q-card style="width: 50%; height: calc(100vh - 200px)">
          <q-card-section>
            <div class="text-subtitle1">
              {{ detailData['filename'] }}
              <q-btn :v-if="detailData['filename']" icon-right="download"
                @click="onDownload(detailData.url_blob ? detailData.url_blob : null)" flat>
                <q-tooltip>Download PDF as file local</q-tooltip>
              </q-btn>
              <q-btn :v-if="detailData['protocol']" icon-right="psychology" @click="onToggleProtocol()" flat>
                <q-tooltip>Toggle between PDF and protocol</q-tooltip>
              </q-btn>
              <q-btn :v-if="detailData['template_name']" icon-right="smart_toy" @click="onToggleProtocol(2)" flat>
                <q-tooltip>Switch to Parser File</q-tooltip>
              </q-btn>

            </div>
          </q-card-section>

          <q-card-section>
            <div v-if="toggleProtocol == '0' && detailData.pdfbase64 != ''">
              <!--iframe :src="detailData.pdfbase64 ? 'data:application/pdf;base64,' + detailData.pdfbase64 : ''" style="height: 75vh; width: 100%"></iframe-->
              <iframe :src="detailData.url_blob ? detailData.url_blob : ''"
                style="height: calc(100vh - 300px); width: 100%"></iframe>
            </div>
            <div v-if="toggleProtocol == '0' && detailData.pdfbase64 == ''" style="height: calc(100vh - 300px);">
              <div v-html="getHTML()" filled style="height: calc(100vh - 280px);overflow: scroll;"></div>
            </div>
            <div v-if="toggleProtocol == '1'">
              <q-input v-model="selected[0]['protocol']" label="protocol" type="textarea"
                input-style="height: 45em;"></q-input>
            </div>
            <div v-if="toggleProtocol == '2'">
              <!--q-input v-model="selected[0]['protocol']" label="protocol" type="textarea"
                input-style="height: 756em;"></q-input-->
            </div>
          </q-card-section>
        </q-card>
      </div>
    </div>

    <!-- subdialog upload-->
    <q-dialog v-model="dialogUpload" persistent class="tw-font-sans">
      <q-card style="min-width: 350px">
        <q-card-section>
          <div class="q-gutter-md row items-start">
            <myuploader id="myuploader" label="File Uploader" multiple accept=".pdf"></myuploader>
          </div>
        </q-card-section>
        <q-card-actions align="right">
          <q-btn flat label="Cancel" v-close-popup class="tw-bg-red-300"></q-btn>
        </q-card-actions>
      </q-card>
    </q-dialog>
  </q-page>
</template>
