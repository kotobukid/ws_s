<script setup lang="ts">
import type {PaneDefinition} from "~/types";

const props = defineProps<{
  components: PaneDefinition[],
}>();

const emits = defineEmits<{
  (e: 'update-height', value: {
    component: string,
    deltaX: number,
    deltaY: number,
  }): void
}>()

const dragging = ref(false);
const target_component = ref('');

const start_drag = (event: MouseEvent, component_name: string) => {
  target_component.value = component_name;
  dragging.value = true;
};

const drag_move = (event: PointerEvent) => {
  emits('update-height', {
    component: target_component.value,
    deltaX: event.movementX,
    deltaY: event.movementY,
  });
};

const commit_drag = (e: PointerEvent) => {
  dragging.value = false;
};
</script>

<template lang="pug">
  .flex_container
    .component.handle_parent(v-for="component in props.components" :style="`height: ${component.height}px; max-height: ${component.height}px;`")
      h1 {{ component.name }}
      FlexPane(:component="component.name")
      .handle.extend_handle.handle_bottom(@pointerdown="start_drag($event, component.name)")
    .drag_screen(v-if="dragging"
      @pointermove="drag_move"
      @pointerup="commit_drag"
      @pointerleave="commit_drag"
    )
</template>

<style scoped lang="less">
.flex_container {
  background-color: #efefef;

  width: 300px;
}

.handle_parent {
  position: relative;
}

.handle {
  position: absolute;
  left: 0;
  width: 100%;
  bottom: 0;
  user-select: none;
}

.component {
  h1 {
    font-size: 10px;
    background-color: #6f7dd5;
    padding: 4px;
    margin: 0;
  }

  overflow-y: auto;

  padding: 2px;
  outline: 1px solid black;
  margin: 5px;

  .extend_handle {
    &.handle_bottom {
      cursor: row-resize;
      height: 4px;
      background-color: transparent;

      &:hover {
        background-color: #0e5e14;
      }
    }
  }
}

.drag_screen {
  padding: 0;
  margin: 0;
  position: absolute;
  top: 0;
  left: 0;
  width: 100vw;
  height: 100vh;
  background-color: blueviolet;
  opacity: 0.1;

  cursor: row-resize;
}
</style>