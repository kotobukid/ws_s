<script setup lang="ts">
import type {ComponentAppType} from "~/types";

const props = defineProps<{current: ComponentAppType}>();

const emits = defineEmits<{
  (e: 'switch-component', value: ComponentAppType): void
}>();

const switch_component = (component: ComponentAppType) => {
  emits('switch-component', component);
}
</script>

<template lang="pug">
  table.component-app-list
    colgroup
      col(style="width: 100%")
      col(style="width: 100%")
      col(style="width: 100%")
    tbody
      tr
        td.component_name(@click="switch_component('ChatFront')" :data-active="current === 'ChatFront' ? 'active' : ''") ChatFront
        td.component_name(@click="switch_component('TabSync')" :data-active="current === 'TabSync' ? 'active' : ''") TabSync
        td.component_name(@click="switch_component('Unknown')" :data-active="current === 'Unknown' ? 'active' : ''") Unknown
</template>

<style scoped lang="less">
table {
  z-index: 1;
  position: relative;
  top: 0;

  width: 300px;
  table-layout: fixed;
  border-collapse: collapse;
}

td {
  cursor: pointer;

  &:hover {
    background-color: #888888;
    color: white;
  }

  padding: 5px;
  outline: 1px solid white;
  background-color: black;
  color: white;
}

td.component_name {
  &[data-active="active"] {
    font-weight: bolder;
    color: red;
  }
}
</style>