<template>
  <div id="app" class="app-root">
    <header class="app-header">
      <h1>Synapse-Ultra</h1>
      <p>Next-Generation Multiplayer Visual Data-Flow Engine</p>
    </header>

    <main class="app-main">
      <Canvas :room-id="roomId" :server-url="serverUrl" />
    </main>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { initBridge } from '@workers/bridge';
import Canvas from '@components/Canvas.vue';

const roomId = ref('default-workspace');
const serverUrl = ref('ws://localhost:1234');

onMounted(() => {
  // Initialize the worker bridge
  initBridge();

  // Parse URL params for room and server
  const params = new URLSearchParams(window.location.search);
  if (params.has('room')) {
    roomId.value = params.get('room') || 'default-workspace';
  }
  if (params.has('server')) {
    serverUrl.value = params.get('server') || 'ws://localhost:1234';
  }

  console.log('Synapse-Ultra initialized', { roomId: roomId.value, serverUrl: serverUrl.value });
});
</script>

<style scoped>
.app-root {
  width: 100%;
  height: 100vh;
  display: flex;
  flex-direction: column;
  background: #0f172a;
  color: #f1f5f9;
  font-family: 'Inter', 'Helvetica Neue', sans-serif;
}

.app-header {
  padding: 16px 24px;
  border-bottom: 1px solid rgba(71, 85, 105, 0.3);
  background: rgba(15, 23, 42, 0.95);
  backdrop-filter: blur(10px);
  z-index: 50;
}

.app-header h1 {
  margin: 0;
  font-size: 24px;
  font-weight: 700;
  letter-spacing: -0.5px;
}

.app-header p {
  margin: 4px 0 0;
  font-size: 12px;
  color: #94a3b8;
  font-weight: 400;
}

.app-main {
  flex: 1;
  overflow: hidden;
  position: relative;
}
</style>

<style>
* {
  box-sizing: border-box;
}

html, body {
  margin: 0;
  padding: 0;
  width: 100%;
  height: 100%;
  overflow: hidden;
}

#app {
  width: 100%;
  height: 100%;
}
</style>
