<template>
  <div>
    <Chat_front />
<!--    <NuxtRouteAnnouncer />-->
<!--    <NuxtWelcome />-->
  </div>
</template>
<script setup lang="ts">
import Chat_front from "~/components/chat_front.vue";

fetch('/api/health.json').then(res => res.json()).then(json => {
  console.log(json);
});
</script>