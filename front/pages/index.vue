<script setup lang="ts">
import {useWasmTest} from "~/composables/wasm_test";
import type {ComponentAppType, PaneDefinition} from "~/types";

const {init, test, create_text_message, deserialize_text_message} = useWasmTest();

await init();
// await test();
const chat_message_b = create_text_message("taro", 42, "good morning");
const chat_message_a = deserialize_text_message(chat_message_b);

console.log({
      chat_message_b,
      chat_message_a
    }
);

fetch('/api/health.json').then(res => res.json()).then(json => {
  console.log(json);
});

const components: Ref<PaneDefinition[]> = ref([
  {height: 150, component: 'ChatFront', id: 'chat_front'},
  {height: 212, component: 'TabSync', id: 'tab_sync'},
  {height: 80, component: 'Unknown', id: 'unknown_component'},
]);

const update_height = (data: { id: string, deltaX: number, deltaY: number }) => {
  for (const component of components.value) {
    if (component.id === data.id) {
      component.height += data.deltaY;
      break;
    }
  }
};

const switch_component = ({component_name, id}: { component_name: ComponentAppType, id: string }) => {
  for (const component of components.value) {
    if (component.id === id) {
      component.component = component_name;
      break;
    }
  }
};
</script>

<template lang="pug">
  FlexFrame(
    :components="components"
    @update-height="update_height"
    @switch-component="switch_component"
  )
</template>

<style scoped>

</style>