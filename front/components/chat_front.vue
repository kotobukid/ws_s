<script setup lang="ts">
import {onMounted, type Ref, ref} from "vue";
import {useWS} from "~/composables/websocket";

const {connect} = useWS();
const ws: WebSocket = connect();
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
  // ws.value = new WebSocket('http://localhost:8080/ws'); // OK
  // ws.value = new WebSocket('ws://localhost:8080/ws'); // OK
  // ws.value = new WebSocket('ws://localhost:3000/ws'); //  NG
  // ws.value = new WebSocket('http://127.0.0.1:3000/ws'); //
  // ws.value = new WebSocket('http://localhost:3000/ws');
  // ws.value = new WebSocket(`${location.hostname}/ws`);

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
    let v = str_to_binary(message.value);
    if (ws) {
      ws.send(v);
    }
    message.value = '';
  }
}
</script>

<template lang="pug">
  form
    input#input(type="text" v-model="message")
    button#send_button(@click.prevent="send_text") send
    button#send_button2(@click.prevent="send_binary") send as binary
  br
  span#output(v-text="last_log")
</template>

<style scoped>

</style>