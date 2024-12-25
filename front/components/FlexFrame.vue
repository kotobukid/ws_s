<script setup lang="ts">
import type { PaneDefinition } from "~/types";

const props = defineProps<{
  components: PaneDefinition[],
}>();
</script>

<template lang="pug">
  .flex_container
    .component(v-for="component in props.components" :style="`height: ${component.height}px;`")
      h1 {{ component.name }}
      FlexPane(:component="component.name")
</template>

<style scoped lang="less">
.flex_container {
  background-color: #efefef;
}

.component {
  h1 {
    font-size: 10px;
    background-color: #6f7dd5;
    padding: 4px;
    margin: 0;
  }

  padding: 2px;
  outline: 1px solid black;
  margin: 5px;
}
</style>