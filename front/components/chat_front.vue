<script setup lang="ts">
import {onMounted, ref} from "vue";
import {useWS} from "~/composables/websocket";
import {useRuntimeConfig} from '#imports';
import {useWasmTest} from "#imports";

const {connect} = useWS();
const config = useRuntimeConfig();

const {create_text_message, create_list_message} = useWasmTest();

let ws_url = config.public.wsHost as string;

const ws: WebSocket = connect(ws_url);

const message = ref('');
const last_log = ref('')

const str_to_binary = (() => {
  const encoder = new TextEncoder();

  return (str: string) => encoder.encode(str);
})();

const binary_to_str = (() => {
  const decoder = new TextDecoder();
  return (bin: Uint8Array) => decoder.decode(bin);
})();

onMounted(() => {
  ws.onmessage = (event) => {

    // データの型を確認
    if (event.data instanceof Blob) {
      // バイナリデータの場合
      event.data.arrayBuffer().then(buffer => {
        const uint8Array = new Uint8Array(buffer);

        // ここでバイナリデータを処理
        // 例：最初の1バイトを見て処理を分岐
        // const firstByte = uint8Array[0];

        // 残りのデータをテキストとして処理する例
        // const decoder = new TextDecoder();
        // const text = decoder.decode(uint8Array.slice(1));

        const text = binary_to_str(uint8Array);

        last_log.value = `binary(${text})`;
      });
    } else {
      // テキストデータの場合
      console.log(event.data);
      last_log.value = `text(${event.data})`;
    }
  };

  ws.addEventListener('close', () => {
    console.log('closed(client)');
  });

  ws.addEventListener('open', () => {
    console.log('connected(client)');
    if (ws) {
      ws.send("hello");
    }
  });
});

const send_text = () => {
  let value = message.value.trim();

  if (value) {
    if (ws) {
      ws.send(value);
    }
    message.value = '';
  }
}

const send_binary = () => {
  if (message.value) {
    const chat_message_b = create_text_message("taro", 42, message.value);

    if (ws) {
      ws.send(chat_message_b);
    }
    message.value = '';
  }
}

const list_socket = () => {
  const list_message = create_list_message("taro", 42, "");
  if (ws) {
    ws.send(list_message);
  }
}
</script>

<template lang="pug">
  form
    input#input(type="text" v-model="message")
    button#send_button(@click.prevent="send_text") send
    button#send_button2(@click.prevent="send_binary") send as binary
    br
    button#send_button_list_socket(@click.prevent="list_socket") list socket
  br
  pre#output(v-text="last_log")
</template>

<style scoped>

</style>