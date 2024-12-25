<script setup lang="ts">
const channel = new BroadcastChannel('channel');
const messages = ref([]) as Ref<string[]>;
const message = ref('');

const post_message = () => {
  const msg = message.value.trim();
  if (msg) {
    channel.postMessage({event: 'message-from-main', data: msg});
    message.value = '';
  }
};

channel.onmessage = (event) => {
  if (event.data.event === 'message-to-main') {
    messages.value.push(event.data.data);
  }
};

const clear_log = () => {
  messages.value = [];
};
</script>

<template lang="pug">
  .tab_sync
    h1 BroadcastChannel
    a(href="/additional/" target="_blank") open sub tab
    br
    input#input(type="text" v-model="message")
    button(@click="post_message") broadcast to all sub
    br
    button(@click="clear_log") clear log
    ul
      li(v-for="message in messages") {{ message }}
</template>

<style scoped lang="less">
h1 {
  font-size: 20px;
  padding: 0;
  margin: 0;
}

.tab_sync {
  background-color: #90d56f;

  width: 500px;
  border: 1px solid grey;
  padding: 5px;
}

input[type="text"] {
  margin-right: 5px;
}
</style>