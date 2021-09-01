/* eslint-disable */
<template>
  <el-menu class="el-menu-demo" mode="horizontal" background-color="#EEEEEE">
    <div class="status">
      <p>{{hostname}}</p>
    </div>
  </el-menu>
  <el-row :gutter="10">
    <el-col :xs="0" :sm="1" :md="3" :lg="4" :xl="4"></el-col>
    <el-col :xs="24" :sm="22" :md="18" :lg="16" :xl="16">
      <el-table
          :data="entries"
          style="width: 100%"
          v-loading="loading"
          @current-change="changeDir"
          class="table">
        <el-table-column
            prop="path"
            label="File">
          <template #default="scope">
            <a v-if="scope.row.is_directory" v-bind:href="'${STATIC_PREFIX}' + scope.row.path">{{ scope.row.path }}</a>
            <a v-else v-bind:href="'${API_PREFIX}/download/' + scope.row.path">{{ scope.row.path }}</a>
          </template>
        </el-table-column>
        <el-table-column
            prop="type"
            label="Type">
          <template #default="scope">
            <p v-if="scope.row.is_directory">Directory</p>
            <p v-else>File</p>
          </template>
        </el-table-column>
      </el-table>
    </el-col>
    <el-col :xs="0" :sm="1" :md="3" :lg="4" :xl="4"></el-col>
  </el-row>

</template>

<script>
import {ref, onMounted} from "vue";

export default {
  name: 'directory-explorer',
  setup() {
    const entries = ref([]);
    const loading = ref(true)
    const hostname = ref("")
    onMounted(() => {
      function checkIfOk() {
        return res => {
          if(res.ok) {
            return res.json();
          }
          throw new Error("Cannot get list")
        }
      }

      return Promise.all([
        fetch("${API_PREFIX}/list" + window.location.pathname.replaceAll("${STATIC_PREFIX}", "")).then(checkIfOk()),
        fetch("${API_PREFIX}/instance/info").then(checkIfOk())
      ])
          .then(res => {
            entries.value = res[0]
            hostname.value = res[1].host
            loading.value = false
          });
    });
    return {
      entries,
      loading,
      hostname
    }
  },
  created() {

  },
  methods: {
    changeDir(value) {
      console.log(value);
    }
  }

}
</script>

<style lang="css" scoped>
  .status {
    text-align: center;
    color: rgb(144, 147, 153);
  }
</style>
