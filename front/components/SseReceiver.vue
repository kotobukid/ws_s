<script setup lang="ts">
const eventSource: Ref<EventSource | null> = ref(null);
const listening = ref(false);
const last_log = ref('');

const start_subscription = () => {
  eventSource.value = new EventSource('/api/sse');

  eventSource.value.onmessage = (event: MessageEvent<string>) => {
    last_log.value = event.data;
  };

  eventSource.value.onerror = (event: Event) => {
    console.log(event);
    last_log.value = 'error';
  };

  eventSource.value.onopen = (event: Event) => {
    console.log(event);
    last_log.value = 'open';
    listening.value = true;
  };
};

const stop_subscription = () => {
  if (eventSource.value) {

    // 明示的に解除しておく
    eventSource.value.onmessage = null;
    eventSource.value.onerror = null;
    eventSource.value.onopen = null;

    eventSource.value.close();
    eventSource.value = null;
    listening.value = false;
  }
};
</script>

<template lang="pug">
.sse_receiver
  h1 SSE Receiver
  button.start(@click="start_subscription" v-if="!listening") start subscription
  button.stop(@click="stop_subscription" v-if="listening") stop subscription
  br
  span {{ last_log }}
</template>

<style scoped lang="less">
.sse_receiver {
  border: 1px solid blue;
  background-color: #6fccd5;
}

button.start {
  border: 1px solid grey;
  background-color: #bcffbc;
}
button.stop {
  border: 1px solid grey;
  background-color: #ffbc90;
}
</style>