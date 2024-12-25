<script setup lang="ts">
const channel: Ref<BroadcastChannel | null> = ref(null);
const messages = ref([]) as Ref<string[]>;
const message = ref("");

onMounted(() => {
  channel.value = new BroadcastChannel('channel');

  channel.value.onmessage = function (event) {
    if (event.data.event === 'message-from-main') {
      messages.value.push(event.data.data);
    }
  };
});

const clear_logs = () => {
  messages.value = [];
};

const post_message_to_main = () => {
  const msg = message.value.trim();
  if (msg) {
    if (channel.value) {
      channel.value.postMessage({event: 'message-to-main', data: msg});
      message.value = '';
    }
  }
};
</script>

<template lang="pug">
  .additional
    h1 Additional
    p This is additional content.
    input(type="text" v-model="message")
    button(@click="post_message_to_main") Send to main
    br
    button(@click="clear_logs") clear log
    ul
      li(v-for="message in messages") {{ message }}
</template>

<style scoped lang="less">

.additional {
  h1 {
    font-size: 20px;
  }

  input[type="text"] {
    margin-right: 5px;
  }

  padding: 5px;
  border: 1px solid green;
  background-color: #d0f8bc;
}
</style>