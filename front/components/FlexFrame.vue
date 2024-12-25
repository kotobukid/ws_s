<script setup lang="ts">
import type {ComponentAppType, PaneDefinition} from "~/types";

const props = defineProps<{
  components: PaneDefinition[],
}>();

const emits = defineEmits<{
  (e: 'update-height', value: {
    id: string,
    deltaX: number,
    deltaY: number,
  }): void,
  (e: 'switch-component', value: { component_name: ComponentAppType, id: string }): void,
}>();

const dragging = ref(false);
const target_component = ref('');

const start_drag = (event: MouseEvent, component_id: string) => {
  target_component.value = component_id;
  dragging.value = true;
};

const drag_move = (event: PointerEvent) => {
  emits('update-height', {
    id: target_component.value,
    deltaX: 0,
    deltaY: event.movementY,
  });
};

const commit_drag = (e: PointerEvent) => {
  dragging.value = false;
};

const switch_component = (component: string, id: string) => {
  emits('switch-component', {component_name: component, id});
};

</script>

<template lang="pug">
  .flex_container
    .component.handle_parent(v-for="component in props.components" :style="`height: ${component.height}px; max-height: ${component.height}px;`")
      FlexPane(
        :component="component.component"
        @switch-component="switch_component($event, component.id)"
      )
      .handle.extend_handle.handle_bottom(@pointerdown="start_drag($event, component.id)")
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
  opacity: 0;

  cursor: row-resize;
}
</style>