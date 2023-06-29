<script lang="ts">
  import { invoke } from "@tauri-apps/api/tauri";
  import { defineComponent } from "vue";
  import * as echarts from "echarts";

  export default defineComponent({
    name: "DashboardPage",
    props: ["langu", "data"],
    components: {},
    //here are the data of the view
    data: () => {
      return {
        ServerData: undefined,

        Chart1: {},
        option1: {},
        Chart2: {},
        option2: {},
      };
    },

    computed: {},
    created() {
      console.log(`DashboardPage created()`);
    },
    mounted() {
      // based on prepared DOM, initialize echarts instance
      console.log(`DashboardPage mounted()`);

      if (this.ServerData !== this.data && this.data) {
        this.doFromMain(this.data);
        this.ServerData = this.data;
      }

      // based on prepared DOM, initialize echarts instance
      let l_graph1 = document.getElementById("graph1");
      if (l_graph1) {
        this.Chart1 = echarts.init(l_graph1);
      }

      // specify chart configuration item and data
      this.option1 = {
        title: {
          text: "Number of documents",
        },
        tooltip: {},
        legend: {
          data: ["count"],
        },
        xAxis: {
          data: ["", "", "", "", "", ""],
        },
        yAxis: {
          type: "value",
        },
        series: [
          {
            name: "count",
            type: "line",
            data: [0, 0, 0, 0, 0, 0],
          },
        ],
      };

      //
      // window.electronAPI.send("toMain", {
      //   path: "chart1",
      //   query: "",
      //   data: "option1",
      // });
      invoke("js2rs", {
        message: JSON.stringify({
          path: "chart1",
          query: "",
          data: "option1",
        }),
      });

      // based on prepared DOM, initialize echarts instance
      let l_graph2 = document.getElementById("graph2");
      if (l_graph2) {
        this.Chart2 = echarts.init(l_graph2);
      }

      // specify chart configuration item and data
      this.option2 = {
        title: {
          text: "Total invoice values",
        },
        tooltip: {},
        legend: {
          data: ["amount"],
        },
        xAxis: {
          data: ["", "", "", "", "", ""],
        },
        yAxis: {
          type: "value",
        },
        series: [
          {
            name: "amount",
            type: "line",
            data: [0, 0, 0, 0, 0, 0],
          },
        ],
      };

      // window.electronAPI.send("toMain", {
      //   path: "chart2",
      //   query: "",
      //   data: "option2",
      // });
      invoke("js2rs", {
        message: JSON.stringify({
          path: "chart2",
          query: "",
          data: "option2",
        }),
      });
    },
    //
    updated() {
      console.log(`DashboardPage updated()`);

      if (this.ServerData !== this.data && this.data) {
        this.doFromMain(this.data);
        this.ServerData = this.data;
      }
    },
    //
    methods: {
      async doFromMain(iData) {
        console.log(`DashboardPage doFromMain()`);
        console.log(iData);

        let that = this;

        let { dataname: lDataName, data: lData, error: lError } = iData;

        if (lError) {
          this.$q.notify({
            message: "Fehler: " + lError.message,
            color: "negative",
            icon: "warning",
          });
          return;
        }

        if (!lData) {
          return;
        }

        if (lDataName == "option1") {
          this.option1 = lData;
          // use configuration item and data specified to show chart
          this.Chart1.setOption(this.option1);
          return;
        }

        if (lDataName == "option2") {
          this.option2 = lData;
          // use configuration item and data specified to show chart
          this.Chart2.setOption(this.option2);
          return;
        }
      },
    },
  });
</script>

<template>
  <div class="q-pa-md tw-flex tw-flex-wrap tw-justify-center">
    <q-card class="tw-w-1/2">
      <q-card-section>
        <div id="graph1" class="tw-h-96"></div>
      </q-card-section>
    </q-card>
    <q-card class="tw-w-1/2">
      <q-card-section>
        <div id="graph2" class="tw-h-96"></div>
      </q-card-section>
    </q-card>
  </div>
</template>
