<script lang="ts">
import { invoke } from "@tauri-apps/api/tauri";
import { defineComponent } from "vue";
import * as echarts from "echarts";

export default defineComponent({
  name: "DashboardPage",
  props: ["langu"],
  components: {},
  //here are the data of the view
  data: () => {
    return {
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

    let that = this;
    invoke("js2rs", {
      message: JSON.stringify({
        path: "chart_count",
        query: "Rechnung",
        data: "count",
      }),
    }).then((data) => that.doFromMain(data));

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


    invoke("js2rs", {
      message: JSON.stringify({
        path: "chart_amount",
        query: "Rechnung",
        data: "amount",
      }),
    }).then((data) => that.doFromMain(data));

  },
  //
  updated() {
    console.log(`DashboardPage updated()`);

  },
  //
  methods: {
    generate_options(chart_name, data, titel, legend, series) {
      console.log(`DashboardPage generate_options()`);
      data.sort((a, b) => {
        let a_split = a.x_value.split('/');
        let b_split = b.x_value.split('/');
        if (`${a_split[1]}${a_split[0] * 10}` < `${b_split[1]}${b_split[0] * 10}`) { return -1 } else { return 1 }
      })
      this[chart_name] = {
        title: {
          text: titel,
        },
        tooltip: {},
        legend: {
          data: [legend],
        },
        grid: {
          left: "3%",
          right: "3%",
          bottom: "3%",
          containLabel: true,
        },
        toolbox: {
          feature: {
            saveAsImage: {
              show: true,
            },
          },
        },
        xAxis: {
          data: [],
        },
        yAxis: {
          type: "value",
        },
        series: [
          {
            name: series,
            type: "line",
            label: {
              show: true,
              position: "top",
            },
            data: [],
          },
        ],
      };
      let that = this;
      data.sort((a, b) => a.x_value > b.x_value);
      data.forEach((element) => {
        that[chart_name].series[0].data.push(
          ((element["y_value"] * 100) / 100).toString()
        );
        that[chart_name].xAxis.data.push(element["x_value"]);
      });
    },
    async doFromMain(iData) {
      console.log(`DashboardPage doFromMain()`);
      console.log(iData.substring(0, 150));

      let data = JSON.parse(iData);
      if (data.data) {
        data.data = JSON.parse(data.data);
      }

      let { dataname: lDataName, data: lData, error: lError } = data;

      if (lError) {
        this.$q.notify({
          message: "Error: " + lError.message,
          color: "negative",
          icon: "warning",
        });
        return;
      }

      if (!lData) {
        return;
      }

      if (lDataName == "count") {
        this.generate_options("option1", lData, "Number of documents", "count", "count");
        // use configuration item and data specified to show chart
        this.Chart1.setOption(this.option1);
        return;
      }

      if (lDataName == "amount") {
        this.generate_options("option2", lData, "Total invoice values", "amount", "amount");
        this.option2.series[0].lineStyle = { color: "red" };
        this.option2.series[0].itemStyle = { color: "red" }

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
