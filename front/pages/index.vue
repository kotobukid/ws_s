<script setup lang="ts">
import {useWasmTest} from "~/composables/wasm_test";
import type {PaneDefinition} from "~/types";

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
  {height: 150, name: 'ChatFront'},
  {height: 212, name: 'TabSync'},
  {height: 48, name: 'UnknownComponent'},
]);

const update_height = (data: { component: string, deltaX: number, deltaY: number }) => {
  for (const component of components.value) {
    if (component.name === data.component) {
      component.height += data.deltaY;
      break;
    }
  }
}
</script>

<template lang="pug">
  FlexFrame(
    :components="components"
    @update-height="update_height"
  )
</template>

<style scoped>

</style>