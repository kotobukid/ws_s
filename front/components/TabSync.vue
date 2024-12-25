<script setup lang="ts">
const channel: Ref<BroadcastChannel | null> = ref(null);
const messages = ref([]) as Ref<string[]>;
const message = ref('');


onMounted(() => {
  channel.value = new BroadcastChannel('channel');

  channel.value.onmessage = (event) => {
    if (event.data.event === 'message-to-main') {
      messages.value.push(event.data.data);
    }
  };
});

const post_message = () => {
  const msg = message.value.trim();
  if (msg) {
    if (channel.value) {
      channel.value.postMessage({event: 'message-from-main', data: msg});
      message.value = '';
    }
  }
};

const clear_logs = () => {
  messages.value = [];
};
</script>

<template lang="pug">
  .tab_sync
    h1 BroadcastChannel
    a(href="/additional/" target="_blank") open sub tab
    br
    input(type="text" v-model="message")
    button(@click="post_message") broadcast to all subs
    br
    button(@click="clear_logs") clear log
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

  width: 274px;
  border: 1px solid grey;
  padding: 5px;
}

input[type="text"] {
  margin-right: 5px;
}
</style>